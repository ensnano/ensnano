use super::Requests;
use crate::mediator::{Operation, ParameterField, Selection};
use iced::{container, Background, Container, Length};
use iced_native::{pick_list, text_input, Color, PickList, TextInput, Checkbox};
use iced_winit::{Column, Command, Element, Program, Row, Space, Text};
use std::sync::{Arc, Mutex};
use std::str::FromStr;

const STATUS_FONT_SIZE: u16 = 14;

#[derive(Debug)]
enum StatusParameter {
    Value(text_input::State),
    Choice(pick_list::State<String>),
    CheckBox(bool),
}

impl StatusParameter {
    fn get_value(&mut self) -> &mut text_input::State {
        match self {
            StatusParameter::Value(ref mut state) => state,
            _ => panic!("wrong status parameter variant"),
        }
    }

    fn get_choice(&mut self) -> &mut pick_list::State<String> {
        match self {
            StatusParameter::Choice(ref mut state) => state,
            _ => panic!("wrong status parameter variant"),
        }
    }

    fn is_checked(&self) -> bool {
        match self {
            StatusParameter::CheckBox(b) => *b,
            _ => panic!("wrong status parameter variant"),
        }
    }

    fn value() -> Self {
        Self::Value(Default::default())
    }

    fn choice() -> Self {
        Self::Choice(Default::default())
    }

    fn checkbox() -> Self {
        Self::CheckBox(false)
    }
}

pub struct StatusBar {
    parameters: Vec<StatusParameter>,
    values: Vec<String>,
    operation: Option<Arc<dyn Operation>>,
    requests: Arc<Mutex<Requests>>,
    selection: Selection,
}

impl StatusBar {
    pub fn new(requests: Arc<Mutex<Requests>>) -> Self {
        Self {
            parameters: Vec::new(),
            values: Vec::new(),
            operation: None,
            requests,
            selection: Selection::Nothing,
        }
    }

    pub fn update_op(&mut self, operation: Arc<dyn Operation>) {
        let parameters = operation.parameters();
        let mut new_param = Vec::new();
        for p in parameters.iter() {
            match p.field {
                ParameterField::Choice(_) => new_param.push(StatusParameter::choice()),
                ParameterField::Value => new_param.push(StatusParameter::value()),
            }
        }
        self.values = operation.values().clone();
        self.parameters = new_param;
    }

    fn view_op(&mut self) -> Element<Message, iced_wgpu::Renderer> {
        let mut row = Row::new();
        let op = self.operation.as_ref().unwrap(); // the function view op is only called when op is some.
        row = row.push(Text::new(op.description()).size(STATUS_FONT_SIZE));
        let values = &self.values;
        for (i, p) in self.parameters.iter_mut().enumerate() {
            let param = &op.parameters()[i];
            match param.field {
                ParameterField::Value => {
                    row = row
                        .spacing(20)
                        .push(Text::new(param.name.clone()).size(STATUS_FONT_SIZE))
                        .push(
                            TextInput::new(
                                p.get_value(),
                                "",
                                &format!("{0:.4}", values[i]),
                                move |s| Message::ValueChanged(i, s),
                            )
                            .size(STATUS_FONT_SIZE)
                            .width(Length::Units(40)),
                        )
                }
                ParameterField::Choice(ref v) => {
                    row = row.spacing(20).push(
                        PickList::new(
                            p.get_choice(),
                            v.clone(),
                            Some(values[i].clone()),
                            move |s| Message::ValueChanged(i, s),
                        )
                        .text_size(STATUS_FONT_SIZE - 4),
                    )
                }
            }
        }

        let column = Column::new()
            .push(Space::new(Length::Fill, Length::Units(3)))
            .push(row);
        Container::new(column)
            .style(StatusBarStyle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_selection(&mut self) -> Element<Message, iced_wgpu::Renderer> {
        let mut row = Row::new();
        let selection = &self.selection;
        row = row.push(Text::new(selection.info()).size(STATUS_FONT_SIZE));

        match selection {
            Selection::Grid(_, _) => {
                row = row.push(Checkbox::new(bool::from_str(&self.values[0]).unwrap(), "Persistent phantoms", |b| Message::SelectionValueChanged(0, bool_to_string(b))).size(STATUS_FONT_SIZE).text_size(STATUS_FONT_SIZE));
            }
            _ => ()
        }

        let column = Column::new()
            .push(Space::new(Length::Fill, Length::Units(3)))
            .push(row);
        Container::new(column)
            .style(StatusBarStyle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Operation(Arc<dyn Operation>),
    Selection(Selection, Vec<String>),
    ValueChanged(usize, String),
    SelectionValueChanged(usize, String),
    ClearOp,
}

impl Program for StatusBar {
    type Message = Message;
    type Renderer = iced_wgpu::Renderer;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Operation(ref op) => {
                self.operation = Some(op.clone());
                self.update_op(op.clone());
            }
            Message::ValueChanged(n, s) => {
                self.values[n] = s.clone();
                let new_op = self
                    .operation
                    .as_ref()
                    .and_then(|op| op.with_new_value(n, s));
                if let Some(ref op) = new_op {
                    self.operation = Some(op.clone());
                }
                self.requests.lock().unwrap().operation_update = new_op;
            }
            Message::SelectionValueChanged(n, s) => {
                self.values[n] = s.clone();
                self.requests.lock().unwrap().toggle_persistent_helices = bool::from_str(&s).ok();
            }
            Message::Selection(s, v) => {
                self.operation = None;
                self.selection = s;
                self.values = v;
            }
            Message::ClearOp => self.operation = None,
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message, iced_wgpu::Renderer> {
        if self.operation.is_some() {
            self.view_op()
        } else {
            self.view_selection()
        }
    }

}

impl Selection {
    fn parameters(&self) -> Vec<StatusParameter> {
        match self {
            Selection::Grid(_, _) => vec![StatusParameter::checkbox()],
            _ => Vec::new()
        }
    }
}

struct StatusBarStyle;
impl container::StyleSheet for StatusBarStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(BACKGROUND)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub const BACKGROUND: Color = Color::from_rgb(
    0x36 as f32 / 255.0,
    0x39 as f32 / 255.0,
    0x3F as f32 / 255.0,
);


fn bool_to_string(b: bool) -> String {
    if b {
        String::from("true")
    } else {
        String::from("false")
    }
}
