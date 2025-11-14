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

mod color_picker;
mod contextual_panel;
mod discrete_value;
mod export_menu;
mod tabs;

pub use tabs::revolution_tab::*;

use super::{AppState, FogParameters, OverlayType, Requests};
use crate::ensnano_exports::ExportType;
use crate::ensnano_iced::{
    UiSize,
    color_picker::ColorPickerMessage,
    iced_aw::widgets::{TabBarPosition, TabLabel, Tabs},
    iced_runtime::Program,
    theme::GuiBackground,
};
use crate::ensnano_interactor::{
    ActionMode, EquadiffSolvingMethod, HyperboloidRequest, RapierSimulationRequest, Selection,
    SelectionConversion,
    app_state_parameters::{AppStateParameters, CheckXoversParameter, SuggestionParameters},
    graphics::{Background3D, HBondDisplay, RenderingMode},
};
use crate::ensnano_organizer::{Organizer, OrganizerMessage, tree::OrganizerTree};
use crate::{
    ensnano_design::{
        BezierPathId, CameraId, NamedParameter,
        elements::{DesignElement, DesignElementKey},
        grid::GridTypeDescr,
    },
    ensnano_iced::fonts::{ENSNANO_FONT, MATERIAL_ICONS_DARK},
};
use color_picker::ColorPicker;
use contextual_panel::{ContextualPanel, InstantiatedValue, ValueKind};
use discrete_value::{FactoryId, RequestFactory, Requestable, ValueId};
use export_menu::ExportMenu;
use iced::{Color, Command, Element, Length, widget::*};
use std::sync::{Arc, Mutex};
use tabs::{
    CameraShortcutPanel, CameraTab, EditionTab, GridTab, GuiTab, ParametersTab, PenTab,
    SequenceTab, SimulationTab, TabId,
};
use ultraviolet::Vec3;
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::Modifiers,
};

pub struct LeftPanel<R: Requests, S: AppState> {
    logical_size: LogicalSize<f64>,
    logical_position: LogicalPosition<f64>,
    requests: Arc<Mutex<R>>,
    active_tab: TabId,
    /// Provide an organized view of the object being edited.
    organizer: Organizer<DesignElement>,
    ui_size: UiSize,
    grid_tab: GridTab<S>,
    edition_tab: EditionTab<S>,
    camera_tab: CameraTab<S>,
    simulation_tab: SimulationTab<S>,
    sequence_tab: SequenceTab<S>,
    parameters_tab: ParametersTab<S>,
    pen_tab: PenTab<S>,
    revolution_tab: RevolutionTab<S>,
    contextual_panel: ContextualPanel<S>,
    camera_shortcut: CameraShortcutPanel,
    application_state: S,
    exports_menu: ExportMenu,
}

#[derive(Debug, Clone)]
pub enum Message<S: AppState> {
    Resized(LogicalSize<f64>, LogicalPosition<f64>),
    MakeGrids,
    StrandNameChanged(usize, String),
    ColorPickerMessage(ColorPickerMessage),
    NewGrid(GridTypeDescr),
    /// Set camera to fixed position.
    FixPoint(Vec3, Vec3),
    /// Rotate camera.
    RotateCam(f32, f32, f32),
    PositionHelicesChanged(String),
    LengthHelicesChanged(String),
    ScaffoldPositionInput(String),
    FogRadius(f32),
    FogLength(f32),
    RollSimulationRequest,
    StartRapierSimulation,
    DiscreteValue {
        factory_id: FactoryId,
        value_id: ValueId,
        value: f32,
    },
    NewHyperboloid,
    FinalizeHyperboloid,
    RollTargeted(bool),
    /// Start or Stop Rigid Grid simulation.
    RigidGridSimulation(bool),
    /// Start or Stop Rigid Helices simulation.
    RigidHelicesSimulation(bool),
    VolumeExclusion(bool),
    TabSelected(TabId),
    OrganizerMessage(OrganizerMessage<DesignElement>),
    ModifiersChanged(Modifiers),
    UiSizeChanged(UiSize),
    UiSizePicked(UiSize),
    StaplesRequested,
    OrigamisRequested,
    ToggleText(bool),
    AddDoubleStrandHelix(bool),
    ToggleVisibility(bool),
    AllVisible,
    Redim2dHelices(bool),
    InvertScroll(bool),
    BrownianMotion(bool),
    Nothing,
    CancelHyperboloid,
    SelectionValueChanged(String),
    SetSmallSpheres(bool),
    ScaffoldIdSet(usize, bool),
    SelectScaffold,
    ForceHelp,
    ShowTutorial,
    RenderingMode(RenderingMode),
    Background3D(Background3D),
    OpenLink(&'static str),
    NewApplicationState(S),
    FogChoice(tabs::FogChoices),
    SetScaffoldSeqButtonPressed,
    OptimizeScaffoldShiftPressed,
    ResetSimulation,
    EditCameraName(String),
    SubmitCameraName,
    StartEditCameraName(CameraId),
    DeleteCamera(CameraId),
    SelectCamera(CameraId),
    NewCustomCamera,
    NewSuggestionParameters(SuggestionParameters),
    ContextualValueChanged(ValueKind, usize, String),
    ContextualValueSubmitted(ValueKind),
    InstantiatedValueSubmitted(InstantiatedValue),
    CheckXoversParameter(CheckXoversParameter),
    FollowStereographicCamera(bool),
    ShowStereographicCamera(bool),
    ShowHBonds(HBondDisplay),
    RainbowScaffold(bool),
    StopSimulation,
    FinishRelaxation,
    StartTwist,
    NewDnaParameters(NamedParameter),
    SetExpandInsertions(bool),
    InsertionLengthInput(String),
    InsertionLengthSubmitted,
    NewBezierPlane,
    StartBezierPath,
    TurnPathIntoGrid {
        path_id: BezierPathId,
        grid_type: GridTypeDescr,
    },
    SetShowBezierPaths(bool),
    MakeBezierPathCyclic {
        path_id: BezierPathId,
        cyclic: bool,
    },
    Export(ExportType),
    StlExport,
    CurveBuilderPicked(CurveDescriptorBuilder<S>),
    RevolutionEquadiffSolvingMethodPicked(EquadiffSolvingMethod),
    RevolutionParameterUpdate {
        parameter_id: RevolutionParameterId,
        text: String,
    },
    InitRevolutionRelaxation,
    CancelExport,
    LoadSvgFile,
    ScreenShot2D,
    ScreenShot3D,
    SaveNucleotidesPositions,
    IncrRevolutionShift,
    DecrRevolutionShift,
    SetKeyboardPriority(bool),
    SetFocus(text_input::Id),
}

impl<R: Requests, S: AppState> LeftPanel<R, S> {
    /// Create a new [LeftPanel].
    pub fn new(
        requests: Arc<Mutex<R>>,
        logical_size: LogicalSize<f64>,
        logical_position: LogicalPosition<f64>,
        first_time: bool,
        state: &S,
        parameters: &AppStateParameters,
    ) -> Self {
        let mut organizer = Organizer::new();
        organizer.set_width(logical_size.width as u16);
        Self {
            logical_size,
            logical_position,
            requests,
            active_tab: if first_time {
                TabId::Grid
            } else {
                TabId::Sequence
            },
            organizer,
            ui_size: parameters.ui_size,
            grid_tab: GridTab::new(),
            edition_tab: EditionTab::new(),
            camera_tab: CameraTab::new(parameters),
            simulation_tab: SimulationTab::new(),
            sequence_tab: SequenceTab::new(),
            parameters_tab: ParametersTab::new(state),
            pen_tab: Default::default(),
            revolution_tab: Default::default(),
            contextual_panel: ContextualPanel::new(logical_size.width as u16),
            camera_shortcut: CameraShortcutPanel::new(logical_size.width as u16),
            application_state: state.clone(),
            exports_menu: Default::default(),
        }
    }

