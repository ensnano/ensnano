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
use super::super::DesignReader;
use super::*;
use crate::helpers::*;
use ensnano_design::{grid::GridId, BezierVertexId};
use ensnano_interactor::{Selection, SimulationState};
use iced_native::widget;
use iced_native::widget::helpers::*;

mod value_constructor;
use value_constructor::{BezierVertexBuilder, Builder, GridBuilder};
pub use value_constructor::{BuilderMessage, InstanciatedValue, ValueKind};

use ultraviolet::{Rotor3, Vec2, Vec3};
pub enum ValueRequest {
    HelixGridPosition {
        grid_id: GridId,
        position: Vec3,
    },
    GridOrientation {
        grid_id: GridId,
        orientation: Rotor3,
    },
    GridNbTurn {
        grid_id: GridId,
        nb_turn: f32,
    },
    BezierVertexPosition {
        vertex_id: BezierVertexId,
        position: Vec2,
    },
}

impl ValueRequest {
    fn from_value_and_selection(selection: &Selection, value: InstanciatedValue) -> Option<Self> {
        match value {
            InstanciatedValue::HelixGridPosition(v) => {
                if let Selection::Grid(_, g_id) = selection {
                    Some(Self::HelixGridPosition {
                        grid_id: *g_id,
                        position: v,
                    })
                } else {
                    log::error!("Recieved value {:?} with selection {:?}", value, selection);
                    None
                }
            }
            InstanciatedValue::GridOrientation(orientation) => {
                if let Selection::Grid(_, g_id) = selection {
                    Some(Self::GridOrientation {
                        grid_id: *g_id,
                        orientation,
                    })
                } else {
                    log::error!("Recieved value {:?} with selection {:?}", value, selection);
                    None
                }
            }
            InstanciatedValue::GridNbTurn(nb_turn) => {
                if let Selection::Grid(_, g_id) = selection {
                    Some(Self::GridNbTurn {
                        grid_id: *g_id,
                        nb_turn,
                    })
                } else {
                    log::error!("Recieved value {:?} with selection {:?}", value, selection);
                    None
                }
            }
            InstanciatedValue::BezierVertexPosition(pos) => {
                if let Selection::BezierVertex(vertex_id) = selection {
                    Some(Self::BezierVertexPosition {
                        vertex_id: *vertex_id,
                        position: pos,
                    })
                } else {
                    log::error!("Recieved value {:?} with selection {:?}", value, selection);
                    None
                }
            }
        }
    }

    pub(super) fn make_request(&self, request: Arc<Mutex<dyn Requests>>) {
        match self {
            Self::HelixGridPosition { grid_id, position } => request
                .lock()
                .unwrap()
                .set_grid_position(*grid_id, *position),
            Self::GridOrientation {
                grid_id,
                orientation,
            } => request
                .lock()
                .unwrap()
                .set_grid_orientation(*grid_id, *orientation),
            Self::GridNbTurn { grid_id, nb_turn } => {
                request.lock().unwrap().set_nb_turn(*grid_id, *nb_turn)
            }
            Self::BezierVertexPosition {
                vertex_id,
                position,
            } => request
                .lock()
                .unwrap()
                .set_position_of_bezier_vertex(*vertex_id, *position),
        }
    }
}

struct InstantiatedBuilder<S: AppState> {
    selection: Selection,
    builder: Box<dyn Builder<S>>,
}

