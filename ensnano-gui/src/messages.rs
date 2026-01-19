use std::collections::{BTreeMap, VecDeque};
use ultraviolet::{Rotor3, Vec2};

use ensnano_design::{
    CameraId,
    bezier_plane::BezierPathId,
    design_element::{DesignElement, DesignElementKey, DnaAttribute},
    grid::GridTypeDescr,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::{GroupId, OrganizerNodeId, OrganizerTree},
    parameters::NamedParameter,
};
use ensnano_exports::ExportType;
use ensnano_physics::parameters::RapierParameters;
use ensnano_utils::{
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    graphics::{Background3D, HBondDisplay, RenderingMode, SplitMode},
    keyboard_priority::PriorityRequest,
    surfaces::EquadiffSolvingMethod,
    ui_size::UiSize,
};
use iced::widget::text_input::Id;
use ultraviolet::Vec3;
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::Modifiers,
};

use crate::{
    color_picker::ColorPickerMessage,
    drag_drop_target::DragIdentifier,
    left_panel::tabs::{
        TabId,
        camera_tab::FogChoices,
        revolution_tab::{CurveDescriptorBuilder, RevolutionParameterId},
    },
    state::GuiAppState,
};

/// Some main application state, mostly related with top bar buttons.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TopBarStateFlags {
    /// Whether the Undo operation is possible.
    pub can_undo: bool,
    /// Whether the Redo operation is possible.
    pub can_redo: bool,
    pub need_save: bool,
    pub can_reload: bool,
    pub can_split_2d: bool,
    pub can_toggle_2d: bool,
    pub is_split_2d: bool,
}

