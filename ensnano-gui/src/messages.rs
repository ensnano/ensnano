use std::collections::VecDeque;

use ensnano_utils::ui_size::UiSize;
use winit::event::Modifiers;

use crate::{
    left_panel,
    state::{GuiAppState, TopBarState},
    status_bar, top_bar,
};

/// Message sent to the gui component
pub struct GuiMessages<S: GuiAppState> {
    pub left_panel: VecDeque<left_panel::Message<S>>,
    pub top_bar: VecDeque<top_bar::Message<S>>,
    pub status_bar: VecDeque<status_bar::Message<S>>,
    pub application_state: S,
    pub last_top_bar_state: TopBarState,
    pub redraw: bool,
}

impl<S: GuiAppState> GuiMessages<S> {
    pub fn new() -> Self {
        Self {
            left_panel: VecDeque::new(),
            top_bar: VecDeque::new(),
            status_bar: VecDeque::new(),
            application_state: Default::default(),
            last_top_bar_state: Default::default(),
            redraw: false,
        }
    }

    pub fn push_message(&mut self, message: String) {
        self.status_bar
            .push_back(status_bar::Message::Message(Some(message)));
    }

    pub fn push_progress(&mut self, progress_name: String, progress: f32) {
        self.status_bar
            .push_back(status_bar::Message::Progress(Some((
                progress_name,
                progress,
            ))));
    }

    pub fn finish_progress(&mut self) {
        self.status_bar
            .push_back(status_bar::Message::Progress(None));
    }

    pub fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.left_panel
            .push_back(left_panel::Message::ModifiersChanged(modifiers));
    }

    pub fn new_ui_size(&mut self, ui_size: UiSize) {
        self.left_panel
            .push_back(left_panel::Message::UiSizeChanged(ui_size));
        self.top_bar
            .push_back(top_bar::Message::UiSizeChanged(ui_size));
        self.status_bar
            .push_back(status_bar::Message::UiSizeChanged(ui_size));
    }

    pub fn push_show_tutorial(&mut self) {
        self.left_panel.push_back(left_panel::Message::ShowTutorial);
    }

    pub fn show_help(&mut self) {
        self.left_panel.push_back(left_panel::Message::ForceHelp);
    }

    pub fn push_application_state(&mut self, state: S, top_bar_state: TopBarState) {
        log::trace!("Old ptr {:p}, new ptr {:p}", state, self.application_state);
        self.application_state = state.clone();
        self.redraw |= top_bar_state != self.last_top_bar_state;
        self.last_top_bar_state = top_bar_state.clone();
        let must_update = self.application_state != state || self.redraw;
        if must_update {
            self.left_panel
                .push_back(left_panel::Message::NewApplicationState(state.clone()));
            self.top_bar
                .push_back(top_bar::Message::NewApplicationState((
                    state.clone(),
                    top_bar_state,
                )));
            self.status_bar
                .push_back(status_bar::Message::NewApplicationState(state));
        }
    }
}