impl<S: AppState> InstantiatedBuilder<S> {
    /// If a builder can be made from the selection, update the builder and return true. Otherwise,
    /// return false.
    fn update(&mut self, selection: &Selection, reader: &dyn DesignReader, app_state: &S) -> bool {
        if *selection != self.selection || app_state.is_transitory() {
            self.selection = *selection;
            if let Some(builder) = Self::new_builder(selection, reader) {
                self.builder = builder;
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    fn new(selection: &Selection, reader: &dyn DesignReader) -> Option<Self> {
        Self::new_builder(selection, reader).map(|builder| Self {
            builder,
            selection: *selection,
        })
    }

    fn new_builder(
        selection: &Selection,
        reader: &dyn DesignReader,
    ) -> Option<Box<dyn Builder<S>>> {
        match selection {
            Selection::Grid(_, g_id) => {
                if let Some((position, orientation)) =
                    reader.get_grid_position_and_orientation(*g_id)
                {
                    Some(Box::new(GridBuilder::new(position, orientation)))
                } else {
                    None
                }
            }
            Selection::BezierVertex(vertex_id) => {
                if let Some(position) = reader.get_bezier_vertex_position(*vertex_id) {
                    Some(Box::new(BezierVertexBuilder::new(position)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub(super) struct ContextualPanel<S: AppState> {
    //scroll: widget::scrollable::State,
    width: u32,
    pub force_help: bool,
    pub show_tutorial: bool,
    add_strand_menu: AddStrandMenu,
    strand_name_state: widget::text_input::State,
    builder: Option<InstantiatedBuilder<S>>,
    insertion_length_state: InsertionLengthState,
}

impl<S: AppState> ContextualPanel<S> {
    pub fn new(width: u32) -> Self {
        Self {
            //scroll: Default::default(),
            width,
            force_help: false,
            show_tutorial: false,
            add_strand_menu: Default::default(),
            strand_name_state: Default::default(),
            builder: None,
            insertion_length_state: Default::default(),
        }
    }

    pub fn new_width(&mut self, width: u32) {
        self.width = width;
    }

    fn update_builder(
        &mut self,
        selection: Option<&Selection>,
        reader: &dyn DesignReader,
        app_state: &S,
    ) {
        if let Some(s) = selection {
            if let Some(builder) = &mut self.builder {
                if !builder.update(s, reader, app_state) {
                    self.builder = None;
                }
            } else {
                self.builder = InstantiatedBuilder::new(s, reader)
            }
        } else {
            self.builder = None;
        }
    }

    pub fn view(&mut self, ui_size: UiSize, app_state: &S) -> iced::Element<Message<S>> {
        let mut content = Column::new().max_width((self.width - 2) as u16);
        let selection = app_state
            .get_selection()
            .get(0)
            .unwrap_or(&Selection::Nothing);
        let nb_selected = app_state
            .get_selection()
            .iter()
            .filter(|s| !matches!(s, Selection::Nothing))
            .count();

        self.update_builder(
            Some(selection).filter(|_| nb_selected == 1),
            app_state.get_reader().as_ref(),
            app_state,
        );

        let xover_len = app_state
            .get_strand_building_state()
            .map(|b| b.dragged_nucl)
            .and_then(|nucl| {
                log::info!("dragged_nucl: {:?}", nucl);
                app_state.get_reader().get_id_of_xover_involving_nucl(nucl)
            })
            .and_then(|id| app_state.get_reader().xover_length(id));

        self.insertion_length_state.update_selection(selection);
        let info_values = values_of_selection(selection, app_state.get_reader().as_ref());
        if self.show_tutorial {
            content = content.push(
                text("Tutorials")
                    .size(ui_size.head_text())
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            );
            content = content.push(text("ENSnano website"));
            content = content.push(link_row("http://ens-lyon.fr/ensnano", ui_size));
        } else if self.force_help && xover_len.is_none() {
            content = content.push(turn_into_help_column(ui_size))
        } else if app_state.get_action_mode().is_build() {
            let strand_menu = self.add_strand_menu.view(ui_size, self.width as u16);
            content = content.push(strand_menu);
        } else if *selection == Selection::Nothing && xover_len.is_none() {
            content = content.push(turn_into_help_column(ui_size))
        } else if nb_selected > 1 {
            content = content
                .push(iced_native::row![
                    horizontal_space(Length::FillPortion(1)),
                    iced_native::column![text_btn("Help", ui_size).on_press(Message::ForceHelp),]
                        .width(Length::FillPortion(1)),
                    horizontal_space(Length::FillPortion(1)),
                ])
                .width(Length::Fill)
                .align_items(iced::Alignment::Center);
            content = content.push(Text::new(format!("{} objects selected", nb_selected)));
        } else {
            content = content.push(
                iced_native::row![
                    horizontal_space(Length::FillPortion(1)),
                    iced_native::column![text_btn("Help", ui_size).on_press(Message::ForceHelp),]
                        .width(Length::FillPortion(1)),
                    horizontal_space(Length::FillPortion(1)),
                ]
                .width(Length::Fill)
                .align_items(iced::Alignment::Center),
            );

            if !matches!(selection, Selection::Nothing) {
                content = content.push(text(selection.info()).size(ui_size.main_text()));
            }

            match selection {
                Selection::Grid(_, g_id) => {
                    let twisting = match app_state.get_simulation_state() {
                        SimulationState::Twisting { grid_id } if *g_id == grid_id => {
                            TwistStatus::Twisting
                        }
                        SimulationState::None => TwistStatus::CanTwist,
                        _ => TwistStatus::CannotTwist,
                    };
                    content =
                        content.push(add_grid_content(info_values.as_slice(), ui_size, twisting))
                }
                Selection::Strand(_, _) => {
                    content = content.push(add_strand_content(info_values.as_slice(), ui_size))
                }
                Selection::Nucleotide(_, _) => {
                    let anchor = info_values[0].clone();
                    content = content.push(text(format!("Anchor {}", anchor)));
                }
                Selection::Xover(_, _) => {
                    if xover_len.is_none() {
                        if let Some(info) = info_values.get(0) {
                            content = content.push(text(info));
                        }
                        if let Some(info) = info_values.get(1) {
                            content = content.push(text(info));
                        }
                    }
                }
                _ => (),
            }
            if let Some(builder) = &mut self.builder {
                content = content.push(builder.builder.view(ui_size, selection, app_state))
            }
        }

        if let Some(info_values) = xover_len.map(|v| fmt_xover_len(Some(v))) {
            if let Some(info) = info_values.get(0) {
                content = content.push(text(info));
            }
            if let Some(info) = info_values.get(1) {
                content = content.push(text(info));
            }
        }

        if let Some(len) = app_state.get_reader().get_insertion_length(selection) {
            let real_len_string = len.to_string();
            let text_input_content = self
                .insertion_length_state
                .input_str
                .as_ref()
                .unwrap_or(&real_len_string);
            content = content.push(iced_native::row![
                text("Loopout"),
                text_input("", text_input_content)
                    .on_input(Message::InsertionLengthInput)
                    .on_submit(Message::InsertionLengthSubmitted),
            ]);
        }

        scrollable(content).into()
    }

    pub fn selection_value_changed<R: Requests>(
        &mut self,
        _n: usize,
        s: String,
        requests: Arc<Mutex<R>>,
    ) {
        if let Ok(g_id) = s.parse() {
            requests
                .lock()
                .unwrap()
                .toggle_helices_persistance_of_grid(g_id);
        }
    }

    pub fn set_small_sphere<R: Requests>(&mut self, b: bool, requests: Arc<Mutex<R>>) {
        requests.lock().unwrap().set_small_sphere(b);
    }

    pub fn scaffold_id_set<R: Requests>(&mut self, n: usize, b: bool, requests: Arc<Mutex<R>>) {
        if b {
            requests.lock().unwrap().set_scaffold_id(Some(n))
        } else {
            requests.lock().unwrap().set_scaffold_id(None)
        }
    }

    pub fn state_updated(&mut self) {
        self.force_help = false;
        self.show_tutorial = false;
    }

    pub(super) fn update_pos_str(&mut self, position_str: String) -> (isize, usize) {
        self.add_strand_menu.update_pos_str(position_str)
    }

    pub(super) fn update_length_str(&mut self, length_str: String) -> (isize, usize) {
        self.add_strand_menu.update_length_str(length_str)
    }

    pub fn has_keyboard_priority(&self) -> bool {
        self.add_strand_menu.has_keyboard_priority()
            || self.strand_name_state.is_focused()
            || self.builder_has_keyboard_priority()
            || self.insertion_length_state.has_keyboard_priority()
    }

    fn builder_has_keyboard_priority(&self) -> bool {
        self.builder
            .as_ref()
            .map(|b| b.builder.has_keyboard_priority())
            .unwrap_or(false)
    }

    pub fn get_build_helix_mode(&self) -> ActionMode {
        self.add_strand_menu.get_build_helix_mode()
    }

    pub fn get_new_strand_parameters(&self) -> Option<(isize, usize)> {
        self.add_strand_menu.get_new_strand_parameters()
    }

    pub fn set_show_strand(&mut self, show: bool) {
        self.add_strand_menu.set_show_strand(show)
    }

    pub fn update_builder_value(&mut self, kind: ValueKind, n: usize, value: String) {
        if let Some(b) = &mut self.builder {
            b.builder.update_str_value(kind, n, value)
        } else {
            log::error!("Cannot update value: No instanciated builder");
        }
    }

    pub fn submit_value(&mut self, kind: ValueKind) -> Option<ValueRequest> {
        if let Some(b) = &mut self.builder {
            if let Some(value) = b.builder.submit_value(kind) {
                ValueRequest::from_value_and_selection(&b.selection, value)
            } else {
                None
            }
        } else {
            log::error!("Cannot submit value: No instanciated builder");
            None
        }
    }

    pub fn request_from_value(&mut self, value: InstanciatedValue) -> Option<ValueRequest> {
        if let Some(b) = &mut self.builder {
            ValueRequest::from_value_and_selection(&b.selection, value)
        } else {
            log::error!("Cannot submit value: No instanciated builder");
            None
        }
    }

    pub fn update_insertion_length_input(&mut self, input: String) {
        self.insertion_length_state.input_str = Some(input);
    }

    pub fn get_insertion_request(&self) -> Option<InsertionRequest> {
        let length = self
            .insertion_length_state
            .input_str
            .as_ref()
            .and_then(|s| s.parse::<usize>().ok())?;
        Some(InsertionRequest {
            selection: self.insertion_length_state.selection,
            length,
        })
    }
}

enum TwistStatus {
    CanTwist,
    CannotTwist,
    Twisting,
}

fn add_grid_content<S: AppState, I: std::ops::Deref<Target = str>>(
    info_values: &[I],
    ui_size: UiSize,
    twisting: TwistStatus,
) -> iced::Element<Message<S>> {
    iced_native::column![
        // twist_button
        match twisting {
            TwistStatus::Twisting => text_btn("Stop", ui_size).on_press(Message::StopSimulation),
            TwistStatus::CanTwist => text_btn("Twist", ui_size).on_press(Message::StartTwist),
            TwistStatus::CannotTwist => text_btn("Twist", ui_size),
        },
        checkbox(
            "Persistent phantoms",
            info_values[0].parse::<bool>().unwrap(),
            |b| Message::SelectionValueChanged(0, bool_to_string(b)),
        )
        .size(ui_size.checkbox())
        .text_size(ui_size.main_text()),
        checkbox("No sphere", info_values[1].parse::<bool>().unwrap(), |b| {
            Message::SetSmallSpheres(b)
        })
        .size(ui_size.checkbox())
        .text_size(ui_size.main_text()),
    ]
    .into()
}

fn add_strand_content<S: AppState, I: std::ops::Deref<Target = str>>(
    info_values: &[I],
    ui_size: UiSize,
) -> iced::Element<Message<S>> {
    let s_id = info_values[2].parse::<usize>().unwrap();
    iced_native::column![
        iced_native::row![
            text("Name").size(ui_size.main_text()),
            text_input("Name", &info_values[4])
                .on_input(move |new_name| { Message::StrandNameChanged(s_id, new_name) })
                .size(ui_size.main_text()),
        ],
        text(format!("length {}", info_values[0].deref())).size(ui_size.main_text()),
        checkbox("Scaffold", info_values[1].parse().unwrap(), move |b| {
            Message::ScaffoldIdSet(s_id, b)
        }),
        text(info_values[3].deref()).size(ui_size.main_text()),
    ]
    .into()
}

fn bool_to_string(b: bool) -> String {
    if b {
        String::from("true")
    } else {
        String::from("false")
    }
}

fn add_help_to_column<'a, S: AppState>(
    help_title: impl ToString,
    help: Vec<(String, String)>,
    ui_size: UiSize,
) -> iced::Element<'a, Message<S>> {
    let mut content = widget::Column::new();
    content = content.push(text(help_title).size(ui_size.intermediate_text()));
    for (l, r) in help {
        if l.is_empty() {
            content = content.push(horizontal_space(10));
        } else if r.is_empty() {
            content = content.push(
                text(l)
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            );
        } else {
            content = content.push(iced_native::row![
                text(l)
                    .width(Length::FillPortion(5))
                    .horizontal_alignment(iced::alignment::Horizontal::Right),
                horizontal_space(Length::FillPortion(1)),
                text(r).width(Length::FillPortion(5)),
            ]);
        }
    }
    content.into()
}

fn turn_into_help_column<'a, S: AppState>(ui_size: UiSize) -> iced::Element<'a, Message<S>> {
    iced_native::column![
        text("Help")
            .size(ui_size.head_text())
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center),
        add_help_to_column("3D view", view_3d_help(), ui_size),
        horizontal_space(15),
        add_help_to_column("2D/3D view", view_2d_3d_help(), ui_size),
        horizontal_space(15),
        add_help_to_column("2D view", view_2d_help(), ui_size),
    ]
    .into()
}

fn view_3d_help() -> Vec<(String, String)> {
    vec![
        (
            format!("{}", LCLICK),
            "Select\nnt → strand → helix".to_owned(),
        ),
        (
            format!("{}+{}", SHIFT, LCLICK),
            "Multiple select".to_owned(),
        ),
        (String::new(), String::new()),
        (
            format!("2x{}", LCLICK),
            "Center selection in 2D view".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{} Drag", MCLICK), "Translate camera".to_owned()),
        (
            format!("{}+{} Drag", ALT, LCLICK),
            "Translate camera".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{}", RCLICK), "Set pivot".to_owned()),
        (
            format!("{} Drag", RCLICK),
            "Rotate camera around pivot (preserve the XZ plane)".to_owned(),
        ),
        (
            format!("{}+{} Drag", CTRL, RCLICK),
            "Rotate camera freely around pivot".to_owned(),
        ),
        (
            format!("{}+{} Drag", ALT, RCLICK),
            "Rotate camera around pivot (preserve the current horizon plane)".to_owned(),
        ),
        (
            format!("{}+{} Drag", SHIFT, RCLICK),
            "Tilt camera".to_owned(),
        ),
        (
            "⎵ (with cursor over the 3D scene)".to_owned(),
            "Export the current view in png format".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{} Drag", LCLICK), "Edit strand".to_owned()),
        (
            format!("long {} Drag", LCLICK),
            "Make crossover (drop on nt)".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("When in 3D {} mode", MOVECHAR), String::new()),
        (
            format!("{} on handle", LCLICK),
            "Move selected object".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("When in 3D {} mode", ROTCHAR), String::new()),
        (
            format!("{} on handle", LCLICK),
            "Rotate selected object".to_owned(),
        ),
    ]
}

fn view_2d_3d_help() -> Vec<(String, String)> {
    vec![
        (format!("{} + C", CTRL), "Copy selection".to_owned()),
        (format!("{} + V", CTRL), "Paste".to_owned()),
        (format!("{} + J", CTRL), "Paste & repeat".to_owned()),
        (String::new(), String::new()),
        (
            format!("{} or {}", SUPPRCHAR, BACKSPACECHAR),
            "Delete selected strands".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{} + S", CTRL), "Save design".to_owned()),
        (format!("{} + O", CTRL), "Open design".to_owned()),
        (format!("{} + Z", CTRL), "Undo".to_owned()),
        (format!("{} + R", CTRL), "Redo".to_owned()),
        (String::new(), String::new()),
        ("Selection mode shortcuts".to_owned(), "".to_owned()),
        ("'N' key".to_owned(), format!("Nucleotide, ({})", NUCLCHAR)),
        ("'S' key".to_owned(), format!("Strand ({})", STRANDCHAR)),
        ("'H' key".to_owned(), format!("Helix ({})", HELIXCHAR)),
        (String::new(), String::new()),
        ("Action mode shortcuts".to_owned(), "".to_owned()),
        ("ESC".to_owned(), format!("Select ({})", SELECTCHAR)),
        ("'T' key".to_owned(), format!("Translation ({})", MOVECHAR)),
        ("'R' key".to_owned(), format!("Rotation ({})", ROTCHAR)),
    ]
}

fn view_2d_help() -> Vec<(String, String)> {
    vec![
        (format!("{} Drag", MCLICK), "Translate camera".to_owned()),
        (
            format!("{} + {} Drag", ALT, LCLICK),
            "Translate camera".to_owned(),
        ),
        (
            format!("{} + {}/{}", ALT, KEY_LEFT, KEY_RIGHT),
            "Tilt camera".to_owned(),
        ),
        (
            format!(
                "{} + {}/{}/{}/{}",
                CTRL, KEY_LEFT, KEY_RIGHT, KEY_UP, KEY_DOWN
            ),
            "Apply symetry to view".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{}", LCLICK), "Select".to_owned()),
        (
            format!("{} + {}", SHIFT, LCLICK),
            "Multiple Select".to_owned(),
        ),
        (
            format!("{} Drag", LCLICK),
            "Rectangular selection".to_owned(),
        ),
        (
            format!("{} Drag, followed by {ALT} before releasing", LCLICK),
            "PNG export of rectangular area".to_owned(),
        ),
        (String::new(), String::new()),
        ("On helix numbers".to_owned(), String::new()),
        (format!("{}", LCLICK), "Select helix".to_owned()),
        (
            format!("{} + {}", SHIFT, LCLICK),
            "Multiple select".to_owned(),
        ),
        (
            format!("{} Drag", LCLICK),
            "Translate selected helices".to_owned(),
        ),
        (
            format!("{} Drag", RCLICK),
            "Rotate selected helices".to_owned(),
        ),
        (String::new(), String::new()),
        ("On nucleotides".to_owned(), String::new()),
        (
            format!("{}", RCLICK),
            "cut/glue strand or double xover".to_owned(),
        ),
        (
            format!("{} Drag", LCLICK),
            "edit strand/crossover".to_owned(),
        ),
        (
            format!("{} + {}", CTRL, LCLICK),
            "Make suggested crossover".to_owned(),
        ),
    ]
}

fn link_row<S: AppState>(link: &'static str, ui_size: UiSize) -> iced::Element<Message<S>> {
    iced_native::row![
        iced_native::column![text(link),].width(Length::FillPortion(3)),
        iced_native::column![text_btn("Go", ui_size).on_press(Message::OpenLink(link)),]
            .width(Length::FillPortion(1)),
    ]
    .into()
}

fn values_of_selection(selection: &Selection, reader: &dyn DesignReader) -> Vec<String> {
    match selection {
        Selection::Grid(_, g_id) => {
            let b1 = reader.grid_has_persistent_phantom(*g_id);
            let b2 = reader.grid_has_small_spheres(*g_id);
            let mut ret: Vec<String> = vec![b1, b2]
                .iter()
                .map(|b| {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                })
                .collect();
            if let Some(f) = reader.get_grid_nb_turn(*g_id) {
                ret.push(f.to_string());
            }
            ret
        }
        Selection::Strand(_, s_id) => vec![
            format!(
                "{:?}",
                reader.get_strand_length(*s_id as usize).unwrap_or(0)
            ),
            format!("{:?}", reader.is_id_of_scaffold(*s_id as usize)),
            s_id.to_string(),
            reader.length_decomposition(*s_id as usize),
            reader.strand_name(*s_id as usize),
        ],
        Selection::Nucleotide(_, nucl) => {
            vec![format!("{}", reader.nucl_is_anchor(*nucl))]
        }
        Selection::Xover(_, xover_id) => fmt_xover_len(reader.xover_length(*xover_id)),
        _ => Vec::new(),
    }
}

fn fmt_xover_len(info: Option<(f32, Option<f32>)>) -> Vec<String> {
    match info {
        Some((len_self, Some(len_neighbour))) => vec![
            format!("length {:.2} nm", len_self),
            format!("{:.2} nm", len_neighbour),
        ],
        Some((len, None)) => vec![format!("length {:.2} nm", len)],
        None => vec![String::from("Error getting length")],
    }
}

struct AddStrandMenu {
    helix_pos: isize,
    helix_length: usize,
    pos_str: String,
    length_str: String,
    text_inputs_are_active: bool,
    builder_input: [widget::text_input::State; 2],
}

impl Default for AddStrandMenu {
    fn default() -> Self {
        Self {
            helix_pos: 0,
            helix_length: 0,
            pos_str: "0".into(),
            length_str: "0".into(),
            text_inputs_are_active: false,
            builder_input: Default::default(),
        }
    }
}

impl AddStrandMenu {
    fn update_pos_str(&mut self, position_str: String) -> (isize, usize) {
        if let Ok(position) = position_str.parse::<isize>() {
            self.helix_pos = position;
        }
        self.pos_str = position_str;
        self.set_show_strand(true);
        (self.helix_pos, self.helix_length)
    }

    fn update_length_str(&mut self, length_str: String) -> (isize, usize) {
        if let Ok(length) = length_str.parse::<usize>() {
            self.helix_length = length
        }
        self.length_str = length_str;
        self.set_show_strand(true);
        (self.helix_pos, self.helix_length)
    }

    fn has_keyboard_priority(&self) -> bool {
        self.builder_input.iter().any(|s| s.is_focused())
    }

    fn get_build_helix_mode(&self) -> ActionMode {
        let (length, position) = if self.text_inputs_are_active {
            (self.helix_length, self.helix_pos)
        } else {
            (0, 0)
        };
        ActionMode::BuildHelix { length, position }
    }

    fn get_new_strand_parameters(&self) -> Option<(isize, usize)> {
        if self.text_inputs_are_active {
            Some((self.helix_pos, self.helix_length))
        } else {
            None
        }
    }

    fn set_show_strand(&mut self, show: bool) {
        self.text_inputs_are_active = show;
    }

    #[allow(clippy::needless_lifetimes)]
    fn view<S: AppState>(&self, ui_size: UiSize, width: u16) -> iced::Element<Message<S>> {
        let mut ret = widget::Column::new();
        let mut inputs = self.builder_input.iter_mut();
        let position_input = widget::TextInput::new("Position", &self.pos_str)
            .on_input(Message::PositionHelicesChanged)
            .style(BadValue(self.pos_str == self.helix_pos.to_string()));

        let length_input = widget::TextInput::new("Length", &self.length_str)
            .on_input(Message::LengthHelicesChanged)
            .style(BadValue(self.length_str == self.helix_length.to_string()));

        ret = ret.push(right_checkbox(
            self.text_inputs_are_active,
            "Add double strand on helix",
            Message::AddDoubleStrandHelix,
            ui_size,
        ));
        let color_white = Color::WHITE;
        let color_gray = Color {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };
        let color_choose_strand_start_length = if self.text_inputs_are_active {
            color_white
        } else {
            color_gray
        };
        let row = widget::Row::new()
            .push(
                widget::Column::new()
                    .push(
                        Text::new("Starting nt")
                            .style(iced::theme::Text::Color(color_choose_strand_start_length)),
                    )
                    .push(position_input)
                    .width(width / 2),
            )
            .push(
                widget::Column::new()
                    .push(
                        Text::new("Length (nt)")
                            .style(iced::theme::Text::Color(color_choose_strand_start_length)),
                    )
                    .push(length_input),
            );
        ret = ret.push(row);
        ret.into()
    }
}

struct InsertionLengthState {
    state: widget::text_input::State,
    selection: Selection,
    input_str: Option<String>,
}

impl Default for InsertionLengthState {
    fn default() -> Self {
        Self {
            state: Default::default(),
            selection: Selection::Nothing,
            input_str: None,
        }
    }
}

impl InsertionLengthState {
    fn update_selection(&mut self, selection: &Selection) {
        if selection != &self.selection {
            self.input_str = None;
            self.selection = *selection;
        }
    }

    fn has_keyboard_priority(&self) -> bool {
        self.state.is_focused()
    }
}

pub(super) struct InsertionRequest {
    pub selection: Selection,
    pub length: usize,
}
