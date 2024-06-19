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
use super::{AppState, Requests, UiSize};
use ensnano_interactor::operation::{Operation, ParameterField};
pub use ensnano_interactor::StrandBuildingStatus;
use iced::{Alignment, Element, Length};
use iced_graphics::text::Paragraph;
use iced_runtime::{Command, Program};
use iced_widget::*;
use iced_winit::winit;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use winit::dpi::LogicalSize;

const GOLD_ORANGE: iced::Color = iced::Color::from_rgb(0.84, 0.57, 0.20);

#[derive(Debug)]
enum StatusParameter {
    Value(text_input::State<Paragraph>),
    Choice(pick_list::State<Paragraph>),
}

impl StatusParameter {
    fn get_value(&mut self) -> &mut text_input::State<Paragraph> {
        match self {
            StatusParameter::Value(ref mut state) => state,
            _ => panic!("wrong status parameter variant"),
        }
    }

    fn get_choice(&mut self) -> &mut pick_list::State<Paragraph> {
        match self {
            StatusParameter::Choice(ref mut state) => state,
            _ => panic!("wrong status parameter variant"),
        }
    }

    fn value() -> Self {
        Self::Value(Default::default())
    }

    fn choice() -> Self {
        Self::Choice(Default::default())
    }

    fn has_keyboard_priority(&self) -> bool {
        match self {
            Self::Choice(_) => false,
            Self::Value(state) => state.is_focused(),
        }
    }

    fn focus(&mut self) -> bool {
        if let Self::Value(state) = self {
            state.focus();
            state.select_all();
            true
        } else {
            false
        }
    }

    fn unfocus(&mut self) {
        if let Self::Value(state) = self {
            state.unfocus()
        }
    }
}

pub struct StatusBar<R: Requests, S: AppState> {
    info_values: Vec<String>,
    operation: Option<OperationInput>,
    requests: Arc<Mutex<R>>,
    progress: Option<(String, f32)>,
    #[allow(dead_code)]
    slider_state: slider::State,
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
            info_values: Vec::new(),
            operation: None,
            requests,
            progress: None,
            slider_state: Default::default(),
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
        if let Some(new_operation) = self.app_state.get_curent_operation_state() {
            if let Some(operation) = self.operation.as_mut() {
                operation.update(new_operation);
            } else {
                self.operation = Some(OperationInput::new(new_operation));
            }
        } else {
            self.operation = None;
        }
    }

    fn view_progress(&self) -> Row<Message<State>, iced::Theme, iced::Renderer> {
        let progress = self.progress.as_ref().unwrap();
        row![text(format!("{}, {:.1}%", progress.0, progress.1 * 100.))
            .size(self.ui_size.main_text()),]
    }

    /* TODO
    fn view_overed_strand(&self) -> Element<Message<S>, iced_wgpu::Renderer> {
        let mut row = Row::new();
        if let Some(strand) = self.app_state.get_overed_strand() {
            row = row.push(Text::new(strand.info()).size(self.ui_size.status_font())
        }
        row.into()
    }*/

    pub fn has_keyboard_priority(&self) -> bool {
        self.operation
            .as_ref()
            .map(|op| op.has_keyboard_priority())
            .unwrap_or(false)
    }

    pub fn process_tab(&mut self) {
        let op = self.operation.as_mut().and_then(|op| op.process_tab());
        if !self.has_keyboard_priority() {
            log::info!("Updating operation");
            if let Some(op) = op {
                self.requests.lock().unwrap().update_current_operation(op)
            }
        }
    }
}

