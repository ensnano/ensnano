pub(super) mod value_constructor;

use crate::{
    AppState, GuiDesignReaderExt, Requests,
    helpers::{extra_jump, right_checkbox, section, subsection, text_button},
    left_panel::Message,
    theme,
};
use ensnano_design::{bezier_plane::BezierVertexId, grid::GridId};
use ensnano_organizer::keyboard_priority::keyboard_priority;
use ensnano_utils::{
    SimulationState,
    consts::{
        ALT, BACKSPACE_CHAR, CTRL, HELIX_CHAR, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_UP, L_CLICK,
        M_CLICK, MOVE_CHAR, NUCL_CHAR, R_CLICK, ROT_CHAR, SELECT_CHAR, SHIFT, STRAND_CHAR,
        SUPPR_CHAR,
    },
    selection::{ActionMode, Selection},
    ui_size::UiSize,
};
use iced::{
    Alignment, Command, Length,
    alignment::Horizontal,
    widget::{Column, Space, checkbox, column, row, scrollable, text, text_input},
};
use std::sync::{Arc, Mutex};
use ultraviolet::{Rotor3, Vec2, Vec3};
use value_constructor::{BezierVertexBuilder, Builder, GridBuilder, InstantiatedValue, ValueKind};

pub(super) enum ValueRequest {
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
    fn from_value_and_selection(selection: &Selection, value: InstantiatedValue) -> Option<Self> {
        match value {
            InstantiatedValue::HelixGridPosition(v) => {
                if let Selection::Grid(_, g_id) = selection {
                    Some(Self::HelixGridPosition {
                        grid_id: *g_id,
                        position: v,
                    })
                } else {
                    log::error!("Received value {value:?} with selection {selection:?}");
                    None
                }
            }
            InstantiatedValue::GridOrientation(orientation) => {
                if let Selection::Grid(_, g_id) = selection {
                    Some(Self::GridOrientation {
                        grid_id: *g_id,
                        orientation,
                    })
                } else {
                    log::error!("Received value {value:?} with selection {selection:?}");
                    None
                }
            }
            InstantiatedValue::GridNbTurn(nb_turn) => {
                if let Selection::Grid(_, g_id) = selection {
                    Some(Self::GridNbTurn {
                        grid_id: *g_id,
                        nb_turn,
                    })
                } else {
                    log::error!("Received value {value:?} with selection {selection:?}");
                    None
                }
            }
            InstantiatedValue::BezierVertexPosition(pos) => {
                if let Selection::BezierVertex(vertex_id) = selection {
                    Some(Self::BezierVertexPosition {
                        vertex_id: *vertex_id,
                        position: pos,
                    })
                } else {
                    log::error!("Received value {value:?} with selection {selection:?}");
                    None
                }
            }
        }
    }

