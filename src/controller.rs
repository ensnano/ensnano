/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Handles windows and dialog (Alert, and file pickers) interactions.

pub mod channel_reader;
mod download_intervals;
pub mod download_staples;
mod messages;
pub mod normal_state;
mod quit;
pub mod set_scaffold_sequence;

use super::{OverlayType, SplitMode, dialog};
use crate::MainStateView;
use dialog::{MustAckMessage, YesNoQuestion};
use ensnano_exports::ExportType;
use ensnano_iced::UiSize;
use ensnano_interactor::consts::CANNOT_OPEN_DEFAULT_DIR;
use normal_state::NormalState;
use quit::*;
use set_scaffold_sequence::*;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use ultraviolet::{Rotor3, Vec3};

pub struct Controller {
    /// The sate of the windows
    state: Box<dyn State + 'static>,
}

impl Controller {
    pub fn new() -> Self {
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
                log::error!("{:?}", e);
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

pub enum LoadDesignError {
    JsonError(serde_json::Error),
    ScadnanoImportError(ensnano_design::scadnano::ScadnanoImportError),
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
                {:?}",
                    e
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
pub struct SaveDesignError(String);

impl<E: std::error::Error> From<E> for SaveDesignError {
    fn from(e: E) -> Self {
        Self(format!("{}", e))
    }
}

impl SaveDesignError {
    pub fn cannot_open_default_dir() -> Self {
        Self(CANNOT_OPEN_DEFAULT_DIR.to_string())
    }
}
