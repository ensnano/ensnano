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

use super::{AppState, Requests};
use crate::ensnano_iced::{helpers::*, theme::GuiBackground, ui_size::UiSize};
use crate::ensnano_interactor::{StrandBuildingStatus, operation::Operation};
use iced::{Alignment, Color, Element, Length};
use iced_graphics::text::Paragraph;
use iced_runtime::{Command, Program};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use winit::dpi::LogicalSize;

const GOLD_ORANGE: Color = Color::from_rgb(0.84, 0.57, 0.20);

// Very weird struct, doesn't seem to be used properly
#[derive(Debug)]
struct StatusParameter {
    text_input: text_input::State<Paragraph>,
}
impl StatusParameter {
    fn new() -> Self {
        Self {
            text_input: Default::default(),
        }
    }

    fn has_keyboard_priority(&self) -> bool {
        self.text_input.is_focused()
    }
}

pub struct StatusBar<R: Requests, S: AppState> {
    operation: Option<OperationInput>,
    requests: Arc<Mutex<R>>,
    progress: Option<(String, f32)>,
    app_state: S,
    ui_size: UiSize,
    message: Option<String>,
    logical_size: LogicalSize<f64>,
}

impl<R: Requests, State: AppState> StatusBar<R, State> {
    pub fn new(
        requests: Arc<Mutex<R>>,
        state: &State,
        logical_size: LogicalSize<f64>,
        ui_size: UiSize,
    ) -> Self {
        Self {
            operation: None,
            requests,
            progress: None,
            app_state: state.clone(),
            ui_size,
            message: None,
            logical_size,
        }
    }

    pub fn set_ui_size(&mut self, ui_size: UiSize) {
        self.ui_size = ui_size;
    }

    fn update_operation(&mut self) {
        if let Some(new_operation) = self.app_state.get_current_operation_state() {
            if let Some(operation) = self.operation.as_mut() {
                operation.update(new_operation);
            } else {
                self.operation = Some(OperationInput::new(new_operation));
            }
        } else {
            self.operation = None;
        }
    }

    fn view_progress(&self) -> Row<'_, Message<State>, iced::Theme, iced::Renderer> {
        let progress = self.progress.as_ref().unwrap();
        row![
            text(format!("{}, {:.1}%", progress.0, progress.1 * 100.))
                .size(self.ui_size.main_text()),
        ]
    }
}

// List of Messages that can be send by the status bar.
#[derive(Clone, Debug)]
pub enum Message<S: AppState> {
    ValueStrChanged(usize, String),
    ValueSet(usize, String),
    Progress(Option<(String, f32)>),
    NewApplicationState(S),
    UiSizeChanged(UiSize),
    TabPressed,
    Message(Option<String>),
    Resize(LogicalSize<f64>),
    SetKeyboardPriority(bool),
}

impl<R: Requests, S: AppState> Program for StatusBar<R, S> {
    type Message = Message<S>;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.update_operation();
        if self.progress.is_some() {
            self.operation = None;
            self.message = None;
        } else if self.app_state.get_strand_building_state().is_some() {
            self.operation = None;
            self.message = None;
        } else if self.message.is_some() {
            self.operation = None;
        } else if let Some(_) = self.operation {
            log::trace!("operation is some");
        } else {
            log::trace!("operation is none");
        };
        match message {
            Message::ValueStrChanged(n, s) => {
                if let Some(operation) = self.operation.as_mut() {
                    operation.update_input_str(n, s);
                }
            }
            Message::ValueSet(n, s) => {
                if let Some(operation) = self.operation.as_mut()
                    && let Some(new_operation) = operation.update_value(n, s)
                {
                    self.requests
                        .lock()
                        .unwrap()
                        .update_current_operation(new_operation);
                }
            }
            Message::Progress(progress) => self.progress = progress,
            Message::NewApplicationState(state) => self.app_state = state,
            Message::UiSizeChanged(ui_size) => self.set_ui_size(ui_size),
            //Message::TabPressed => self.process_tab(),
            Message::TabPressed => (),
            Message::Message(message) => self.message = message,
            Message::Resize(size) => self.logical_size = size,
            Message::SetKeyboardPriority(priority) => self
                .requests
                .lock()
                .unwrap()
                .set_keyboard_priority(priority),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        let clipboard_text = format!("Clipboard: {}", self.app_state.get_clipboard_content());
        let pasting_text = match self.app_state.get_pasting_status() {
            crate::ensnano_interactor::PastingStatus::Copy => "Pasting",
            crate::ensnano_interactor::PastingStatus::None => "",
            crate::ensnano_interactor::PastingStatus::Duplication => "Duplicating",
        }
        .to_string();

        let size = self.logical_size;
        let mut content = if self.progress.is_some() {
            self.view_progress()
        } else if let Some(building_info) = self.app_state.get_strand_building_state() {
            row![text(building_info.to_info()).size(self.ui_size.main_text()),]
        } else if let Some(message) = &self.message {
            row![text(message).size(self.ui_size.main_text()),]
        } else if let Some(operation) = &self.operation {
            log::trace!("operation is some");
            operation.view(self.ui_size)
        } else {
            log::trace!("operation is none");
            row![]
        };

        content = row![
            content,
            horizontal_space(), // To right align the clipboard text
            text(clipboard_text),
            Space::with_width(5),
        ]
        .align_items(Alignment::End);

        let pasting_status_row =
            row![horizontal_space(), text(pasting_text), Space::with_width(5),];

        let content = self::column![Space::new(Length::Fill, 3), content, pasting_status_row,];

        container(content)
            .style(GuiBackground)
            .width(size.width as f32)
            .height(Length::Fill)
            .into()
    }
}

