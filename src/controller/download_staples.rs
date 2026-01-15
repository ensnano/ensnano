use crate::{
    app_state::design_interactor::DesignInteractor,
    controller::{
        AutomataState, TransitionMessage,
        messages::{
            NO_FILE_RECEIVED_STAPLE, NO_SCAFFOLD_SEQUENCE_SET, NO_SCAFFOLD_SET, STAPLES_FILTERS,
            successful_staples_export_msg,
        },
        normal_state::NormalState,
    },
    dialog,
    state::MainStateView,
};
use dialog::{MustAckMessage, PathInput};
use std::path::PathBuf;

#[derive(Default)]
pub(super) struct DownloadStaples {
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
    Downloading { path: PathBuf },
}

impl AutomataState for DownloadStaples {
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn AutomataState> {
        let downloader = main_state.get_design_interactor();
        match self.step {
            Step::Init => get_design_providing_staples(&downloader),
            Step::AskingPath(state) => ask_path(state, main_state),
            Step::PathAsked {
                path_input,
                design_id,
            } => poll_path(path_input, design_id),
            Step::Downloading { path } => download_staples(&downloader, path),
        }
    }
}

fn get_design_providing_staples(downloader: &DesignInteractor) -> Box<dyn AutomataState> {
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

fn ask_path(mut state: AskingPath_, main_state: &MainStateView) -> Box<DownloadStaples> {
    if let Some(must_ack) = state.warning_ack.as_ref()
        && !must_ack.was_ack()
    {
        Box::new(DownloadStaples {
            step: Step::AskingPath(state),
        })
    } else if let Some(msg) = state.warnings.pop() {
        let must_ack = dialog::blocking_message(msg.into(), rfd::MessageLevel::Warning);
        state.with_ack(must_ack)
    } else {
        let path_input = dialog::get_file_to_write(
            STAPLES_FILTERS,
            main_state.get_current_design_directory(),
            main_state.get_current_file_name(),
        );
        Box::new(DownloadStaples {
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
    fn to_state(self) -> Box<DownloadStaples> {
        Box::new(DownloadStaples {
            step: Step::AskingPath(self),
        })
    }

    fn with_ack(mut self, ack: MustAckMessage) -> Box<DownloadStaples> {
        self.warning_ack = Some(ack);
        self.to_state()
    }
}

fn poll_path(path_input: PathInput, design_id: usize) -> Box<dyn AutomataState> {
    if let Some(result) = path_input.get() {
        if let Some(path) = result {
            Box::new(DownloadStaples {
                step: Step::Downloading { path },
            })
        } else {
            TransitionMessage::new(
                NO_FILE_RECEIVED_STAPLE,
                rfd::MessageLevel::Error,
                Box::new(NormalState),
            )
        }
    } else {
        Box::new(DownloadStaples {
            step: Step::PathAsked {
                path_input,
                design_id,
            },
        })
    }
}

fn download_staples(downloader: &DesignInteractor, path: PathBuf) -> Box<dyn AutomataState> {
    downloader.write_staples_xlsx(&path);
    let msg = successful_staples_export_msg(&path);
    TransitionMessage::new(msg, rfd::MessageLevel::Error, Box::new(NormalState))
}

pub(crate) enum DownloadStapleError {
    /// No strand is set as the scaffold
    NoScaffoldSet,
    /// There is no sequence set for the scaffold
    ScaffoldSequenceNotSet,
}

pub(crate) struct DownloadStapleOk {
    pub warnings: Vec<String>,
}
