use std::collections::VecDeque;

use ensnano_utils::{keyboard_priority::PriorityRequest, ui_size::UiSize};
use winit::{dpi::LogicalSize, event::Modifiers};

use crate::{
    left_panel,
    state::{GuiAppState, TopBarStateFlags},
    top_bar,
};

/// Message sent to the gui component
pub struct GuiMessages<S: GuiAppState> {
    pub left_panel: VecDeque<left_panel::LeftPanelMessage<S>>,
    pub top_bar: VecDeque<top_bar::TopBarMessage<S>>,
    pub status_bar: VecDeque<StatusBarMessage<S>>,
    pub application_state: S,
    pub last_top_bar_state: TopBarStateFlags,
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
            .push_back(StatusBarMessage::Message(Some(message)));
    }

    pub fn push_progress(&mut self, progress_name: String, progress: f32) {
        self.status_bar
            .push_back(StatusBarMessage::Progress(Some((progress_name, progress))));
    }

    pub fn finish_progress(&mut self) {
        self.status_bar.push_back(StatusBarMessage::Progress(None));
    }

    pub fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.left_panel
            .push_back(left_panel::LeftPanelMessage::ModifiersChanged(modifiers));
    }

    pub fn new_ui_size(&mut self, ui_size: UiSize) {
        self.left_panel
            .push_back(left_panel::LeftPanelMessage::UiSizeChanged(ui_size));
        self.top_bar
            .push_back(top_bar::TopBarMessage::UiSizeChanged(ui_size));
        self.status_bar
            .push_back(StatusBarMessage::UiSizeChanged(ui_size));
    }

    pub fn push_show_tutorial(&mut self) {
        self.left_panel
            .push_back(left_panel::LeftPanelMessage::ShowTutorial);
    }

    pub fn show_help(&mut self) {
        self.left_panel
            .push_back(left_panel::LeftPanelMessage::ForceHelp);
    }

    pub fn push_application_state(&mut self, state: S, top_bar_state: TopBarStateFlags) {
        log::trace!("Old ptr {:p}, new ptr {:p}", state, self.application_state);
        self.application_state = state.clone();
        self.redraw |= top_bar_state != self.last_top_bar_state;
        self.last_top_bar_state = top_bar_state.clone();
        let must_update = self.application_state != state || self.redraw;
        if must_update {
            self.left_panel
                .push_back(left_panel::LeftPanelMessage::NewApplicationState(
                    state.clone(),
                ));
            self.top_bar
                .push_back(top_bar::TopBarMessage::NewApplicationState((
                    state.clone(),
                    top_bar_state,
                )));
            self.status_bar
                .push_back(StatusBarMessage::NewApplicationState(state));
        }
    }
}

/// List of Messages that can be send by the status bar.
#[derive(Clone, Debug)]
pub enum StatusBarMessage<S: GuiAppState> {
    ValueStrChanged(usize, String),
    ValueSet(usize, String),
    Progress(Option<(String, f32)>),
    NewApplicationState(S),
    UiSizeChanged(UiSize),
    TabPressed,
    Message(Option<String>),
    Resize(LogicalSize<f64>),
    SetKeyboardPriority(PriorityRequest),
}
