use crate::controller::{
    State, TransitionMessage,
    messages::{
        NO_FILE_RECEIVED_STAPLE, NO_SCAFFOLD_SEQUENCE_SET, NO_SCAFFOLD_SET, ORIGAMI_FILTERS,
        successful_staples_export_msg,
    },
    normal_state::NormalState,
};
use crate::ensnano_consts::ORIGAMI_EXTENSION;
use crate::{
    MainStateView,
    controller::download_staples::{DownloadStapleError, DownloadStapleOk, StaplesDownloader},
    dialog,
};
use dialog::{MustAckMessage, PathInput};
use std::path::PathBuf;

#[derive(Default)]
pub(super) struct DownloadIntervals {
    step: Step,
}

#[derive(Default)]
enum Step {
    /// The staple downloading request has just started
    #[default]
    Init,
    /// Asking the user where to write the result
    AskingPath(AskingPath_),
    /// The path was asked, waiting for user to chose it
    PathAsked {
        path_input: PathInput,
        design_id: usize,
    },
    /// Downloading
    Downloading { design_id: usize, path: PathBuf },
}

impl State for DownloadIntervals {
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn State> {
        let downloader = main_state.get_staple_downloader();
        match self.step {
            Step::Init => get_design_providing_staples(downloader.as_ref()),
            Step::AskingPath(state) => ask_path(state, main_state),
            Step::PathAsked {
                path_input,
                design_id,
            } => poll_path(path_input, design_id),
            Step::Downloading { design_id, path } => {
                download_staples(downloader.as_ref(), design_id, path)
            }
        }
    }
}

fn get_design_providing_staples(downloader: &dyn StaplesDownloader) -> Box<dyn State> {
    let result = downloader.download_staples();
    match result {
        Ok(DownloadStapleOk { warnings }) => AskingPath_ {
            warnings,
            design_id: 0,
            warning_ack: None,
        }
        .to_state(),
        Err(DownloadStapleError::NoScaffoldSet) => TransitionMessage::new(
            NO_SCAFFOLD_SET,
            rfd::MessageLevel::Error,
            Box::new(NormalState),
        ),
        Err(DownloadStapleError::ScaffoldSequenceNotSet) => TransitionMessage::new(
            NO_SCAFFOLD_SEQUENCE_SET,
            rfd::MessageLevel::Error,
            Box::new(NormalState),
        ),
    }
}

fn ask_path(mut state: AskingPath_, main_state: &MainStateView) -> Box<DownloadIntervals> {
    if let Some(must_ack) = state.warning_ack.as_ref()
        && !must_ack.was_ack()
    {
        Box::new(DownloadIntervals {
            step: Step::AskingPath(state),
        })
    } else if let Some(msg) = state.warnings.pop() {
        let must_ack = dialog::blocking_message(msg.into(), rfd::MessageLevel::Warning);
        state.with_ack(must_ack)
    } else {
        let candidate_name = main_state.get_current_file_name().map(|p| {
            let mut ret = p.to_owned();
            ret.set_extension(ORIGAMI_EXTENSION);
            ret
        });
        let starting_directory = main_state.get_current_design_directory();
        let path_input =
            dialog::get_file_to_write(ORIGAMI_FILTERS, starting_directory.as_ref(), candidate_name);
        Box::new(DownloadIntervals {
            step: Step::PathAsked {
                path_input,
                design_id: state.design_id,
            },
        })
    }
}

struct AskingPath_ {
    warnings: Vec<String>,
    design_id: usize,
    warning_ack: Option<MustAckMessage>,
}

impl AskingPath_ {
    fn to_state(self) -> Box<DownloadIntervals> {
        Box::new(DownloadIntervals {
            step: Step::AskingPath(self),
        })
    }

    fn with_ack(mut self, ack: MustAckMessage) -> Box<DownloadIntervals> {
        self.warning_ack = Some(ack);
        self.to_state()
    }
}

fn poll_path(path_input: PathInput, design_id: usize) -> Box<dyn State> {
    if let Some(result) = path_input.get() {
        if let Some(path) = result {
            Box::new(DownloadIntervals {
                step: Step::Downloading { path, design_id },
            })
        } else {
            TransitionMessage::new(
                NO_FILE_RECEIVED_STAPLE,
                rfd::MessageLevel::Error,
                Box::new(NormalState),
            )
        }
    } else {
        Box::new(DownloadIntervals {
            step: Step::PathAsked {
                path_input,
                design_id,
            },
        })
    }
}

fn download_staples(
    downloader: &dyn StaplesDownloader,
    _design_id: usize,
    path: PathBuf,
) -> Box<dyn State> {
    downloader.write_intervals(&path);
    let msg = successful_staples_export_msg(&path);
    TransitionMessage::new(msg, rfd::MessageLevel::Error, Box::new(NormalState))
}
