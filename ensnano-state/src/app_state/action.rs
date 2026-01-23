use crate::{
    app_state::design_interactor::controller::clipboard::PastePosition,
    design::operation::{DesignOperation, HyperboloidRequest},
    utils::application::Notification,
};
use ensnano_design::{
    CameraId,
    grid::{GridId, GridTypeDescr},
    group_attributes::GroupPivot,
    parameters::HelixParameters,
};
use ensnano_exports::ExportType;
use ensnano_physics::parameters::RapierParameters;
use ensnano_utils::{
    RigidBodyConstants, RollRequest,
    graphics::{FogParameters, SplitMode},
    overlay::OverlayType,
    surfaces::RevolutionSurfaceSystemDescriptor,
    ui_size::UiSize,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use ultraviolet::{Rotor3, Vec3};

/// An action to be performed at the end of an event loop iteration, and that will have an effect
/// on the main application state, e.g. Closing the window, or toggling between 3D/2D views.
#[derive(Debug, Clone)]
pub enum Action {
    LoadDesign(Option<PathBuf>),
    NewDesign,
    SaveAs,
    QuickSave,
    DownloadStaplesRequest,
    DownloadOrigamiRequest,
    /// Trigger the sequence of action that will set the scaffold of the sequence.
    SetScaffoldSequence {
        shift: usize,
    },
    Exit,
    ToggleSplit(SplitMode),
    Export(ExportType),
    CloseOverlay(OverlayType),
    ChangeUiSize(UiSize),
    ErrorMsg(String),
    DesignOperation(DesignOperation),
    SilentDesignOperation(DesignOperation),
    Undo,
    Redo,
    NotifyApps(Notification),
    TurnSelectionIntoGrid,
    AddGrid(GridTypeDescr),
    /// Change the color of all the selected strands
    ChangeColorStrand(u32),
    FinishChangingColor,
    ToggleHelicesPersistence(bool),
    ToggleSmallSphere(bool),
    RollRequest(RollRequest),
    UpdateRapierParameters(RapierParameters),
    StopSimulation,
    RollHelices(f32),
    Copy,
    PasteCandidate(Option<PastePosition>),
    InitPaste,
    ApplyPaste,
    Duplicate,
    RigidGridSimulation {
        parameters: RigidBodyConstants,
    },
    RevolutionSimulation {
        desc: RevolutionSurfaceSystemDescriptor,
    },
    FinishRelaxationSimulation,
    RigidHelicesSimulation {
        parameters: RigidBodyConstants,
    },
    ResetSimulation,
    RigidParametersUpdate(RigidBodyConstants),
    TurnIntoAnchor,
    NewHyperboloid(HyperboloidRequest),
    SetVisibilitySieve {
        compl: bool,
    },
    DeleteSelection,
    ScaffoldToSelection,
    /// Save the nucleotides 3D positions by strand as a json file in the design directory
    GetDesignPathAndNotify(fn(Option<Arc<Path>>) -> Notification),
    SuspendOp,
    Fog(FogParameters),
    Split2D,
    ReloadFile,
    ClearVisibilitySieve,
    SetGroupPivot(GroupPivot),
    TranslateGroupPivot(Vec3),
    RotateGroupPivot(Rotor3),
    NewCamera,
    SelectCamera(CameraId),
    SelectFavoriteCamera(u32),
    Toggle2D,
    MakeAllSuggestedXover {
        doubled: bool,
    },
    FlipSplitViews,
    Twist(GridId),
    SetDnaParameters(HelixParameters),
    SetExpandInsertions(bool),
    AddBezierPlane,
    SetExporting(bool),
    Import3DObject,
    ImportSvg,
    OptimizeShift,
}
