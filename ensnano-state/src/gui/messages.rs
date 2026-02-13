use crate::{
    app_state::AppState,
    gui::{
        curve::CurveDescriptorBuilder, drag_drop_target::DragIdentifier,
        state::RevolutionParameterId,
    },
};
use ensnano_design::{
    CameraId,
    bezier_plane::BezierPathId,
    design_element::{DesignElement, DesignElementKey, DnaAttribute},
    grid::GridTypeDescr,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::{GroupId, OrganizerNodeId, OrganizerTree},
    parameters::NamedParameter,
};
use ensnano_physics::parameters::RapierParameters;
use ensnano_utils::{
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    export::ExportType,
    graphics::{Background3D, HBondDisplay, RenderingMode, SplitMode, fog_kind},
    keyboard_priority::PriorityRequest,
    surfaces::EquadiffSolvingMethod,
    ui_size::UiSize,
};
use iced::{Color, widget::text_input::Id};
use std::collections::{BTreeMap, VecDeque};
use ultraviolet::{Rotor3, Vec2, Vec3};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::Modifiers,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FogChoices {
    #[default]
    None,
    FromCamera,
    FromPivot,
    DarkFromCamera,
    DarkFromPivot,
    ReversedFromPivot,
}

pub const ALL_FOG_CHOICES: &[FogChoices] = &[
    FogChoices::None,
    FogChoices::FromCamera,
    FogChoices::FromPivot,
    FogChoices::DarkFromCamera,
    FogChoices::DarkFromPivot,
    FogChoices::ReversedFromPivot,
];

impl std::fmt::Display for FogChoices {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ret = match self {
            Self::None => "None",
            Self::FromCamera => "From Camera",
            Self::FromPivot => "From Pivot",
            Self::DarkFromCamera => "Dark from Camera",
            Self::DarkFromPivot => "Dark from Pivot",
            Self::ReversedFromPivot => "Reversed from Pivot",
        };
        write!(f, "{ret}")
    }
}

impl FogChoices {
    pub fn from_param(visible: bool, from_camera: bool, dark: bool, reversed: bool) -> Self {
        Self::None
            .visible(visible)
            .dark(dark)
            .from_camera(from_camera)
            .reversed(reversed)
    }

    pub fn to_param(self) -> (bool, bool, bool, bool) {
        (
            self.is_visible(),
            self.is_from_camera(),
            self.is_dark(),
            self.is_reversed(),
        )
    }

    #[must_use]
    pub fn visible(self, visible: bool) -> Self {
        if visible {
            if self == Self::None {
                Self::FromPivot
            } else {
                self
            }
        } else {
            Self::None
        }
    }

    #[must_use]
    pub fn from_camera(self, from_camera: bool) -> Self {
        if from_camera {
            match self {
                Self::FromPivot => Self::FromCamera,
                Self::DarkFromPivot => Self::DarkFromCamera,
                _ => self,
            }
        } else {
            match self {
                Self::FromCamera => Self::FromPivot,
                Self::DarkFromCamera => Self::DarkFromPivot,
                _ => self,
            }
        }
    }

    #[must_use]
    pub fn reversed(self, reversed: bool) -> Self {
        match (self, reversed) {
            (Self::FromPivot, true) => Self::ReversedFromPivot,
            (Self::ReversedFromPivot, false) => Self::FromPivot,
            _ => self,
        }
    }

