use std::path::PathBuf;

use crate::{
    CameraId,
    bezier_plane::{
        BezierPathId, BezierPlaneDescriptor, BezierPlaneId, BezierVertex, BezierVertexId,
    },
    curves::bezier::BezierControlPoint,
    elements::{DesignElementKey, DnaAttribute},
    grid::{
        GridDescriptor, GridId, GridObject, GridTypeDescr, HelixGridPosition,
        hyperboloid::Hyperboloid,
    },
    group_attributes::GroupPivot,
    nucl::Nucl,
    organizer::tree::{GroupId, OrganizerTree},
    parameters::HelixParameters,
    selection::Selection,
};
use ultraviolet::{Isometry2, Rotor3, Vec2, Vec3};

/// An operation that can be performed on a design
#[derive(Debug, Clone)]
pub enum DesignOperation {
    /// Rotate an element of the design
    Rotation(DesignRotation),
    /// Translate an element of the design
    Translation(DesignTranslation),
    /// Add an helix on a grid
    AddGridHelix {
        position: HelixGridPosition,
        start: isize,
        length: usize,
    },
    AddTwoPointsBezier {
        start: HelixGridPosition,
        end: HelixGridPosition,
    },
    RmHelices {
        h_ids: Vec<usize>,
    },
    RmXovers {
        xovers: Vec<(Nucl, Nucl)>,
    },
    /// Split a strand at a given position. If the strand containing the nucleotide has length 1,
    /// delete the strand.
    Cut {
        nucl: Nucl,
    },
    /// Make a cross-over between two nucleotides, splitting the source and target strands if needed
    GeneralXover {
        source: Nucl,
        target: Nucl,
    },
    /// Merge two strands by making a cross-over between the 3'end of prime_5 and the 5'end of
    /// prime_3
    Xover {
        prime5_id: usize,
        prime3_id: usize,
    },
    /// Make a cross over from a strand end to a nucleotide, splitting the target strand if needed.
    CrossCut {
        target_3prime: bool,
        source_id: usize,
        target_id: usize,
        nucl: Nucl,
    },
    /// Delete a strand
    RmStrands {
        strand_ids: Vec<usize>,
    },
    /// Add a grid to the design
    AddGrid(GridDescriptor),
    /// Pick a new color at random for all the strands that are not the scaffold
    RecolorStaples,
    /// Change the color of a set of strands
    ChangeColor {
        color: u32,
        strands: Vec<usize>,
    },
    /// Set the strand with a given id as the scaffold
    SetScaffoldId(Option<usize>),
    /// Change the shift of the scaffold without changing the sequence
    SetScaffoldShift(usize),
    /// Change the sequence and the shift of the scaffold
    SetScaffoldSequence {
        sequence: String,
        shift: usize,
    },
    HyperboloidOperation(HyperboloidOperation),
    CleanDesign,
    HelicesToGrid(Vec<Selection>),
    SetHelicesPersistence {
        grid_ids: Vec<GridId>,
        persistent: bool,
    },
    UpdateAttribute {
        attribute: DnaAttribute,
        elements: Vec<DesignElementKey>,
    },
    SetSmallSpheres {
        grid_ids: Vec<GridId>,
        small: bool,
    },
    /// Apply a translation to the 2d representation of helices holding each pivot
    SnapHelices {
        pivots: Vec<(Nucl, usize)>,
        translation: Vec2,
    },
    RotateHelices {
        helices: Vec<usize>,
        center: Vec2,
        angle: f32,
    },
    ApplySymmetryToHelices {
        helices: Vec<usize>,
        centers: Vec<Vec2>,
        symmetry: Vec2,
    },
    SetIsometry {
        helix: usize,
        segment_idx: usize,
        isometry: Isometry2,
    },
    RequestStrandBuilders {
        nucls: Vec<Nucl>,
    },
    MoveBuilders(isize),
    SetRollHelices {
        helices: Vec<usize>,
        roll: f32,
    },
    SetVisibilityHelix {
        helix: usize,
        visible: bool,
    },
    FlipHelixGroup {
        helix: usize,
    },
    FlipAnchors {
        nucls: Vec<Nucl>,
    },
    AttachObject {
        object: GridObject,
        grid: GridId,
        x: isize,
        y: isize,
    },
    SetOrganizerTree(OrganizerTree<DesignElementKey>),
    SetStrandName {
        s_id: usize,
        name: String,
    },
    SetGroupPivot {
        group_id: GroupId,
        pivot: GroupPivot,
    },
    DeleteCamera(CameraId),
    CreateNewCamera {
        position: Vec3,
        orientation: Rotor3,
        pivot_position: Option<Vec3>,
    },
    SetCameraName {
        camera_id: CameraId,
        name: String,
    },
    SetGridPosition {
        grid_id: GridId,
        position: Vec3,
    },
    SetGridOrientation {
        grid_id: GridId,
        orientation: Rotor3,
    },
    SetGridNbTurn {
        grid_id: GridId,
        nb_turn: f32,
    },
    MakeSeveralXovers {
        xovers: Vec<(Nucl, Nucl)>,
        doubled: bool,
    },
    CheckXovers {
        xovers: Vec<usize>,
    },
    SetRainbowScaffold(bool),
    SetGlobalHelixParameters {
        helix_parameters: HelixParameters,
    },
    SetInsertionLength {
        length: usize,
        insertion_point: InsertionPoint,
    },
    AddBezierPlane {
        desc: BezierPlaneDescriptor,
    },
    CreateBezierPath {
        first_vertex: BezierVertex,
    },
    AppendVertexToPath {
        path_id: BezierPathId,
        vertex: BezierVertex,
    },
    /// Move the first vertex to `position` and apply the same translation to the other vertices
    MoveBezierVertex {
        vertices: Vec<BezierVertexId>,
        position: Vec2,
    },
    SetBezierVertexPosition {
        vertex_id: BezierVertexId,
        position: Vec2,
    },
    TurnPathVerticesIntoGrid {
        path_id: BezierPathId,
        grid_type: GridTypeDescr,
    },
    ApplyHomothethyOnBezierPlane {
        homothethy: BezierPlaneHomothethy,
    },
    SetVectorOfBezierTangent(NewBezierTangentVector),
    MakeBezierPathCyclic {
        path_id: BezierPathId,
        cyclic: bool,
    },
    RmFreeGrids {
        grid_ids: Vec<usize>,
    },
    RmBezierVertices {
        vertices: Vec<BezierVertexId>,
    },
    Add3DObject {
        file_path: PathBuf,
        design_path: PathBuf,
    },
    ImportSvgPath {
        path: PathBuf,
    },
}