    /// Resize the [LeftPanel] to the given dimensions.
    pub fn resize(
        &mut self,
        logical_size: LogicalSize<f64>,
        logical_position: LogicalPosition<f64>,
    ) {
        self.logical_size = logical_size;
        self.logical_position = logical_position;
        self.contextual_panel.new_width(logical_size.width as u16);
        self.camera_shortcut.new_width(logical_size.width as u16);
        self.organizer.set_width(logical_size.width as u16);
    }

    /// Convert an [OrganizerMessage] into a LeftPanel [Message].
    fn organizer_message(&mut self, m: OrganizerMessage<DesignElement>) -> Option<Message<S>> {
        match m {
            OrganizerMessage::InternalMessage(m) => {
                let selection = self
                    .application_state
                    .get_selection()
                    .iter()
                    .filter_map(|s| DesignElementKey::from_selection(s, 0))
                    .collect();
                return self
                    .organizer
                    .message(&m, &selection)
                    .map(|m_| Message::OrganizerMessage(m_));
            }
            OrganizerMessage::Selection(s, group_id) => self
                .requests
                .lock()
                .unwrap()
                .set_selected_keys(s, group_id, false),
            OrganizerMessage::NewAttribute(a, keys) => {
                self.requests
                    .lock()
                    .unwrap()
                    .update_attribute_of_elements(a, keys.into_iter().collect());
            }
            OrganizerMessage::NewTree(tree) => {
                self.requests.lock().unwrap().update_organizer_tree(tree);
            }
            OrganizerMessage::Candidates(candidates) => self
                .requests
                .lock()
                .unwrap()
                .set_candidates_keys(candidates),
            OrganizerMessage::NewGroup {
                group_id,
                elements_selected,
                new_tree,
            } => {
                self.requests
                    .lock()
                    .unwrap()
                    .update_organizer_tree(new_tree);
                self.requests.lock().unwrap().set_selected_keys(
                    elements_selected,
                    Some(group_id),
                    true,
                );
            }
            OrganizerMessage::SetKeyboardPriority(priority) => self
                .requests
                .lock()
                .unwrap()
                .set_keyboard_priority(priority),
            OrganizerMessage::SetFocus(id) => return Some(Message::SetFocus(id)),
            _ => (),
        }
        None
    }
}

impl<R, S> Program for LeftPanel<R, S>
where
    R: Requests,
    S: AppState,
{
    type Theme = crate::ensnano_iced::Theme;
    type Renderer = super::Renderer;
    type Message = Message<S>;

    // BUG: Increasing the left panel too much crashes ENSnano.

    // NOTE: The Command feature of Iced has not been used in ENSnan.
    // NOTE: Trying it, it seems that commands are not executed.

    fn update(&mut self, message: Message<S>) -> Command<Message<S>> {
        let notify_new_tree =
            if let Some(tree) = self.application_state.get_reader().get_organizer_tree() {
                self.organizer.read_tree(tree.as_ref())
            } else {
                self.organizer.read_tree(&OrganizerTree::Node {
                    name: String::from("root"),
                    children: vec![],
                    expanded: true,
                    id: None,
                })
            };
        if notify_new_tree {
            self.requests
                .lock()
                .unwrap()
                .update_organizer_tree(self.organizer.tree());
        }
        log::debug!("Message: {:?}", &message);
        let command = match message {
            Message::StrandNameChanged(s_id, name) => {
                self.requests.lock().unwrap().set_strand_name(s_id, name);
                Command::none()
            }
            Message::ColorPickerMessage(message) => {
                self.edition_tab.update_color_picker(message);
                // Forward action to Requests.
                match message {
                    ColorPickerMessage::HueChanged(_)
                    | ColorPickerMessage::HsvSatValueChanged(_, _)
                    | ColorPickerMessage::ColorPicked(_) => {
                        let color = self.edition_tab.current_strand_color();
                        self.requests.lock().unwrap().change_strand_color(color);
                    }
                    ColorPickerMessage::FinishChangingColor => {
                        self.requests.lock().unwrap().finish_changing_color();
                    }
                }
                Command::none()
            }
            Message::Resized(size, position) => {
                self.resize(size, position);
                Command::none()
            }
            Message::NewGrid(grid_type) => {
                self.requests.lock().unwrap().create_grid(grid_type);
                let action_mode = self.contextual_panel.get_build_helix_mode();
                self.requests
                    .lock()
                    .unwrap()
                    .change_action_mode(action_mode);
                Command::none()
            }
            Message::RotateCam(xz, yz, xy) => {
                self.camera_shortcut
                    .set_angles(xz as isize, yz as isize, xy as isize);
                self.requests
                    .lock()
                    .unwrap()
                    .perform_camera_rotation(xz, yz, xy);
                Command::none()
            }
            Message::FixPoint(point, up) => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_camera_dir_up_vec(point, up);
                self.camera_shortcut.reset_angles();
                Command::none()
            }
            Message::LengthHelicesChanged(length_str) => {
                let new_strand_parameters = self.contextual_panel.update_length_str(length_str);
                self.requests
                    .lock()
                    .unwrap()
                    .add_double_strand_on_new_helix(Some(new_strand_parameters));
                Command::none()
            }
            Message::PositionHelicesChanged(position_str) => {
                let new_strand_parameters = self.contextual_panel.update_pos_str(position_str);
                self.requests
                    .lock()
                    .unwrap()
                    .add_double_strand_on_new_helix(Some(new_strand_parameters));
                Command::none()
            }
            Message::ScaffoldPositionInput(position_str) => {
                if let Some(n) = self.sequence_tab.update_pos_str(position_str) {
                    self.requests.lock().unwrap().set_scaffold_shift(n);
                };
                Command::none()
            }
            Message::FogLength(length) => {
                self.camera_tab.fog_length(length);
                let request = self.camera_tab.get_fog_request();
                self.requests.lock().unwrap().set_fog_parameters(request);
                Command::none()
            }
            Message::FogRadius(radius) => {
                self.camera_tab.fog_radius(radius);
                let request = self.camera_tab.get_fog_request();
                self.requests.lock().unwrap().set_fog_parameters(request);
                Command::none()
            }
            Message::RollSimulationRequest => {
                if self.application_state.get_simulation_state().is_rolling() {
                    self.requests.lock().unwrap().stop_simulations();
                } else {
                    let request = self.simulation_tab.get_physical_simulation_request();
                    self.requests.lock().unwrap().start_roll_simulation(request);
                }
                Command::none()
            }
            Message::StartRapierSimulation => {
                self.requests
                    .lock()
                    .unwrap()
                    .request_rapier_simulation(RapierSimulationRequest::Start);
                Command::none()
            }
            Message::FogChoice(choice) => {
                let (visible, from_camera, dark, reversed) = choice.to_param();
                self.camera_tab.fog_camera(from_camera);
                self.camera_tab.fog_visible(visible);
                self.camera_tab.fog_dark(dark);
                self.camera_tab.fog_reversed(reversed);
                let request = self.camera_tab.get_fog_request();
                self.requests.lock().unwrap().set_fog_parameters(request);
                Command::none()
            }
            Message::DiscreteValue {
                factory_id,
                value_id,
                value,
            } => match factory_id {
                FactoryId::Scroll => {
                    let mut request = None;
                    self.parameters_tab
                        .update_scroll_request(value_id, value, &mut request);
                    if let Some(request) = request {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_scroll_sensitivity(request);
                    }
                    Command::none()
                }
                FactoryId::HelixRoll => {
                    let mut request = None;
                    self.edition_tab
                        .update_roll_request(value_id, value, &mut request);
                    if let Some(request) = request {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_roll_of_selected_helices(request);
                    }
                    Command::none()
                }
                FactoryId::Hyperboloid => {
                    let mut request = None;
                    self.grid_tab
                        .update_hyperboloid_request(value_id, value, &mut request);
                    if let Some(request) = request {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_current_hyperboloid(request);
                    }
                    Command::none()
                }
                FactoryId::RigidBody => {
                    let mut request = None;
                    self.simulation_tab
                        .update_request(value_id, value, &mut request);
                    if let Some(request) = request {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_rigid_body_simulation_parameters(request);
                    }
                    Command::none()
                }
                FactoryId::Brownian => {
                    let mut request = None;
                    self.simulation_tab
                        .update_brownian(value_id, value, &mut request);
                    if let Some(request) = request {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_rigid_body_simulation_parameters(request);
                    }
                    Command::none()
                }
            },
            Message::VolumeExclusion(b) => {
                self.simulation_tab.set_volume_exclusion(b);
                let mut request: Option<RigidBodyParametersRequest> = None;
                self.simulation_tab.make_rigid_body_request(&mut request);
                if let Some(request) = request {
                    self.requests
                        .lock()
                        .unwrap()
                        .update_rigid_body_simulation_parameters(request);
                }
                Command::none()
            }
            Message::BrownianMotion(b) => {
                self.simulation_tab.set_brownian_motion(b);
                let mut request: Option<RigidBodyParametersRequest> = None;
                self.simulation_tab.make_rigid_body_request(&mut request);
                if let Some(request) = request {
                    self.requests
                        .lock()
                        .unwrap()
                        .update_rigid_body_simulation_parameters(request);
                }
                Command::none()
            }
            Message::NewHyperboloid => {
                let mut request: Option<HyperboloidRequest> = None;
                self.grid_tab.new_hyperboloid(&mut request);
                if let Some(request) = request {
                    self.requests
                        .lock()
                        .unwrap()
                        .create_new_hyperboloid(request);
                }
                Command::none()
            }
            Message::FinalizeHyperboloid => {
                self.requests.lock().unwrap().finalize_hyperboloid();
                Command::none()
            }
            Message::RigidGridSimulation(start) => {
                if start {
                    let mut request: Option<RigidBodyParametersRequest> = None;
                    self.simulation_tab.make_rigid_body_request(&mut request);
                    if let Some(request) = request {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_rigid_grids_simulation(request);
                    }
                } else {
                    self.requests.lock().unwrap().stop_simulations();
                }
                Command::none()
            }
            Message::RigidHelicesSimulation(start) => {
                if start {
                    let mut request: Option<RigidBodyParametersRequest> = None;
                    self.simulation_tab.make_rigid_body_request(&mut request);
                    if let Some(request) = request {
                        self.requests
                            .lock()
                            .unwrap()
                            .update_rigid_helices_simulation(request);
                    }
                } else {
                    self.requests.lock().unwrap().stop_simulations();
                }
                Command::none()
            }
            Message::MakeGrids => {
                self.requests.lock().unwrap().make_grid_from_selection();
                Command::none()
            }
            Message::RollTargeted(b) => {
                let selection = self.application_state.get_selection_as_design_element();
                if b {
                    if let Some(simulation_request) = self.edition_tab.get_roll_request(&selection)
                    {
                        self.requests
                            .lock()
                            .unwrap()
                            .start_roll_simulation(simulation_request);
                    }
                } else {
                    self.requests.lock().unwrap().stop_roll_simulation();
                }
                Command::none()
            }
            Message::TabSelected(tab_id) => {
                if let ActionMode::BuildHelix { .. } = self.application_state.get_action_mode()
                    && tab_id != TabId::Grid
                {
                    let action_mode = ActionMode::Normal;
                    self.requests
                        .lock()
                        .unwrap()
                        .change_action_mode(action_mode);
                }
                if tab_id != TabId::Grid && self.application_state.is_building_hyperboloid() {
                    self.requests.lock().unwrap().finalize_hyperboloid();
                }
                if self.active_tab == TabId::Revolution && tab_id != TabId::Revolution {
                    self.simulation_tab
                        .leave_tab(Arc::clone(&self.requests), &self.application_state);
                }
                if tab_id == TabId::Revolution {
                    self.requests.lock().unwrap().notify_revolution_tab();
                }
                self.active_tab = tab_id;
                Command::none()
            }
            Message::OrganizerMessage(m) => {
                let next_message = self.organizer_message(m);
                if let Some(message) = next_message {
                    self.update(message)
                } else {
                    Command::none()
                }
            }
            Message::ModifiersChanged(modifiers) => {
                self.organizer
                    .new_modifiers(iced_winit::conversion::modifiers(modifiers.state()));
                Command::none()
            }
            Message::UiSizePicked(ui_size) => {
                self.requests.lock().unwrap().set_ui_size(ui_size);
                Command::none()
            }
            Message::UiSizeChanged(ui_size) => {
                self.ui_size = ui_size;
                Command::none()
            }
            Message::SetScaffoldSeqButtonPressed => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_scaffold_sequence(self.sequence_tab.get_scaffold_shift());
                Command::none()
            }
            Message::OptimizeScaffoldShiftPressed => {
                self.requests.lock().unwrap().optimize_scaffold_shift();
                Command::none()
            }
            Message::StaplesRequested => {
                self.requests.lock().unwrap().download_staples();
                Command::none()
            }
            Message::ToggleText(b) => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_dna_sequences_visibility(b);
                self.sequence_tab.toggle_text_value(b);
                Command::none()
            }
            Message::AddDoubleStrandHelix(b) => {
                self.contextual_panel.set_show_strand(b);
                let new_strand_parameters = self.contextual_panel.get_new_strand_parameters();
                self.requests
                    .lock()
                    .unwrap()
                    .add_double_strand_on_new_helix(new_strand_parameters);
                Command::none()
            }
            Message::ToggleVisibility(b) => {
                self.requests.lock().unwrap().toggle_visibility(b);
                Command::none()
            }
            Message::AllVisible => {
                self.requests.lock().unwrap().make_all_elements_visible();
                Command::none()
            }
            Message::Redim2dHelices(b) => {
                self.requests.lock().unwrap().resize_2d_helices(b);
                Command::none()
            }
            Message::InvertScroll(b) => {
                self.requests.lock().unwrap().invert_scroll(b);
                Command::none()
            }
            Message::CancelHyperboloid => {
                self.requests.lock().unwrap().cancel_hyperboloid();
                Command::none()
            }
            Message::SelectionValueChanged(s) => {
                self.contextual_panel
                    .selection_value_changed(s, Arc::clone(&self.requests));
                Command::none()
            }
            Message::SetSmallSpheres(b) => {
                self.contextual_panel
                    .set_small_sphere(b, Arc::clone(&self.requests));
                Command::none()
            }
            Message::ScaffoldIdSet(n, b) => {
                self.contextual_panel
                    .scaffold_id_set(n, b, Arc::clone(&self.requests));
                Command::none()
            }
            Message::SelectScaffold => {
                self.requests.lock().unwrap().set_scaffold_from_selection();
                Command::none()
            }
            Message::RenderingMode(mode) => {
                self.requests.lock().unwrap().change_3d_rendering_mode(mode);
                self.camera_tab.rendering_mode = mode;
                Command::none()
            }
            Message::Background3D(bg) => {
                self.requests.lock().unwrap().change_3d_background(bg);
                self.camera_tab.background3d = bg;
                Command::none()
            }
            Message::ForceHelp => {
                self.contextual_panel.force_help = true;
                self.contextual_panel.show_tutorial = false;
                Command::none()
            }
            Message::ShowTutorial => {
                self.contextual_panel.show_tutorial ^= true;
                self.contextual_panel.force_help = false;
                Command::none()
            }
            Message::OpenLink(link) => {
                // ATM we continue even in case of error, later any error will be prompted to user
                let _ = open::that(link);
                Command::none()
            }
            Message::NewApplicationState(state) => {
                if state.design_was_modified(&self.application_state) {
                    let reader = state.get_reader();
                    self.organizer.update_elements(reader.get_dna_elements());
                    self.contextual_panel.state_updated();
                    let unrooted_surface = self
                        .revolution_tab
                        .get_current_unrooted_surface(&self.application_state);
                    self.requests
                        .lock()
                        .unwrap()
                        .set_unrooted_surface(unrooted_surface);
                }
                if state.selection_was_updated(&self.application_state) {
                    let selected_group = state.get_selected_group();
                    self.organizer.notify_selection(selected_group);
                    self.contextual_panel.state_updated();
                }
                if state.get_action_mode() != self.application_state.get_action_mode() {
                    self.contextual_panel.state_updated();
                }
                self.application_state = state;
                let _ = self.revolution_tab.update(&mut self.application_state);
                Command::none()
            }
            Message::ResetSimulation => {
                self.requests.lock().unwrap().reset_simulations();
                Command::none()
            }
            Message::Nothing => Command::none(),
            Message::SubmitCameraName => {
                if let Some((id, name)) = self.camera_shortcut.stop_editing() {
                    self.requests.lock().unwrap().set_camera_name(id, name);
                }
                Command::none()
            }
            Message::EditCameraName(name) => {
                self.camera_shortcut.set_camera_input_name(name);
                Command::none()
            }
            Message::StartEditCameraName(camera_id) => {
                self.camera_shortcut.start_editing(camera_id);
                Command::none()
            }
            Message::DeleteCamera(camera_id) => {
                self.requests.lock().unwrap().delete_camera(camera_id);
                Command::none()
            }
            Message::SelectCamera(camera_id) => {
                self.requests.lock().unwrap().select_camera(camera_id);
                Command::none()
            }
            Message::NewCustomCamera => {
                self.requests.lock().unwrap().create_new_camera();
                self.camera_shortcut.scroll_down();
                Command::none()
            }
            Message::NewSuggestionParameters(param) => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_suggestion_parameters(param);
                Command::none()
            }
            Message::ContextualValueSubmitted(kind) => {
                if let Some(request) = self.contextual_panel.submit_value(kind) {
                    request.make_request(Arc::clone(&self.requests));
                }
                Command::none()
            }
            Message::ContextualValueChanged(kind, n, val) => {
                self.contextual_panel.update_builder_value(kind, n, val);
                Command::none()
            }
            Message::InstantiatedValueSubmitted(value) => {
                if let Some(request) = self.contextual_panel.request_from_value(value) {
                    request.make_request(Arc::clone(&self.requests));
                }
                Command::none()
            }
            Message::CheckXoversParameter(parameters) => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_check_xover_parameters(parameters);
                Command::none()
            }
            Message::FollowStereographicCamera(b) => {
                self.requests.lock().unwrap().follow_stereographic_camera(b);
                Command::none()
            }
            Message::ShowStereographicCamera(b) => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_show_stereographic_camera(b);
                Command::none()
            }
            Message::ShowHBonds(b) => {
                self.requests.lock().unwrap().set_show_h_bonds(b);
                Command::none()
            }
            Message::RainbowScaffold(b) => {
                self.requests.lock().unwrap().set_rainbow_scaffold(b);
                Command::none()
            }
            Message::StopSimulation => {
                self.requests.lock().unwrap().stop_simulations();
                Command::none()
            }
            Message::StartTwist => {
                if let Some(Selection::Grid(_, g_id)) =
                    self.application_state.get_selection().first()
                {
                    self.requests.lock().unwrap().start_twist_simulation(*g_id);
                }
                Command::none()
            }
            Message::OrigamisRequested => {
                self.requests.lock().unwrap().download_origamis();
                Command::none()
            }
            Message::NewDnaParameters(parameters) => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_dna_parameters(parameters.value);
                Command::none()
            }
            Message::SetExpandInsertions(b) => {
                self.requests.lock().unwrap().set_expand_insertions(b);
                Command::none()
            }
            Message::InsertionLengthInput(s) => {
                self.contextual_panel.update_insertion_length_input(s);
                Command::none()
            }
            Message::InsertionLengthSubmitted => {
                if let Some(request) = self.contextual_panel.get_insertion_request() {
                    if let Some(insertion_point) = self
                        .application_state
                        .get_reader()
                        .get_insertion_point(&request.selection)
                    {
                        self.requests
                            .lock()
                            .unwrap()
                            .set_insertion_length(insertion_point, request.length);
                    } else {
                        log::error!("No insertion point for {:?}", request.selection);
                    }
                }
                Command::none()
            }
            Message::NewBezierPlane => {
                self.requests.lock().unwrap().create_bezier_plane();
                Command::none()
            }
            Message::StartBezierPath => {
                self.requests
                    .lock()
                    .unwrap()
                    .change_action_mode(ActionMode::EditBezierPath);
                Command::none()
            }
            Message::TurnPathIntoGrid { path_id, grid_type } => {
                self.requests
                    .lock()
                    .unwrap()
                    .turn_path_into_grid(path_id, grid_type);
                Command::none()
            }
            Message::SetShowBezierPaths(b) => {
                self.requests.lock().unwrap().set_show_bezier_paths(b);
                Command::none()
            }
            Message::MakeBezierPathCyclic { path_id, cyclic } => {
                self.requests
                    .lock()
                    .unwrap()
                    .make_bezier_path_cyclic(path_id, cyclic);
                Command::none()
            }
            Message::Export(export_type) => {
                self.requests.lock().unwrap().export(export_type);
                Command::none()
            }
            Message::CancelExport => {
                self.requests.lock().unwrap().set_exporting(false);
                Command::none()
            }
            Message::CurveBuilderPicked(builder) => {
                self.revolution_tab.set_builder(builder);
                let bezier_path_id = self.revolution_tab.get_current_bezier_path_id();
                self.requests
                    .lock()
                    .unwrap()
                    .set_bezier_revolution_id(bezier_path_id);
                let unrooted_surface = self
                    .revolution_tab
                    .get_current_unrooted_surface(&self.application_state);
                self.requests
                    .lock()
                    .unwrap()
                    .set_unrooted_surface(unrooted_surface);
                Command::none()
            }
            Message::RevolutionEquadiffSolvingMethodPicked(method) => {
                self.revolution_tab.set_method(method);
                Command::none()
            }
            Message::RevolutionParameterUpdate { parameter_id, text } => {
                if let RevolutionParameterId::RevolutionRadius = parameter_id
                    && let Some(radius) = text.parse::<f64>().ok()
                {
                    self.requests
                        .lock()
                        .unwrap()
                        .set_bezier_revolution_radius(radius);
                }
                self.revolution_tab
                    .update_builder_parameter(parameter_id, text);
                let bezier_path_id = self.revolution_tab.get_current_bezier_path_id();
                self.requests
                    .lock()
                    .unwrap()
                    .set_bezier_revolution_id(bezier_path_id);
                let unrooted_surface = self
                    .revolution_tab
                    .get_current_unrooted_surface(&self.application_state);
                self.requests
                    .lock()
                    .unwrap()
                    .set_unrooted_surface(unrooted_surface);
                Command::none()
            }
            Message::InitRevolutionRelaxation => {
                if let Some(desc) = self
                    .revolution_tab
                    .get_revolution_system(&self.application_state, true)
                {
                    self.requests
                        .lock()
                        .unwrap()
                        .start_revolution_relaxation(desc);
                }
                Command::none()
            }
            Message::FinishRelaxation => {
                self.requests.lock().unwrap().finish_revolution_relaxation();
                Command::none()
            }
            Message::LoadSvgFile => {
                self.requests.lock().unwrap().load_svg();
                Command::none()
            }
            Message::StlExport => {
                self.requests.lock().unwrap().request_stl_export();
                Command::none()
            }
            Message::ScreenShot2D => {
                self.requests.lock().unwrap().request_screenshot_2d();
                Command::none()
            }
            Message::ScreenShot3D => {
                self.requests.lock().unwrap().request_screenshot_3d();
                Command::none()
            }
            Message::SaveNucleotidesPositions => {
                self.requests
                    .lock()
                    .unwrap()
                    .request_save_nucleotides_positions();
                Command::none()
            }
            Message::IncrRevolutionShift => {
                self.revolution_tab.shift_idx += 1;
                Command::none()
            }
            Message::DecrRevolutionShift => {
                self.revolution_tab.shift_idx -= 1;
                Command::none()
            }
            Message::SetKeyboardPriority(priority) => {
                self.requests
                    .lock()
                    .unwrap()
                    .set_keyboard_priority(priority);
                Command::none()
            }
            Message::SetFocus(id) => text_input::focus(id),
        };

        let command = Command::batch(vec![
            command,
            self.grid_tab.update(&mut self.application_state),
            self.edition_tab.update(&mut self.application_state),
            self.camera_tab.update(&mut self.application_state),
            self.simulation_tab.update(&mut self.application_state),
            self.sequence_tab.update(&mut self.application_state),
            self.parameters_tab.update(&mut self.application_state),
            self.pen_tab.update(&mut self.application_state),
            self.revolution_tab.update(&mut self.application_state),
            self.camera_shortcut.update(&mut self.application_state),
            self.contextual_panel.update(&mut self.application_state),
        ]);
        log::debug!("Command: {:?}", &command);
        command
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        let width = self.logical_size.cast::<u16>().width;
        let tabs = Tabs::new(Message::TabSelected)
            .push(
                TabId::Grid,
                self.grid_tab.label(),
                self.grid_tab.view(self.ui_size, &self.application_state),
            )
            .push(
                TabId::Edition,
                self.edition_tab.label(),
                self.edition_tab.view(self.ui_size, &self.application_state),
            )
            .push(
                TabId::Camera,
                self.camera_tab.label(),
                self.camera_tab.view(self.ui_size, &self.application_state),
            )
            .push(
                TabId::Simulation,
                self.simulation_tab.label(),
                self.simulation_tab
                    .view(self.ui_size, &self.application_state),
            )
            .push(
                TabId::Sequence,
                self.sequence_tab.label(),
                self.sequence_tab
                    .view(self.ui_size, &self.application_state),
            )
            .push(
                TabId::Parameters,
                self.parameters_tab.label(),
                self.parameters_tab
                    .view(self.ui_size, &self.application_state),
            )
            .push(
                TabId::Pen,
                self.pen_tab.label(),
                self.pen_tab.view(self.ui_size, &self.application_state),
            )
            .push(
                TabId::Revolution,
                self.revolution_tab.label(),
                self.revolution_tab
                    .view(self.ui_size, &self.application_state),
            )
            .set_active_tab(&self.active_tab)
            .tab_bar_position(TabBarPosition::Top)
            .icon_font(ENSNANO_FONT)
            .icon_size(self.ui_size.icon())
            .text_font(MATERIAL_ICONS_DARK)
            .text_size(self.ui_size.main_text())
            .tab_bar_height(Length::Fixed(self.ui_size.tab_bar_height()))
            .tab_bar_style(GuiBackground.into())
            .width(Length::Fixed(width as f32))
            .height(Length::Fill);
        // NOTE: The style, height and width values are necessary to clear the tab when
        //       switching to a new tab.
        //
        let camera_shortcut = self
            .camera_shortcut
            .view(self.ui_size, &self.application_state);
        let contextual_menu = self
            .contextual_panel
            .view(self.ui_size, &self.application_state);

        let selection: std::collections::BTreeSet<DesignElementKey> = self
            .application_state
            .get_selection()
            .iter()
            .filter_map(|e| DesignElementKey::from_selection(e, 0))
            .collect();

        let organizer = self
            .organizer
            .view(selection)
            .map(|m| Message::OrganizerMessage(m));

        let first_container = if self.application_state.is_exporting() {
            container(self.exports_menu.view())
        } else {
            container(tabs)
        };

        container(
            self::column![
                first_container.height(Length::FillPortion(2)),
                horizontal_rule(5),
                container(camera_shortcut).height(Length::FillPortion(1)),
                horizontal_rule(5),
                container(contextual_menu).height(Length::FillPortion(1)),
                horizontal_rule(5),
                container(organizer).height(Length::FillPortion(2)),
            ]
            .width(Length::Fill)
            .padding(1),
        )
        .style(GuiBackground)
        .height(self.logical_size.height as f32)
        .into()
    }
}