// List of Messages that can be send by the status bar.
#[derive(Clone, Debug)]
pub enum Message<S: AppState> {
    ValueStrChanged(usize, String),
    ValueSet(usize, String),
    Progress(Option<(String, f32)>),
    #[allow(dead_code)]
    SetShift(f32),
    NewApplicationState(S),
    UiSizeChanged(UiSize),
    TabPressed,
    Message(Option<String>),
    Resize(LogicalSize<f64>),
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
        } else if let Some(_) = self.app_state.get_strand_building_state() {
            self.operation = None;
            self.message = None;
        } else if let Some(_) = self.message {
            self.operation = None;
        } else if let Some(_) = self.operation {
            log::trace!("operation is some");
        } else {
            log::trace!("operation is none");
        };
        match message {
            Message::ValueStrChanged(n, s) => {
                if let Some(operation) = self.operation.as_mut() {
                    operation.update_input_str(n, s)
                }
            }
            Message::ValueSet(n, s) => {
                if let Some(operation) = self.operation.as_mut() {
                    if let Some(new_operation) = operation.update_value(n, s) {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_current_operation(new_operation);
                    }
                }
            }
            Message::Progress(progress) => self.progress = progress,
            Message::SetShift(f) => {
                self.info_values[2] = f.to_string();
                self.requests.lock().unwrap().update_hyperboloid_shift(f);
            }
            Message::NewApplicationState(state) => self.app_state = state,
            Message::UiSizeChanged(ui_size) => self.set_ui_size(ui_size),
            Message::TabPressed => self.process_tab(),
            Message::Message(message) => self.message = message,
            Message::Resize(size) => self.logical_size = size,
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Self::Theme, Self::Renderer> {
        let clipboard_text = format!(
            "Clipboard: {}",
            self.app_state.get_clipboard_content().to_string()
        );
        let pasting_text = match self.app_state.get_pasting_status() {
            ensnano_interactor::PastingStatus::Copy => "Pasting",
            ensnano_interactor::PastingStatus::None => "",
            ensnano_interactor::PastingStatus::Duplication => "Duplicating",
        }
        .to_string();

        let size = self.logical_size.clone();
        let mut content = if self.progress.is_some() {
            self.view_progress()
        } else if let Some(building_info) = self.app_state.get_strand_building_state() {
            row![text(building_info.to_info()).size(self.ui_size.main_text()),]
        } else if let Some(ref message) = &self.message {
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
            .style(crate::theme::GuiBackground)
            .width(size.width as f32)
            .height(Length::Fill)
            .into()
    }
}

pub struct CurentOpState {
    pub current_operation: Arc<dyn Operation>,
    pub operation_id: usize,
}

struct OperationInput {
    /// The values obatained with Operation::values
    values: Vec<String>,
    /// The String in the text inputs,
    values_str: Vec<String>,
    parameters: Vec<StatusParameter>,
    op_id: usize,
    operation: Arc<dyn Operation>,
    inputed_values: HashMap<usize, String>,
}

impl OperationInput {
    pub fn new(operation_state: CurentOpState) -> Self {
        let operation = operation_state.current_operation;
        let parameters = operation.parameters();
        let mut status_parameters = Vec::new();
        for p in parameters.iter() {
            match p.field {
                ParameterField::Choice(_) => status_parameters.push(StatusParameter::choice()),
                ParameterField::Value => status_parameters.push(StatusParameter::value()),
            }
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
            inputed_values: HashMap::new(),
        }
    }

    #[must_use = "Do not forget to apply the oppertaion"]
    pub fn process_tab(&mut self) -> Option<Arc<dyn Operation>> {
        let mut was_focus = false;
        let mut old_foccussed_idx: Option<usize> = None;
        for (i, p) in self.parameters.iter_mut().enumerate() {
            if was_focus {
                was_focus ^= p.focus()
            } else {
                if p.has_keyboard_priority() {
                    p.unfocus();
                    old_foccussed_idx = Some(i);
                    was_focus = true;
                }
            }
        }

        old_foccussed_idx.and_then(|i| {
            self.inputed_values.insert(i, self.values_str[i].clone());
            self.update_value(i, self.values_str[i].clone())
        })
    }

    pub fn update(&mut self, operation_state: CurentOpState) {
        let op_is_new = self.op_id != operation_state.operation_id;
        let operation = operation_state.current_operation;
        self.values = operation.values().clone();
        if op_is_new {
            self.values_str = self.values.clone();
            self.op_id = operation_state.operation_id;

            let mut status_parameters = Vec::new();
            for p in operation.parameters().iter() {
                match p.field {
                    ParameterField::Choice(_) => status_parameters.push(StatusParameter::choice()),
                    ParameterField::Value => status_parameters.push(StatusParameter::value()),
                }
            }
            self.parameters = status_parameters;
        } else {
            for (v_id, v) in self.values.iter().enumerate() {
                let foccused_parameter = self
                    .parameters
                    .get(v_id)
                    .map(|p| p.has_keyboard_priority())
                    .unwrap_or(false);
                if !foccused_parameter {
                    self.values_str[v_id] =
                        self.inputed_values.get(&v_id).cloned().unwrap_or(v.clone())
                }
            }
        }
        self.operation = operation;
    }

    fn view<S: AppState>(&self, ui_size: UiSize) -> Row<Message<S>, iced::Theme, iced::Renderer> {
        let mut row = Row::new();
        let op = self.operation.as_ref();
        row = row.push(Text::new(op.description()).size(ui_size.main_text()));
        let values = &self.values;
        let str_values = &self.values_str;
        let active_input = (0..values.len())
            .map(|i| self.active_input(i))
            .collect::<Vec<_>>();
        let mut need_validation = false;
        for i in 0..self.values.len() {
            if let Some(param) = op.parameters().get(i) {
                match param.field {
                    ParameterField::Value => {
                        let mut input = TextInput::new("", &format!("{0:.4}", str_values[i]))
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
                            .push(Text::new(param.name.clone()).size(ui_size.main_text()))
                            .push(input)
                    }
                    ParameterField::Choice(ref v) => {
                        row = row.spacing(20).push(
                            PickList::new(v.clone(), Some(values[i].clone()), move |s| {
                                Message::ValueSet(i, s)
                            })
                            .text_size(ui_size.main_text() - 4.0),
                        )
                    }
                }
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
            .map(|p| p.has_keyboard_priority())
            .unwrap_or(false)
    }

    fn update_input_str(&mut self, value_id: usize, new_str: String) {
        if let Some(s) = self.values_str.get_mut(value_id) {
            *s = new_str.clone()
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

    fn has_keyboard_priority(&self) -> bool {
        self.parameters.iter().any(|p| p.has_keyboard_priority())
    }
}

mod input_color {
    use iced::theme;
    use iced::{Background, Border, Color};
    use iced_widget::text_input::*;

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

impl ToString for ClipboardContent {
    fn to_string(&self) -> String {
        match self {
            Self::Empty => "Empty".into(),
            Self::Xovers(n) => format!("{n} xover(s)"),
            Self::Strands(n) => format!("{n} strand(s)"),
            Self::Grids(n) => format!("{n} grid(s)"),
            Self::Helices(n) => format!("{n} helice(s)"),
        }
    }
}