impl DesignOperation {
    pub fn label(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Rotation(rotation) => format!("Rotation of {}", rotation.target).into(),
            Self::Translation(translation) => {
                format!("Translation of {}", translation.target).into()
            }
            Self::AddGridHelix { .. } => "Helix creation".into(),
            Self::AddTwoPointsBezier { .. } => "Bezier curve creation".into(),
            Self::RmHelices { .. } => "Helix deletion".into(),
            Self::RmXovers { .. } => "Xover deletion".into(),
            Self::Cut { nucl, .. } => format!("Cut on {nucl:?}").into(),
            Self::GeneralXover { source, target } => {
                format!("Xover between {source:?} and {target:?}").into()
            }
            Self::Xover { .. } => "Xover".into(),
            Self::CrossCut { .. } => "Cut and crossover".into(),
            Self::RmStrands { .. } => "Strand deletion".into(),
            Self::AddGrid(_) => "Grid creation".into(),
            Self::RecolorStaples => "Staple recoloring".into(),
            Self::ChangeColor { .. } => "Color modification".into(),
            Self::SetScaffoldId(_) => "Scaffold setting".into(),
            Self::SetScaffoldSequence { .. } => "Scaffold sequence setting".into(),
            Self::HyperboloidOperation(_) => "Nanotube operation".into(),
            Self::CleanDesign => "Clean design".into(),
            Self::HelicesToGrid(_) => "Grid creation from helices".into(),
            Self::SetHelicesPersistence {
                persistent: true, ..
            } => "Show phantom helices".into(),
            Self::SetHelicesPersistence {
                persistent: false, ..
            } => "Hide phantom helices".into(),
            Self::UpdateAttribute { .. } => "Update attribute from organizer".into(),
            Self::SetSmallSpheres { small: true, .. } => "Hide nucleotides".into(),
            Self::SetSmallSpheres { small: false, .. } => "Show nucleotides".into(),
            Self::SnapHelices { .. } => "Move 2D helices".into(),
            Self::RotateHelices { .. } => "Translate 2D helices".into(),
            Self::SetIsometry { .. } => "Set isometry of helices".into(),
            Self::RequestStrandBuilders { nucls } => format!("Build on {nucls:?}").into(),
            Self::MoveBuilders(_) => "Move builders".into(),
            Self::SetRollHelices { .. } => "Set roll of helix".into(),
            Self::SetVisibilityHelix { visible: true, .. } => "Make helices visible".into(),
            Self::SetVisibilityHelix { visible: false, .. } => "Make helices invisible".into(),
            Self::FlipHelixGroup { .. } => "Change xover group of helices".into(),
            Self::FlipAnchors { .. } => "Set/Unset nucl anchor".into(),
            Self::AttachObject { .. } => "Move grid object".into(),
            Self::SetOrganizerTree(_) => "Update organizer tree".into(),
            Self::SetStrandName { .. } => "Update name of strand".into(),
            Self::SetGroupPivot { .. } => "Set group pivot".into(),
            Self::DeleteCamera(_) => "Delete camera".into(),
            Self::CreateNewCamera { .. } => "Create camera shortcut".into(),
            Self::SetGridPosition { .. } => "Set grid position".into(),
            Self::SetGridOrientation { .. } => "Set grid orientation".into(),
            Self::MakeSeveralXovers { .. } => "Multiple xovers".into(),
            _ => "Unnamed operation".into(),
        }
    }
}