// TODO: Remove ColorOverlay

pub struct ColorOverlay<R: Requests> {
    logical_size: LogicalSize<f64>,
    color_picker: ColorPicker,
    requests: Arc<Mutex<R>>,
}

impl<R: Requests> ColorOverlay<R> {
    pub fn new(requests: Arc<Mutex<R>>, logical_size: LogicalSize<f64>) -> Self {
        Self {
            logical_size,
            color_picker: ColorPicker::new(),
            requests,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColorMessage {
    HsvSatValueChanged(f64, f64),
    HueChanged(f64),
    FinishChangingColor,
    Closed,
}

impl<R: Requests> Program for ColorOverlay<R> {
    type Renderer = super::Renderer;
    type Theme = crate::ensnano_iced::Theme;
    type Message = ColorMessage;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            ColorMessage::HsvSatValueChanged(_sat, _value) => {}
            ColorMessage::HueChanged(x) => self.color_picker.change_hue(x),
            ColorMessage::Closed => {
                self.requests
                    .lock()
                    .unwrap()
                    .close_overlay(OverlayType::Color);
            }
            ColorMessage::FinishChangingColor => {
                self.requests.lock().unwrap().finish_changing_color();
            }
        };
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        let width = self.logical_size.cast::<u16>().width;

        let widget = Column::new()
            .width(width)
            .height(Length::Fill)
            .spacing(5)
            .push(self.color_picker.new_view())
            .spacing(5)
            .push(Button::new(Text::new("Close")).on_press(ColorMessage::Closed));

        Container::new(widget)
            //.style(FloatingStyle)
            // TODO: Maybe reimplement style.
            .height(Length::Fill)
            .into()
    }
}

pub struct Hyperboloid_ {}

impl Requestable for Hyperboloid_ {
    type Request = HyperboloidRequest;
    fn request_from_values(&self, values: &[f32]) -> HyperboloidRequest {
        HyperboloidRequest {
            radius: values[0].round() as usize,
            length: values[1],
            shift: values[2],
            radius_shift: values[3],
            nb_turn: values[4] as f64,
        }
    }
    fn nb_values(&self) -> usize {
        5
    }
    fn initial_value(&self, n: usize) -> f32 {
        match n {
            0 => 10f32,
            1 => 30f32,
            2 => 0f32,
            3 => 0.2f32,
            4 => 0.0f32,
            _ => unreachable!(),
        }
    }
    fn min_val(&self, n: usize) -> f32 {
        use std::f32::consts::PI;
        match n {
            0 => 5f32,
            1 => 1f32,
            2 => -PI + 1f32.to_radians(),
            3 => 0.,
            4 => -5f32,
            _ => unreachable!(),
        }
    }