    pub(super) fn make_request<R: Requests>(&self, request: Arc<Mutex<R>>) {
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
                request.lock().unwrap().set_nb_turn(*grid_id, *nb_turn);
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

struct InstantiatedBuilder<State>
where
    State: AppState,
{
    selection: Selection,
    builder: Box<dyn Builder<State>>,
}

impl<State> InstantiatedBuilder<State>
where
    State: AppState,
{
    /// If a builder can be made from the selection, update the builder and return true. Otherwise,
    /// return false.
    fn update(
        &mut self,
        selection: &Selection,
        reader: &dyn GuiDesignReaderExt,
        app_state: &State,
    ) -> bool {
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

    fn new(selection: &Selection, reader: &dyn GuiDesignReaderExt) -> Option<Self> {
        Self::new_builder(selection, reader).map(|builder| Self {
            builder,
            selection: *selection,
        })
    }

    fn new_builder(
        selection: &Selection,
        reader: &dyn GuiDesignReaderExt,
    ) -> Option<Box<dyn Builder<State>>> {
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

pub(super) struct ContextualPanel<State>
where
    State: AppState,
{
    width: u16,
    pub force_help: bool,
    pub show_tutorial: bool,
    add_strand_menu: AddStrandMenu,
    builder: Option<InstantiatedBuilder<State>>,
    insertion_length_state: InsertionLengthState,
}

impl<State> ContextualPanel<State>
where
    State: AppState,
{
    pub(super) fn new(width: u16) -> Self {
        Self {
            width,
            force_help: false,
            show_tutorial: false,
            add_strand_menu: Default::default(),
            builder: None,
            insertion_length_state: Default::default(),
        }
    }

    pub(super) fn new_width(&mut self, width: u16) {
        self.width = width;
    }

    pub(super) fn update(&mut self, app_state: &State) -> Command<Message<State>> {
        let selection = app_state
            .get_selection()
            .first()
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
        self.insertion_length_state.update_selection(selection);
        Command::none()
    }

    fn update_builder(
        &mut self,
        selection: Option<&Selection>,
        reader: &dyn GuiDesignReaderExt,
        app_state: &State,
    ) {
        if let Some(s) = selection {
            if let Some(builder) = &mut self.builder {
                if !builder.update(s, reader, app_state) {
                    self.builder = None;
                }
            } else {
                self.builder = InstantiatedBuilder::new(s, reader);
            }
        } else {
            self.builder = None;
        }
    }

    pub(super) fn view(
        &self,
        ui_size: UiSize,
        app_state: &State,
    ) -> iced::Element<'_, Message<State>> {
        let selection = app_state
            .get_selection()
            .first()
            .unwrap_or(&Selection::Nothing);

        let nb_selected = app_state
            .get_selection()
            .iter()
            .filter(|s| !matches!(s, Selection::Nothing))
            .count();

        let xover_len = app_state
            .get_strand_building_state()
            .map(|b| b.dragged_nucl)
            .and_then(|nucl| {
                log::info!("dragged_nucl: {nucl:?}");
                app_state.get_reader().get_id_of_xover_involving_nucl(nucl)
            })
            .and_then(|id| app_state.get_reader().xover_length(id));

        let info_values = values_of_selection(selection, app_state.get_reader().as_ref());

        // NOTE: The branching below determines what is viewed in the contextual panel.
        //
        let mut content = if self.show_tutorial {
            let link = "http://ens-lyon.fr/ensnano";
            column![
                section("Tutorials", ui_size)
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Center),
                extra_jump(),
                subsection("ENSnano website", ui_size),
                row![
                    text(link),
                    Space::with_width(Length::Fill),
                    text_button("Go", ui_size).on_press(Message::OpenLink(link)),
                ],
            ]
        } else if self.force_help && xover_len.is_none() {
            turn_into_help_column(ui_size)
        } else if app_state.get_action_mode().is_build() {
            self.add_strand_menu.view(ui_size, self.width)
        } else if *selection == Selection::Nothing && xover_len.is_none() {
            turn_into_help_column(ui_size)
        } else if nb_selected > 1 {
            // NOTE: When the number of objects selected is greater than one,
            //       we only print the number of object selected.
            column![text(format!("{nb_selected} objects selected"))]
                .width(Length::Fill)
                .align_items(Alignment::Center)
        } else {
            // NOTE: Print information about selection.
            let mut column = Column::new();
            column = column.push(
                row![
                    Space::with_width(Length::FillPortion(1)),
                    column![text_button("Help", ui_size).on_press(Message::ForceHelp),]
                        .width(Length::FillPortion(1)),
                    Space::with_width(Length::FillPortion(1)),
                ]
                .width(Length::Fill)
                .align_items(Alignment::Center),
            );

            if !matches!(selection, Selection::Nothing) {
                column = column.push(text(selection.info()).size(ui_size.main_text()));
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
                    column = column.push(add_grid_content(info_values.clone(), ui_size, twisting));
                }
                Selection::Strand(_, _) => {
                    column = column.push(add_strand_content(info_values.clone(), ui_size));
                }
                Selection::Nucleotide(_, _) => {
                    let anchor = info_values[0].clone();
                    column = column.push(text(format!("Anchor {anchor}")));
                }
                Selection::Xover(_, _) => {
                    if xover_len.is_none() {
                        if let Some(info) = info_values.first() {
                            column = column.push(text(info));
                        }
                        if let Some(info) = info_values.get(1) {
                            column = column.push(text(info));
                        }
                    }
                }
                _ => (),
            }
            if let Some(builder) = &self.builder {
                column = column.push(builder.builder.view(ui_size, selection, app_state));
            }
            column
        };

        if let Some(info_values) = xover_len.map(|v| fmt_xover_len(Some(v))) {
            if let Some(info) = info_values.first() {
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
            content = content.push(row![
                text("Loopout"),
                keyboard_priority(
                    "Loopout",
                    Message::SetKeyboardPriority,
                    text_input("", text_input_content)
                        .on_input(Message::InsertionLengthInput)
                        .on_submit(Message::InsertionLengthSubmitted)
                )
            ]);
        }

        scrollable(content.max_width(self.width - 2)).into()
        // NOTE: I don't really understand why there is a “- 2” here.
    }

    pub(super) fn selection_value_changed<R: Requests>(&self, s: String, requests: Arc<Mutex<R>>) {
        if let Ok(g_id) = s.parse() {
            requests
                .lock()
                .unwrap()
                .toggle_helices_persistence_of_grid(g_id);
        }
    }

    pub(super) fn set_small_sphere<R: Requests>(&self, b: bool, requests: Arc<Mutex<R>>) {
        requests.lock().unwrap().set_small_sphere(b);
    }

    pub(super) fn scaffold_id_set<R: Requests>(&self, n: usize, b: bool, requests: Arc<Mutex<R>>) {
        if b {
            requests.lock().unwrap().set_scaffold_id(Some(n));
        } else {
            requests.lock().unwrap().set_scaffold_id(None);
        }
    }

    pub(super) fn state_updated(&mut self) {
        self.force_help = false;
        self.show_tutorial = false;
    }

    pub(super) fn update_pos_str(&mut self, position_str: String) -> (isize, usize) {
        self.add_strand_menu.update_pos_str(position_str)
    }

    pub(super) fn update_length_str(&mut self, length_str: String) -> (isize, usize) {
        self.add_strand_menu.update_length_str(length_str)
    }

    pub(super) fn get_build_helix_mode(&self) -> ActionMode {
        self.add_strand_menu.get_build_helix_mode()
    }

    pub(super) fn get_new_strand_parameters(&self) -> Option<(isize, usize)> {
        self.add_strand_menu.get_new_strand_parameters()
    }

    pub(super) fn set_show_strand(&mut self, show: bool) {
        self.add_strand_menu.set_show_strand(show);
    }

    pub(super) fn update_builder_value(&mut self, kind: ValueKind, n: usize, value: String) {
        if let Some(b) = &mut self.builder {
            b.builder.update_str_value(kind, n, value);
        } else {
            log::error!("Cannot update value: No instantiated builder");
        }
    }

    pub(super) fn submit_value(&mut self, kind: ValueKind) -> Option<ValueRequest> {
        if let Some(b) = &mut self.builder {
            if let Some(value) = b.builder.submit_value(kind) {
                ValueRequest::from_value_and_selection(&b.selection, value)
            } else {
                None
            }
        } else {
            log::error!("Cannot submit value: No instantiated builder");
            None
        }
    }

    pub(super) fn request_from_value(&mut self, value: InstantiatedValue) -> Option<ValueRequest> {
        if let Some(b) = &mut self.builder {
            ValueRequest::from_value_and_selection(&b.selection, value)
        } else {
            log::error!("Cannot submit value: No instantiated builder");
            None
        }
    }

    pub(super) fn update_insertion_length_input(&mut self, input: String) {
        self.insertion_length_state.input_str = Some(input);
    }

    pub(super) fn get_insertion_request(&self) -> Option<InsertionRequest> {
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

fn add_grid_content<'a, State: AppState>(
    info_values: Vec<String>,
    ui_size: UiSize,
    twisting: TwistStatus,
) -> iced::Element<'a, Message<State>> {
    column![
        // twist_button
        match twisting {
            TwistStatus::Twisting => text_button("Stop", ui_size).on_press(Message::StopSimulation),
            TwistStatus::CanTwist => text_button("Twist", ui_size).on_press(Message::StartTwist),
            TwistStatus::CannotTwist => text_button("Twist", ui_size),
        },
        checkbox(
            "Persistent phantoms",
            info_values[0].parse::<bool>().unwrap()
        )
        .on_toggle(|b| Message::SelectionValueChanged(bool_to_string(b)),)
        .size(ui_size.checkbox())
        .text_size(ui_size.main_text()),
        checkbox("No sphere", info_values[1].parse::<bool>().unwrap())
            .on_toggle(|b| { Message::SetSmallSpheres(b) })
            .size(ui_size.checkbox())
            .text_size(ui_size.main_text()),
    ]
    .into()
}

fn add_strand_content<'a, State: AppState>(
    info_values: Vec<String>,
    ui_size: UiSize,
) -> iced::Element<'a, Message<State>> {
    let s_id = info_values[2].parse::<usize>().unwrap();
    column![
        row![
            text("Name").size(ui_size.main_text()),
            keyboard_priority(
                "Name",
                Message::SetKeyboardPriority,
                text_input("Name", &info_values[4])
                    .on_input(move |new_name| { Message::StrandNameChanged(s_id, new_name) })
                    .size(ui_size.main_text())
            )
        ],
        text(format!("length {}", info_values[0])).size(ui_size.main_text()),
        checkbox("Scaffold", info_values[1].parse().unwrap())
            .on_toggle(move |b| { Message::ScaffoldIdSet(s_id, b) }),
        text(info_values[3].as_str()).size(ui_size.main_text()),
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

fn add_help_to_column<'a, State: AppState>(
    help_title: impl ToString,
    help: Vec<(String, String)>,
    ui_size: UiSize,
) -> Column<'a, Message<State>> {
    column![
        text(help_title).size(ui_size.intermediate_text()),
        column(help.iter().map(|(l, r)| {
            if l.is_empty() {
                row![Space::with_width(10)]
            } else if r.is_empty() {
                row![
                    text(l)
                        .width(Length::Fill)
                        .horizontal_alignment(Horizontal::Center)
                ]
            } else {
                row![
                    text(l)
                        .width(Length::FillPortion(5))
                        .horizontal_alignment(Horizontal::Right),
                    Space::with_width(Length::FillPortion(1)),
                    text(r).width(Length::FillPortion(5)),
                ]
            }
            .into()
        })),
    ]
}

fn turn_into_help_column<'a, State: AppState>(ui_size: UiSize) -> Column<'a, Message<State>> {
    column![
        section("Help", ui_size)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center),
        add_help_to_column("3D view", view_3d_help(), ui_size),
        Space::with_width(15),
        add_help_to_column("2D/3D view", view_2d_3d_help(), ui_size),
        Space::with_width(15),
        add_help_to_column("2D view", view_2d_help(), ui_size),
    ]
}