/// A rotation on an element of a design.
#[derive(Debug, Clone)]
pub struct DesignRotation {
    pub origin: Vec3,
    pub rotation: Rotor3,
    /// The element of the design on which the rotation will be applied
    pub target: IsometryTarget,
    pub group_id: Option<GroupId>,
}

/// A translation of an element of a design
#[derive(Clone, Debug)]
pub struct DesignTranslation {
    pub translation: Vec3,
    pub target: IsometryTarget,
    pub group_id: Option<GroupId>,
}

#[derive(Debug, Clone)]
pub enum HyperboloidOperation {
    New {
        request: HyperboloidRequest,
        position: Vec3,
        orientation: Rotor3,
    },
    Update(HyperboloidRequest),
    Finalize,
    Cancel,
}

#[derive(Debug, Clone, Copy)]
pub struct HyperboloidRequest {
    pub radius: usize,
    pub length: f32,
    pub shift: f32,
    pub radius_shift: f32,
    pub nb_turn: f64,
}

impl HyperboloidRequest {
    pub fn to_grid(self) -> Hyperboloid {
        Hyperboloid {
            radius: self.radius,
            length: self.length,
            shift: self.shift,
            radius_shift: self.radius_shift,
            forced_radius: None,
            nb_turn_per_100_nt: self.nb_turn,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct InsertionPoint {
    pub nucl: Nucl,
    pub nucl_is_prime5_of_insertion: bool,
}

#[derive(Debug, Clone)]
pub struct BezierPlaneHomothethy {
    pub plane_id: BezierPlaneId,
    pub fixed_corner: Vec2,
    pub origin_moving_corner: Vec2,
    pub moving_corner: Vec2,
}

/// A element on which an isometry must be applied
#[derive(Clone, Debug)]
pub enum IsometryTarget {
    /// An helix of the design
    Helices(Vec<usize>, bool),
    /// A grid of the design
    Grids(Vec<GridId>),
    /// The pivot of a group
    GroupPivot(GroupId),
    /// The control points of bezier curves
    ControlPoint(Vec<(usize, BezierControlPoint)>),
}

#[derive(Clone, Debug, Copy)]
pub struct NewBezierTangentVector {
    pub vertex_id: BezierVertexId,
    /// Whether `new_vector` is the vector of the inward or outward tangent
    pub tangent_in: bool,
    pub full_symmetry_other_tangent: bool,
    pub new_vector: Vec2,
}

impl std::fmt::Display for IsometryTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Helices(hs, _) => write!(f, "Helices {hs:?}"),
            Self::Grids(gs) => write!(f, "Grids {gs:?}"),
            Self::GroupPivot(_) => write!(f, "Group pivot"),
            Self::ControlPoint(_) => write!(f, "Bezier control point"),
        }
    }
}