    fn max_val(&self, n: usize) -> f32 {
        match n {
            0 => 60f32,
            1 => 1000f32,
            2 => 2.,
            3 => 1f32,
            4 => 5f32,
            _ => unreachable!(),
        }
    }
    fn step_val(&self, n: usize) -> f32 {
        match n {
            0 => 1f32,
            1 => 1f32,
            2 => 0.01,
            3 => 0.01,
            4 => 0.05,
            _ => unreachable!(),
        }
    }
    fn name_val(&self, n: usize) -> String {
        match n {
            0 => String::from("Nb helices"),
            1 => String::from("Strands length"),
            2 => String::from("Shift"),
            3 => String::from("Tube radius"),
            4 => String::from("nb turn"),
            _ => unreachable!(),
        }
    }

    fn hidden(&self, n: usize) -> bool {
        n == 2 || n == 3
    }
}

struct ScrollSensitivity {
    initial_value: f32,
}

impl Requestable for ScrollSensitivity {
    type Request = f32;
    fn request_from_values(&self, values: &[f32]) -> f32 {
        values[0]
    }
    fn nb_values(&self) -> usize {
        1
    }
    fn initial_value(&self, n: usize) -> f32 {
        if n == 0 {
            self.initial_value
        } else {
            unreachable!()
        }
    }
    fn min_val(&self, n: usize) -> f32 {
        if n == 0 { -10f32 } else { unreachable!() }
    }
    fn max_val(&self, n: usize) -> f32 {
        if n == 0 { 10f32 } else { unreachable!() }
    }
    fn step_val(&self, n: usize) -> f32 {
        if n == 0 { 0.5f32 } else { unreachable!() }
    }
    fn name_val(&self, n: usize) -> String {
        if n == 0 {
            String::from("Sensitivity")
        } else {
            unreachable!()
        }
    }
}

struct HelixRoll {}

impl Requestable for HelixRoll {
    type Request = f32;
    fn request_from_values(&self, values: &[f32]) -> f32 {
        values[0]
    }
    fn nb_values(&self) -> usize {
        1
    }
    fn initial_value(&self, n: usize) -> f32 {
        match n {
            0 => 0f32,
            _ => unreachable!(),
        }
    }
    fn min_val(&self, n: usize) -> f32 {
        use std::f32::consts::PI;
        match n {
            0 => -PI,
            _ => unreachable!(),
        }
    }
    fn max_val(&self, n: usize) -> f32 {
        use std::f32::consts::PI;
        match n {
            0 => PI,
            _ => unreachable!(),
        }
    }
    fn step_val(&self, n: usize) -> f32 {
        match n {
            0 => 1f32.to_radians(),
            _ => unreachable!(),
        }
    }
    fn name_val(&self, n: usize) -> String {
        match n {
            0 => String::from("Roll helix"),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct RigidBodyParametersRequest {
    pub k_springs: f32,
    pub k_friction: f32,
    pub mass_factor: f32,
    pub volume_exclusion: bool,
    pub brownian_motion: bool,
    pub brownian_rate: f32,
    pub brownian_amplitude: f32,
}

struct RigidBodyFactory {
    pub volume_exclusion: bool,
    pub brownian_motion: bool,
    pub brownian_parameters: BrownianParametersFactory,
}

#[derive(Clone)]
struct BrownianParametersFactory {
    pub rate: f32,
    pub amplitude: f32,
}

impl Requestable for BrownianParametersFactory {
    type Request = Self;
    fn request_from_values(&self, values: &[f32]) -> Self {
        Self {
            rate: values[0],
            amplitude: values[1],
        }
    }

    fn nb_values(&self) -> usize {
        2
    }

    fn initial_value(&self, n: usize) -> f32 {
        match n {
            0 => 0.,
            1 => 0.08,
            _ => unreachable!(),
        }
    }

    fn min_val(&self, n: usize) -> f32 {
        match n {
            0 => -2.,
            1 => 0.,
            _ => unreachable!(),
        }
    }

    fn max_val(&self, n: usize) -> f32 {
        match n {
            0 => 2.,
            1 => 0.2,
            _ => unreachable!(),
        }
    }

    fn step_val(&self, n: usize) -> f32 {
        match n {
            0 => 0.1,
            1 => 0.02,
            _ => unreachable!(),
        }
    }

    fn name_val(&self, n: usize) -> String {
        match n {
            0 => "Rate (log scale)".to_owned(),
            1 => "Range".to_owned(),
            _ => unreachable!(),
        }
    }
}

impl Requestable for RigidBodyFactory {
    type Request = RigidBodyParametersRequest;
    fn request_from_values(&self, values: &[f32]) -> RigidBodyParametersRequest {
        RigidBodyParametersRequest {
            k_springs: values[0],
            k_friction: values[1],
            mass_factor: values[2],
            volume_exclusion: self.volume_exclusion,
            brownian_motion: self.brownian_motion,
            brownian_rate: self.brownian_parameters.rate,
            brownian_amplitude: self.brownian_parameters.amplitude,
        }
    }
    fn nb_values(&self) -> usize {
        3
    }
    fn initial_value(&self, n: usize) -> f32 {
        match n {
            0 => 0f32,
            1 => 0f32,
            2 => 0f32,
            _ => unreachable!(),
        }
    }
    fn min_val(&self, n: usize) -> f32 {
        match n {
            0 => -4.,
            1 => -4.,
            2 => -4.,
            3 => -4.,
            _ => unreachable!(),
        }
    }
    fn max_val(&self, n: usize) -> f32 {
        match n {
            0 => 4.,
            1 => 4.,
            2 => 4.,
            3 => 4.,
            _ => unreachable!(),
        }
    }
    fn step_val(&self, n: usize) -> f32 {
        match n {
            0 => 0.1f32,
            1 => 0.1f32,
            2 => 0.1f32,
            3 => 0.1f32,
            _ => unreachable!(),
        }
    }
    fn name_val(&self, n: usize) -> String {
        match n {
            0 => String::from("Stiffness (log scale)"),
            1 => String::from("Friction (log scale)"),
            2 => String::from("Mass (log scale)"),
            _ => unreachable!(),
        }
    }
}

/// Encodes a color into a u32.
fn color_to_u32(color: Color) -> u32 {
    let red = ((color.r * 255.) as u32) << 16;
    let green = ((color.g * 255.) as u32) << 8;
    let blue = (color.b * 255.) as u32;

    red + green + blue
}