/// Message sent to the gui component
pub struct GuiMessages<S: GuiAppState> {
    pub left_panel: VecDeque<LeftPanelMessage<S>>,
    pub top_bar: VecDeque<TopBarMessage<S>>,
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
            .push_back(LeftPanelMessage::ModifiersChanged(modifiers));
    }

    pub fn new_ui_size(&mut self, ui_size: UiSize) {
        self.left_panel
            .push_back(LeftPanelMessage::UiSizeChanged(ui_size));
        self.top_bar
            .push_back(TopBarMessage::UiSizeChanged(ui_size));
        self.status_bar
            .push_back(StatusBarMessage::UiSizeChanged(ui_size));
    }

    pub fn push_show_tutorial(&mut self) {
        self.left_panel.push_back(LeftPanelMessage::ShowTutorial);
    }

    pub fn show_help(&mut self) {
        self.left_panel.push_back(LeftPanelMessage::ForceHelp);
    }

    pub fn push_application_state(&mut self, state: S, top_bar_state: TopBarStateFlags) {
        log::trace!("Old ptr {:p}, new ptr {:p}", state, self.application_state);
        self.application_state = state.clone();
        self.redraw |= top_bar_state != self.last_top_bar_state;
        self.last_top_bar_state = top_bar_state.clone();
        let must_update = self.application_state != state || self.redraw;
        if must_update {
            self.left_panel
                .push_back(LeftPanelMessage::NewApplicationState(state.clone()));
            self.top_bar.push_back(TopBarMessage::NewApplicationState((
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

#[derive(Debug, Clone)]
pub enum LeftPanelMessage<S: GuiAppState> {
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
    /// Changes rapier parameters, including
    /// if a simulation is running.
    UpdateRapierParameters(RapierParameters),
    UpdateRapierParameterField(String, String),
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
    OrganizerMessage(OrganizerMessage),
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
    FogChoice(FogChoices),
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
    SetKeyboardPriority(PriorityRequest),
    SetFocus(Id),
}

/// Public messages generated by an Organizer.
#[derive(Clone, Debug)]
pub enum OrganizerMessage {
    InternalMessage(OrganizerInternalMessage),
    Selection(Vec<DesignElementKey>, Option<GroupId>),
    Candidates(Vec<DesignElementKey>),
    ElementUpdate(Vec<BTreeMap<DesignElementKey, DesignElement>>),
    NewAttribute(DnaAttribute, Vec<DesignElementKey>),
    /// Publishing a new organizer tree.
    NewTree(OrganizerTree),
    /// A new group is created.
    NewGroup {
        group_id: GroupId,
        elements_selected: Vec<DesignElementKey>,
        new_tree: OrganizerTree,
    },
    /// Taking or releasing keyboard priority.
    SetKeyboardPriority(PriorityRequest),
    /// Ask Iced application to focus on this element
    SetFocus(Id),
}

#[derive(Clone, Debug)]
pub enum OrganizerInternalMessage {
    Expand {
        id: OrganizerNodeId,
        expanded: bool,
    },
    NodeSelected {
        id: OrganizerNodeId,
    },
    NodeHovered {
        id: OrganizerNodeId,
        hovered_in: bool,
    },
    KeyHovered {
        key: DesignElementKey,
        hovered_in: bool,
    },
    ElementSelected {
        key: DesignElementKey,
    },
    Edit {
        id: OrganizerNodeId,
    },
    StopEdit,
    NameInput {
        name: String,
    },
    /// Create a new group.
    NewGroup,
    AddSelectionToGroup {
        id: OrganizerNodeId,
    },
    Delete {
        id: OrganizerNodeId,
    },
    DragDropped(DragIdentifier),
    Dragging(DragIdentifier),
    AttributeSelected {
        attribute: DnaAttribute,
        id: OrganizerNodeId,
    },
}

/// Shorthands to send internal messages.
impl OrganizerMessage {
    pub(super) fn expand(id: OrganizerNodeId, expanded: bool) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Expand { id, expanded })
    }

    pub(super) fn node_selected(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NodeSelected { id })
    }

    pub(super) fn node_hovered(id: OrganizerNodeId, hovered_in: bool) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NodeHovered { id, hovered_in })
    }

    pub(super) fn key_hovered(key: DesignElementKey, hovered_in: bool) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::KeyHovered { key, hovered_in })
    }

    pub(super) fn edit(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Edit { id })
    }

    pub(super) fn delete(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Delete { id })
    }

    pub(super) fn name_input(name: String) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NameInput { name })
    }

    pub(super) fn stop_edit() -> Self {
        Self::InternalMessage(OrganizerInternalMessage::StopEdit)
    }

    pub(super) fn element_selected(key: DesignElementKey) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::ElementSelected { key })
    }

    pub(super) fn add_selection_to_group(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::AddSelectionToGroup { id })
    }

    pub(super) fn new_group() -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NewGroup)
    }

    pub(super) fn dragging(key: DragIdentifier) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Dragging(key))
    }

    pub(super) fn drag_dropped(key: DragIdentifier) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::DragDropped(key))
    }

    pub(super) fn attribute_selected(attribute: DnaAttribute, id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::AttributeSelected { attribute, id })
    }
}

#[derive(Debug, Clone)]
pub enum TopBarMessage<S: GuiAppState> {
    SceneFitRequested,
    AlignHorizon,
    OpenFileButtonPressed,
    /// Request to save file, e.g. clicked on “Save” button
    FileSaveRequested,
    /// Request to save file as, e.g. clicked on “Save As” button
    SaveAsRequested,
    Resize(LogicalSize<f64>),
    ToggleView(SplitMode),
    UiSizeChanged(UiSize),
    ExportRequested,
    Split2D,
    // Receive an new application state.
    NewApplicationState((S, TopBarStateFlags)),
    ForceHelp,
    ShowTutorial,
    Undo,
    Redo,
    ButtonNewEmptyDesignPressed,
    ActionModeChanged(ActionMode),
    SelectionModeChanged(SelectionMode),
    Toggle2D,
    Reload,
    FlipSplitViews,
    ThickHelices(bool),
    Import3D,
}

#[derive(Clone, Copy, Debug)]
pub enum ValueKind {
    HelixGridPosition,
    GridOrientation,
    BezierVertexPosition,
}

#[derive(Debug, Clone)]
pub enum InstantiatedValue {
    HelixGridPosition(Vec3),
    GridOrientation(Rotor3),
    GridNbTurn(f32),
    BezierVertexPosition(Vec2),
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum FactoryId {
    HelixRoll,
    Hyperboloid,
    Scroll,
    RigidBody,
    Brownian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueId(pub usize);
