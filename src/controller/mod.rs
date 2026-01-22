//! Handles windows and dialog (Alert, and file pickers) interactions.

mod download_intervals;
mod download_staples;
mod messages;
mod normal_state;
mod quit;
pub(crate) mod set_scaffold_sequence;

use self::normal_state::NormalState;
use crate::{
    MainStateView,
    dialog::{self, MustAckMessage, YesNoQuestion},
};
use std::borrow::Cow;

pub(crate) struct Controller {
    /// The sate of the windows
    state: Box<dyn AutomataState + 'static>,
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

trait AutomataState {
    /// Operate on [MainStateView] and return the new State of the automata
    fn make_progress(self: Box<Self>, main_state: &mut MainStateView) -> Box<dyn AutomataState>;
}

/// A dummy state that should never be constructed.
///
/// It is used as an argument to `std::mem::take`.
struct OhNo;

impl AutomataState for OhNo {
    fn make_progress(self: Box<Self>, _: &mut MainStateView) -> Box<dyn AutomataState> {
        panic!("Oh No !")
    }
}

/// Display a message that must be acknowledged by the user, and transition to a predetermined
/// state.
struct TransitionMessage {
    level: rfd::MessageLevel,
    content: Cow<'static, str>,
    ack: Option<MustAckMessage>,
    transition_to: Box<dyn AutomataState>,
}

impl TransitionMessage {
    fn new<S: Into<Cow<'static, str>>>(
        content: S,
        level: rfd::MessageLevel,
        transition_to: Box<dyn AutomataState + 'static>,
    ) -> Box<Self> {
        Box::new(Self {
            level,
            content: content.into(),
            ack: None,
            transition_to,
        })
    }
}

impl AutomataState for TransitionMessage {
    fn make_progress(
        mut self: Box<Self>,
        _: &mut MainStateView,
    ) -> Box<dyn AutomataState + 'static> {
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
    yes: Box<dyn AutomataState>,
    no: Box<dyn AutomataState>,
}

impl YesNo {
    fn new<S: Into<Cow<'static, str>>>(
        question: S,
        yes: Box<dyn AutomataState>,
        no: Box<dyn AutomataState>,
    ) -> Self {
        Self {
            question: question.into(),
            yes,
            no,
            answer: None,
        }
    }
}

impl AutomataState for YesNo {
    fn make_progress(mut self: Box<Self>, _: &mut MainStateView) -> Box<dyn AutomataState> {
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