    #[must_use]
    pub fn dark(self, dark: bool) -> Self {
        if dark {
            match self {
                Self::FromCamera => Self::DarkFromCamera,
                Self::FromPivot => Self::DarkFromPivot,
                _ => self,
            }
        } else {
            match self {
                Self::DarkFromCamera => Self::FromCamera,
                Self::DarkFromPivot => Self::FromPivot,
                _ => self,
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn is_from_camera(&self) -> bool {
        matches!(self, Self::FromCamera | Self::DarkFromCamera)
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Self::DarkFromCamera | Self::DarkFromPivot)
    }

    pub fn is_reversed(&self) -> bool {
        matches!(self, Self::ReversedFromPivot)
    }

    pub fn fog_kind(&self) -> u32 {
        match self {
            Self::None => fog_kind::NO_FOG,
            Self::FromCamera | Self::FromPivot => fog_kind::TRANSPARENT_FOG,
            Self::DarkFromPivot | Self::DarkFromCamera => fog_kind::DARK_FOG,
            Self::ReversedFromPivot => fog_kind::REVERSED_FOG,
        }
    }
}

/// Messages from ColorPicker.
#[derive(Debug, Clone, Copy)]
pub enum ColorPickerMessage {
    HueChanged(f64),
    HsvSatValueChanged(f64, f64),
    ColorPicked(Color),
    FinishChangingColor,
}

/// Message sent to the gui component.
pub struct GuiMessages {
    pub left_panel: VecDeque<LeftPanelMessage>,
    pub top_bar: VecDeque<TopBarMessage>,
    pub status_bar: VecDeque<StatusBarMessage>,
    pub application_state: AppState,
    pub last_top_bar_state: TopBarStateFlags,
    pub redraw: bool,
}

impl GuiMessages {
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

    pub fn push_application_state(&mut self, state: AppState, top_bar_state: TopBarStateFlags) {
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
pub enum StatusBarMessage {
    ValueStrChanged(usize, String),
    ValueSet(usize, String),
    Progress(Option<(String, f32)>),
    NewApplicationState(AppState),
    UiSizeChanged(UiSize),
    TabPressed,
    Message(Option<String>),
    Resize(LogicalSize<f64>),
    SetKeyboardPriority(PriorityRequest),
}

#[derive(Debug, Clone)]
pub enum LeftPanelMessage {
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
    /// Changes rapier parameters, including if a simulation is running.
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
    NewApplicationState(AppState),
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
    CurveBuilderPicked(CurveDescriptorBuilder),
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
    /// Ask Iced application to focus on this element.
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
    pub fn expand(id: OrganizerNodeId, expanded: bool) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Expand { id, expanded })
    }

    pub fn node_selected(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NodeSelected { id })
    }

    pub fn node_hovered(id: OrganizerNodeId, hovered_in: bool) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NodeHovered { id, hovered_in })
    }

    pub fn key_hovered(key: DesignElementKey, hovered_in: bool) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::KeyHovered { key, hovered_in })
    }

    pub fn edit(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Edit { id })
    }

    pub fn delete(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Delete { id })
    }

    pub fn name_input(name: String) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NameInput { name })
    }

    pub fn stop_edit() -> Self {
        Self::InternalMessage(OrganizerInternalMessage::StopEdit)
    }

    pub fn element_selected(key: DesignElementKey) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::ElementSelected { key })
    }

    pub fn add_selection_to_group(id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::AddSelectionToGroup { id })
    }

    pub fn new_group() -> Self {
        Self::InternalMessage(OrganizerInternalMessage::NewGroup)
    }

    pub fn dragging(key: DragIdentifier) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::Dragging(key))
    }

    pub fn drag_dropped(key: DragIdentifier) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::DragDropped(key))
    }

    pub fn attribute_selected(attribute: DnaAttribute, id: OrganizerNodeId) -> Self {
        Self::InternalMessage(OrganizerInternalMessage::AttributeSelected { attribute, id })
    }
}

#[derive(Debug, Clone)]
pub enum TopBarMessage {
    SceneFitRequested,
    AlignHorizon,
    OpenFileButtonPressed,
    /// Request to save file, e.g. clicked on “Save” button.
    FileSaveRequested,
    /// Request to save file as, e.g. clicked on “Save As” button.
    SaveAsRequested,
    Resize(LogicalSize<f64>),
    ToggleView(SplitMode),
    UiSizeChanged(UiSize),
    ExportRequested,
    Split2D,
    // Receive an new application state.
    NewApplicationState((AppState, TopBarStateFlags)),
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TabId {
    Grid,
    Edition,
    Camera,
    Simulation,
    Sequence,
    Parameters,
    Pen,
    Revolution,
}
