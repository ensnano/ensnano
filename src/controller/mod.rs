//! Handles windows and dialog (Alert, and file pickers) interactions.

pub(crate) mod channel_reader;
mod download_intervals;
pub(crate) mod download_staples;
mod messages;
pub(crate) mod normal_state;
mod quit;
pub(crate) mod set_scaffold_sequence;

use crate::MainStateView;
use crate::dialog::{self, MustAckMessage, YesNoQuestion};
use ensnano_design::scadnano::ScadnanoImportError;
use ensnano_utils::consts::CANNOT_OPEN_DEFAULT_DIR;
use normal_state::NormalState;
use std::borrow::Cow;

pub(crate) struct Controller {
    /// The sate of the windows
    state: Box<dyn State + 'static>,
}

impl Controller {
    pub(crate) fn new() -> Self {
        Self {
            state: Box::new(NormalState),
        }
    }

    /// This function is called to update the state of ENSnano. Its behavior depends on the state
    /// of the [Controller](`Controller`).
    pub(crate) fn make_progress(&mut self, main_state: &mut MainStateView) {
        main_state.check_backup();
        if main_state.need_backup() {
            if let Err(e) = main_state.save_backup() {
                log::error!("{e:?}");
            }
        } else {
            let old_state = std::mem::replace(&mut self.state, Box::new(OhNo));
            self.state = old_state.make_progress(main_state);
        }
    }
}

pub(crate) trait State {
    /// Operate on [MainStateView] and return the new State of the automata
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn State>;
}

/// A dummy state that should never be constructed.
///
/// It is used as an argument to `std::mem::take`.
struct OhNo;

impl State for OhNo {
    fn make_progress(self: Box<Self>, _: &mut MainStateView) -> Box<dyn State> {
        panic!("Oh No !")
    }
}

/// Display a message that must be acknowledged by the user, and transition to a predetermined
/// state.
struct TransitionMessage {
    level: rfd::MessageLevel,
    content: Cow<'static, str>,
    ack: Option<MustAckMessage>,
    transition_to: Box<dyn State>,
}

impl TransitionMessage {
    fn new<S: Into<Cow<'static, str>>>(
        content: S,
        level: rfd::MessageLevel,
        transition_to: Box<dyn State + 'static>,
    ) -> Box<Self> {
        Box::new(Self {
            level,
            content: content.into(),
            ack: None,
            transition_to,
        })
    }
}

impl State for TransitionMessage {
    fn make_progress(mut self: Box<Self>, _: &mut MainStateView) -> Box<dyn State + 'static> {
        if let Some(ack) = self.ack.as_ref() {
            if ack.was_ack() {
                self.transition_to
            } else {
                self
            }
        } else {
            let ack = dialog::blocking_message(self.content.clone(), clone_msg_level(&self.level));
            self.ack = Some(ack);
            self
        }
    }
}

// TODO: Remove this function? rfd::MessageLevel already implements Clone
fn clone_msg_level(level: &rfd::MessageLevel) -> rfd::MessageLevel {
    match level {
        rfd::MessageLevel::Warning => rfd::MessageLevel::Warning,
        rfd::MessageLevel::Info => rfd::MessageLevel::Info,
        rfd::MessageLevel::Error => rfd::MessageLevel::Error,
    }
}

/// Ask the user a yes/no question and transition to a state that depends on their answer.
struct YesNo {
    question: Cow<'static, str>,
    answer: Option<YesNoQuestion>,
    yes: Box<dyn State>,
    no: Box<dyn State>,
}

impl YesNo {
    fn new<S: Into<Cow<'static, str>>>(
        question: S,
        yes: Box<dyn State>,
        no: Box<dyn State>,
    ) -> Self {
        Self {
            question: question.into(),
            yes,
            no,
            answer: None,
        }
    }
}

impl State for YesNo {
    fn make_progress(mut self: Box<Self>, _: &mut MainStateView) -> Box<dyn State> {
        if let Some(ans) = self.answer.as_ref() {
            if let Some(b) = ans.answer() {
                if b { self.yes } else { self.no }
            } else {
                self
            }
        } else {
            let yesno = dialog::yes_no_dialog(self.question.clone());
            self.answer = Some(yesno);
            self
        }
    }
}

pub(crate) enum LoadDesignError {
    JsonError(serde_json::Error),
    ScadnanoImportError(ScadnanoImportError),
    IncompatibleVersion { current: String, required: String },
}

impl std::fmt::Display for LoadDesignError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::JsonError(e) => write!(f, "Json error: {e}"),
            Self::ScadnanoImportError(e) => {
                write!(
                    f,
                    "Scadnano file detected but the following error was encountered:
                {e:?}",
                )
            }
            Self::IncompatibleVersion { current, required } => {
                write!(
                    f,
                    "Your ENSnano version is too old to load this design.
                Your version: {current},
                Required version: {required}"
                )
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct SaveDesignError(String);

impl<E: std::error::Error> From<E> for SaveDesignError {
    fn from(e: E) -> Self {
        Self(format!("{e}"))
    }
}

impl SaveDesignError {
    pub(crate) fn cannot_open_default_dir() -> Self {
        Self(CANNOT_OPEN_DEFAULT_DIR.to_owned())
    }
}