fn view_3d_help() -> Vec<(String, String)> {
    vec![
        (
            format!("{L_CLICK}"),
            "Select\nnt → strand → helix".to_owned(),
        ),
        (format!("{SHIFT}+{L_CLICK}"), "Multiple select".to_owned()),
        (String::new(), String::new()),
        (
            format!("2x{L_CLICK}"),
            "Center selection in 2D view".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{M_CLICK} Drag"), "Translate camera".to_owned()),
        (
            format!("{ALT}+{L_CLICK} Drag"),
            "Translate camera".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{R_CLICK}"), "Set pivot".to_owned()),
        (
            format!("{R_CLICK} Drag"),
            "Rotate camera around pivot (preserve the XZ plane)".to_owned(),
        ),
        (
            format!("{CTRL}+{R_CLICK} Drag"),
            "Rotate camera freely around pivot".to_owned(),
        ),
        (
            format!("{ALT}+{R_CLICK} Drag"),
            "Rotate camera around pivot (preserve the current horizon plane)".to_owned(),
        ),
        (format!("{SHIFT}+{R_CLICK} Drag"), "Tilt camera".to_owned()),
        (
            "⎵ (with cursor over the 3D scene)".to_owned(),
            "Export the current view in png format".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{L_CLICK} Drag"), "Edit strand".to_owned()),
        (
            format!("long {L_CLICK} Drag"),
            "Make crossover (drop on nt)".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("When in 3D {MOVE_CHAR} mode"), String::new()),
        (
            format!("{L_CLICK} on handle"),
            "Move selected object".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("When in 3D {ROT_CHAR} mode"), String::new()),
        (
            format!("{L_CLICK} on handle"),
            "Rotate selected object".to_owned(),
        ),
    ]
}

fn view_2d_3d_help() -> Vec<(String, String)> {
    vec![
        (format!("{CTRL} + C"), "Copy selection".to_owned()),
        (format!("{CTRL} + V"), "Paste".to_owned()),
        (format!("{CTRL} + J"), "Paste & repeat".to_owned()),
        (String::new(), String::new()),
        (
            format!("{SUPPR_CHAR} or {BACKSPACE_CHAR}"),
            "Delete selected strands".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{CTRL} + S"), "Save design".to_owned()),
        (format!("{CTRL} + O"), "Open design".to_owned()),
        (format!("{CTRL} + Z"), "Undo".to_owned()),
        (format!("{CTRL} + R"), "Redo".to_owned()),
        (String::new(), String::new()),
        ("Selection mode shortcuts".to_owned(), String::new()),
        ("'N' key".to_owned(), format!("Nucleotide, ({NUCL_CHAR})")),
        ("'S' key".to_owned(), format!("Strand ({STRAND_CHAR})")),
        ("'H' key".to_owned(), format!("Helix ({HELIX_CHAR})")),
        (String::new(), String::new()),
        ("Action mode shortcuts".to_owned(), String::new()),
        ("ESC".to_owned(), format!("Select ({SELECT_CHAR})")),
        ("'T' key".to_owned(), format!("Translation ({MOVE_CHAR})")),
        ("'R' key".to_owned(), format!("Rotation ({ROT_CHAR})")),
    ]
}

fn view_2d_help() -> Vec<(String, String)> {
    vec![
        (format!("{M_CLICK} Drag"), "Translate camera".to_owned()),
        (
            format!("{ALT} + {L_CLICK} Drag"),
            "Translate camera".to_owned(),
        ),
        (
            format!("{ALT} + {KEY_LEFT}/{KEY_RIGHT}"),
            "Tilt camera".to_owned(),
        ),
        (
            format!("{CTRL} + {KEY_LEFT}/{KEY_RIGHT}/{KEY_UP}/{KEY_DOWN}",),
            "Apply symmetry to view".to_owned(),
        ),
        (String::new(), String::new()),
        (format!("{L_CLICK}"), "Select".to_owned()),
        (format!("{SHIFT} + {L_CLICK}"), "Multiple Select".to_owned()),
        (
            format!("{L_CLICK} Drag"),
            "Rectangular selection".to_owned(),
        ),
        (
            format!("{L_CLICK} Drag, followed by {ALT} before releasing"),
            "PNG export of rectangular area".to_owned(),
        ),
        (String::new(), String::new()),
        ("On helix numbers".to_owned(), String::new()),
        (format!("{L_CLICK}"), "Select helix".to_owned()),
        (format!("{SHIFT} + {L_CLICK}"), "Multiple select".to_owned()),
        (
            format!("{L_CLICK} Drag",),
            "Translate selected helices".to_owned(),
        ),
        (
            format!("{R_CLICK} Drag",),
            "Rotate selected helices".to_owned(),
        ),
        (String::new(), String::new()),
        ("On nucleotides".to_owned(), String::new()),
        (
            format!("{R_CLICK}",),
            "cut/glue strand or double xover".to_owned(),
        ),
        (
            format!("{L_CLICK} Drag",),
            "edit strand/crossover".to_owned(),
        ),
        (
            format!("{CTRL} + {L_CLICK}"),
            "Make suggested crossover".to_owned(),
        ),
    ]
}

fn values_of_selection(selection: &Selection, reader: &dyn GuiDesignReaderExt) -> Vec<String> {
    match selection {
        Selection::Grid(_, g_id) => {
            let b1 = reader.grid_has_persistent_phantom(*g_id);
            let b2 = reader.grid_has_small_spheres(*g_id);
            let mut ret: Vec<String> = [b1, b2]
                .iter()
                .map(|b| {
                    if *b {
                        "true".to_owned()
                    } else {
                        "false".to_owned()
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
        Some((len_self, Some(len_neighbor))) => vec![
            format!("length {:.2} nm", len_self),
            format!("{:.2} nm", len_neighbor),
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
}

impl Default for AddStrandMenu {
    fn default() -> Self {
        Self {
            helix_pos: 0,
            helix_length: 0,
            pos_str: "0".into(),
            length_str: "0".into(),
            text_inputs_are_active: false,
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
            self.helix_length = length;
        }
        self.length_str = length_str;
        self.set_show_strand(true);
        (self.helix_pos, self.helix_length)
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
        self.text_inputs_are_active
            .then_some((self.helix_pos, self.helix_length))
    }

    fn set_show_strand(&mut self, show: bool) {
        self.text_inputs_are_active = show;
    }

    fn view<State: AppState>(&self, ui_size: UiSize, width: u16) -> Column<'_, Message<State>> {
        let color_choose_strand_start_length = if self.text_inputs_are_active {
            iced::theme::Text::Color(theme::GUI_PALETTE.text)
        } else {
            theme::DISABLED_TEXT
        };

        column![
            right_checkbox(
                self.text_inputs_are_active,
                "Add double strand on helix",
                Message::AddDoubleStrandHelix,
                ui_size,
            ),
            row![
                column![
                    text("Starting nt").style(color_choose_strand_start_length),
                    // position_input
                    keyboard_priority(
                        "Starting nt",
                        Message::SetKeyboardPriority,
                        text_input("Position", &self.pos_str)
                            .on_input(Message::PositionHelicesChanged)
                            .style(theme::BadValue(self.pos_str == self.helix_pos.to_string()))
                    )
                ]
                .width(width / 2),
                column![
                    text("Length (nt)").style(color_choose_strand_start_length),
                    // length_input
                    keyboard_priority(
                        "Length (nt)",
                        Message::SetKeyboardPriority,
                        text_input("Length", &self.length_str)
                            .on_input(Message::LengthHelicesChanged)
                            .style(theme::BadValue(
                                self.length_str == self.helix_length.to_string()
                            ))
                    )
                ],
            ]
        ]
    }
}

struct InsertionLengthState {
    selection: Selection,
    input_str: Option<String>,
}

impl Default for InsertionLengthState {
    fn default() -> Self {
        Self {
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
}

pub(super) struct InsertionRequest {
    pub selection: Selection,
    pub length: usize,
}
