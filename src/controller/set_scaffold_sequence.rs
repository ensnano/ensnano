use crate::{
    MainStateView,
    controller::{
        AutomataState, TransitionMessage, YesNo,
        messages::{
            NO_FILE_RECEIVED_SCAFFOLD, SEQUENCE_FILTERS, invalid_sequence_file,
            optimize_scaffold_position_msg,
        },
        normal_state::NormalState,
    },
    dialog::{self, PathInput},
};
use ensnano_utils::StandardSequence;
use std::path::{Path, PathBuf};

/// User is in the process of setting the sequence of the scaffold
pub(super) struct SetScaffoldSequence {
    step: Step,
    shift: usize,
}

impl SetScaffoldSequence {
    pub(super) fn init(shift: usize) -> Self {
        Self {
            shift,
            step: Default::default(),
        }
    }

    pub(super) fn optimize_shift() -> Self {
        Self {
            shift: 0,
            step: Step::OptimizeScaffoldPosition { design_id: 0 },
        }
    }

    fn use_default(shift: usize, sequence: StandardSequence) -> Self {
        let sequence = sequence.sequence().to_owned();
        Self {
            step: Step::SetSequence(sequence),
            shift,
        }
    }

    fn ask_path(shift: usize) -> Self {
        Self {
            step: Step::AskPath { path_input: None },
            shift,
        }
    }
}

#[derive(Default)]
enum Step {
    /// The request to set the sequence of the scaffold has been acknowledged. User is asked to
    /// chose between the default m13 scaffold or a custom one.
    #[default]
    Init,
    /// The user has chosen to use a custom scaffold, and is asked a path the sequence file.
    AskPath { path_input: Option<PathInput> },
    /// The user has chosen a sequence file. The content of the file is checked.
    GotPath(PathBuf),
    /// The new sequence of the scaffold has been decided, user is asked if they want to optimize
    /// the starting position
    SetSequence(String),
    /// The user has chosen to optimize the scaffold position.
    OptimizeScaffoldPosition { design_id: usize },
}

impl AutomataState for SetScaffoldSequence {
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn AutomataState> {
        match self.step {
            Step::Init => init_set_scaffold_sequence(self.shift, main_state.get_scaffold_length()),
            Step::AskPath { path_input } => ask_path(
                path_input,
                self.shift,
                main_state.get_current_design_directory(),
            ),
            Step::GotPath(path) => got_path(path, self.shift),
            Step::SetSequence(sequence) => set_sequence(sequence, self.shift, main_state),
            Step::OptimizeScaffoldPosition { design_id } => {
                optimize_scaffold_position(design_id, main_state)
            }
        }
    }
}

fn init_set_scaffold_sequence(
    shift: usize,
    scaffold_length: Option<usize>,
) -> Box<dyn AutomataState> {
    let suggested_sequence = scaffold_length
        .map(StandardSequence::from_length)
        .unwrap_or_default();
    let desc = suggested_sequence.description();
    let message = format!(
        "Use {desc} sequence?
    If you chose no, you will be ask to chose a file containing the scaffold sequence."
    );

    let yes = Box::new(SetScaffoldSequence::use_default(shift, suggested_sequence));
    let no = Box::new(SetScaffoldSequence::ask_path(shift));

    Box::new(YesNo::new(message, yes, no))
}

fn ask_path<P: AsRef<Path>>(
    path_input: Option<PathInput>,
    shift: usize,
    starting_directory: Option<P>,
) -> Box<dyn AutomataState> {
    if let Some(path_input) = path_input {
        if let Some(result) = path_input.get() {
            if let Some(path) = result {
                Box::new(SetScaffoldSequence {
                    step: Step::GotPath(path),
                    shift,
                })
            } else {
                TransitionMessage::new(
                    NO_FILE_RECEIVED_SCAFFOLD,
                    rfd::MessageLevel::Error,
                    Box::new(NormalState),
                )
            }
        } else {
            Box::new(SetScaffoldSequence {
                step: Step::AskPath {
                    path_input: Some(path_input),
                },
                shift,
            })
        }
    } else {
        let path_input = dialog::load(starting_directory, SEQUENCE_FILTERS);
        Box::new(SetScaffoldSequence {
            step: Step::AskPath {
                path_input: Some(path_input),
            },
            shift,
        })
    }
}

fn got_path(path: PathBuf, shift: usize) -> Box<dyn AutomataState> {
    let mut content = std::fs::read_to_string(path).unwrap();
    content.make_ascii_uppercase();
    if let Some(n) =
        content.find(|c: char| c != 'A' && c != 'T' && c != 'G' && c != 'C' && !c.is_whitespace())
    {
        let msg = invalid_sequence_file(n);
        TransitionMessage::new(msg, rfd::MessageLevel::Error, Box::new(NormalState))
    } else {
        Box::new(SetScaffoldSequence {
            step: Step::SetSequence(content),
            shift,
        })
    }
}

fn set_sequence(
    sequence: String,
    shift: usize,
    scaffold_setter: &mut MainStateView,
) -> Box<dyn AutomataState> {
    let result = scaffold_setter.set_scaffold_sequence(sequence, shift);
    match result {
        Ok(SetScaffoldSequenceOk {
            default_shift,
            target_scaffold_length,
        }) => match target_scaffold_length {
            TargetScaffoldLength::Ok => {
                let message = optimize_scaffold_position_msg(default_shift.unwrap_or(0));
                let yes = Box::new(SetScaffoldSequence {
                    step: Step::OptimizeScaffoldPosition { design_id: 0 },
                    shift,
                });
                let no = Box::new(NormalState);
                Box::new(YesNo::new(message, yes, no))
            }
            TargetScaffoldLength::NotOk {
                design_length,
                input_scaffold_length,
            } => TransitionMessage::new(
                format!(
                    "Current scaffold length and input sequence length are different.
                Current scaffold length: {design_length}
                Input sequence length: {input_scaffold_length}"
                ),
                rfd::MessageLevel::Warning,
                Box::new(NormalState),
            ),
        },
        Err(err) => TransitionMessage::new(
            format!("{err:?}"),
            rfd::MessageLevel::Error,
            Box::new(NormalState),
        ),
    }
}

fn optimize_scaffold_position(
    _design_id: usize,
    main_state: &mut MainStateView,
) -> Box<dyn AutomataState> {
    main_state.optimize_shift();
    Box::new(NormalState)
}

pub(crate) trait ScaffoldSetter {
    fn get_scaffold_length(&self) -> Option<usize>;
    fn set_scaffold_sequence(
        &mut self,
        sequence: String,
        shift: usize,
    ) -> Result<SetScaffoldSequenceOk, SetScaffoldSequenceError>;
    fn optimize_shift(&mut self);
}

pub(crate) struct SetScaffoldSequenceOk {
    pub default_shift: Option<usize>,
    pub target_scaffold_length: TargetScaffoldLength,
}

pub(crate) enum TargetScaffoldLength {
    Ok,
    NotOk {
        design_length: usize,
        input_scaffold_length: usize,
    },
}

#[derive(Debug)]
pub(crate) struct SetScaffoldSequenceError(#[expect(unused)] pub String);