pub struct CurrentOpState {
    pub current_operation: Arc<dyn Operation>,
    pub operation_id: usize,
}

struct OperationInput {
    /// The values obtained with Operation::values
    values: Vec<String>,
    /// The String in the text inputs,
    values_str: Vec<String>,
    parameters: Vec<StatusParameter>,
    op_id: usize,
    operation: Arc<dyn Operation>,
    inputted_values: HashMap<usize, String>,
}

impl OperationInput {
    pub fn new(operation_state: CurrentOpState) -> Self {
        let operation = operation_state.current_operation;
        let parameters = operation.parameters();
        let mut status_parameters = Vec::new();

        // This looks suspicious
        for _ in parameters.iter() {
            status_parameters.push(StatusParameter::new());
        }

        let values = operation.values().clone();
        let values_str = values.clone();
        let op_id = operation_state.operation_id;
        Self {
            parameters: status_parameters,
            op_id,
            values,
            values_str,
            operation,
            inputted_values: HashMap::new(),
        }
    }

    pub fn update(&mut self, operation_state: CurrentOpState) {
        let op_is_new = self.op_id != operation_state.operation_id;
        let operation = operation_state.current_operation;
        self.values = operation.values().clone();
        if op_is_new {
            self.values_str = self.values.clone();
            self.op_id = operation_state.operation_id;

            let mut status_parameters = Vec::new();

            // This looks suspicious
            for _ in operation.parameters() {
                status_parameters.push(StatusParameter::new());
            }

            self.parameters = status_parameters;
        } else {
            for (v_id, v) in self.values.iter().enumerate() {
                if !self.active_input(v_id) {
                    self.values_str[v_id] = self
                        .inputted_values
                        .get(&v_id)
                        .cloned()
                        .unwrap_or(v.clone());
                }
            }
        }
        self.operation = operation;
    }

    fn view<S: AppState>(
        &self,
        ui_size: UiSize,
    ) -> Row<'_, Message<S>, iced::Theme, iced::Renderer> {
        let mut row = Row::new();
        let op = self.operation.as_ref();
        row = row.push(text(op.description()).size(ui_size.main_text()));
        let values = &self.values;
        let str_values = &self.values_str;
        let active_input = (0..values.len())
            .map(|i| self.active_input(i))
            .collect::<Vec<_>>();
        let mut need_validation = false;
        for i in 0..self.values.len() {
            if let Some(param) = op.parameters().get(i) {
                let mut input = text_input("", &format!("{0:.4}", str_values[i]))
                    .on_input(move |s| Message::ValueStrChanged(i, s))
                    .size(ui_size.main_text())
                    .width(40)
                    .on_submit(Message::ValueSet(i, str_values[i].clone()));
                if active_input.get(i) == Some(&true) {
                    use input_color::InputValueState;
                    let state = if values.get(i) == str_values.get(i) {
                        InputValueState::Normal
                    } else if op.with_new_value(i, str_values[i].clone()).is_some() {
                        need_validation = true;
                        InputValueState::BeingTyped
                    } else {
                        InputValueState::Invalid
                    };
                    input = input.style(state);
                }
                row = row
                    .spacing(20)
                    .push(text(param).size(ui_size.main_text()))
                    .push(
                        keyboard_priority(input)
                            .on_priority(Message::SetKeyboardPriority(true))
                            .on_unpriority(Message::SetKeyboardPriority(false)),
                    );
            }
        }
        if need_validation {
            row = row.push(Text::new("(Press enter to validate change)").size(ui_size.main_text()));
        }
        row
    }

    fn active_input(&self, i: usize) -> bool {
        self.parameters
            .get(i)
            .is_some_and(|p| p.has_keyboard_priority())
    }

    fn update_input_str(&mut self, value_id: usize, new_str: String) {
        if let Some(s) = self.values_str.get_mut(value_id) {
            *s = new_str.clone();
        } else {
            log::error!(
                "Changing str of value_id {} but self has {} values",
                value_id,
                self.values_str.len()
            );
        }
    }

    fn update_value(&mut self, value_id: usize, values_str: String) -> Option<Arc<dyn Operation>> {
        if let Some(op) = self.operation.as_ref().with_new_value(value_id, values_str) {
            self.operation = op.clone();
            Some(op)
        } else {
            None
        }
    }
}

mod input_color {
    // TODO: Move this in ensnano_iced.
    use iced::{Background, Border, Color, theme, widget::text_input::*};

    pub enum InputValueState {
        Normal,
        BeingTyped,
        Invalid,
    }

    impl StyleSheet for InputValueState {
        type Style = ();
        fn active(&self, _style: &Self::Style) -> Appearance {
            Appearance {
                background: Background::Color(Color::WHITE),
                border: Border {
                    color: Color::from_rgb(0.7, 0.7, 0.7),
                    width: 1.0,
                    radius: 5.0.into(),
                },
                icon_color: Default::default(), // TODO:Choose an appropriate value for this field.
            }
        }

        fn focused(&self, style: &Self::Style) -> Appearance {
            Appearance {
                border: Border {
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..self.active(style).border
                },
                ..self.active(style)
            }
        }

        fn placeholder_color(&self, _style: &Self::Style) -> Color {
            Color::from_rgb(0.7, 0.7, 0.7)
        }

        fn value_color(&self, _style: &Self::Style) -> Color {
            match self {
                Self::Normal => Color::from_rgb(0.3, 0.3, 0.3),
                Self::Invalid => Color::from_rgb(1., 0.3, 0.3),
                Self::BeingTyped => super::GOLD_ORANGE,
            }
        }

        fn disabled_color(&self, _style: &Self::Style) -> Color {
            Color::from_rgb(0.4, 0.4, 0.4) // TODO: Choose an appropriate value for this field
        }

        fn selection_color(&self, _style: &Self::Style) -> Color {
            Color::from_rgb(0.8, 0.8, 1.0)
        }

        fn disabled(&self, style: &Self::Style) -> Appearance {
            Appearance {
                // TODO: Choose an appropriate value for this field
                border: Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    ..self.active(style).border
                },
                ..self.active(style)
            }
        }
    }

    impl From<InputValueState> for theme::TextInput {
        fn from(_value: InputValueState) -> Self {
            Default::default()
            // Maybe this is not correct. I wrote this to make it compile.
        }
    }
}

trait ToInfo {
    fn to_info(&self) -> String;
}

impl ToInfo for StrandBuildingStatus {
    fn to_info(&self) -> String {
        format!(
            "Current domain length: {} nt ({:.2} nm). 5': {}, 3': {}",
            self.nt_length, self.nm_length, self.prime5.position, self.prime3.position
        )
    }
}

pub enum ClipboardContent {
    Empty,
    Xovers(usize),
    Strands(usize),
    Grids(usize),
    Helices(usize),
}

impl std::fmt::Display for ClipboardContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Xovers(n) => write!(f, "{n} {}", if *n < 2 { "xover" } else { "xovers" }),
            Self::Strands(n) => write!(f, "{n} {}", if *n < 2 { "strand" } else { "strands" }),
            Self::Grids(n) => write!(f, "{n} {}", if *n < 2 { "grid" } else { "grids" }),
            Self::Helices(n) => write!(f, "{n} {}", if *n < 2 { "helix" } else { "helices" }),
        }
    }
}
