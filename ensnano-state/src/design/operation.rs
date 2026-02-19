use crate::{
    app_state::{
        AppState,
        design_interactor::controller::{Controller, OperationError},
    },
    design::selection::Selection,
    operation::{AppStateOperationOutcome, AppStateOperationResult},
};
use ensnano_design::{
    CameraId, Design,
    bezier_plane::{
        BezierPathId, BezierPlaneDescriptor, BezierPlaneId, BezierVertex, BezierVertexId,
    },
    curves::bezier::BezierControlPoint,
    design_element::{DesignElementKey, DnaAttribute},
    grid::{
        GridDescriptor, GridId, GridObject, GridTypeDescr, HelixGridPosition,
        hyperboloid::Hyperboloid,
    },
    group_attributes::GroupPivot,
    nucl::Nucl,
    organizer_tree::{GroupId, OrganizerTree},
    parameters::HelixParameters,
};
use std::{path::PathBuf, sync::Arc};
use ultraviolet::{Isometry2, Rotor3, Vec2, Vec3};

/// An operation that can be performed on a design.
#[derive(Debug, Clone)]
pub enum DesignOperation {
    /// Rotate an element of the design.
    Rotation(DesignRotation),
    /// Translate an element of the design.
    Translation(DesignTranslation),
    /// Add an helix on a grid.
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
    /// Make a cross-over between two nucleotides, splitting the source and target strands if needed.
    GeneralXover {
        source: Nucl,
        target: Nucl,
    },
    /// Merge two strands by making a cross-over between the 3'end of prime_5 and the 5'end of
    /// prime_3.
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
    /// Delete a strand.
    RmStrands {
        strand_ids: Vec<usize>,
    },
    /// Add a grid to the design.
    AddGrid(GridDescriptor),
    /// Pick a new color at random for all the strands that are not the scaffold.
    RecolorStaples,
    /// Change the color of a set of strands.
    ChangeColor {
        color: u32,
        strands: Vec<usize>,
    },
    /// Set the strand with a given id as the scaffold.
    SetScaffoldId(Option<usize>),
    /// Change the shift of the scaffold without changing the sequence.
    SetScaffoldShift(usize),
    /// Change the sequence and the shift of the scaffold.
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
    /// Apply a translation to the 2d representation of helices holding each pivot.
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
    SetOrganizerTree(OrganizerTree),
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
    /// Move the first vertex to `position` and apply the same translation to the other vertices.
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

    fn outcome(&self) -> AppStateOperationOutcome {
        let label = self.label();

        AppStateOperationOutcome::Push { label }
    }

    pub fn apply(self, state: &mut AppState) -> AppStateOperationResult {
        let outcome = self.outcome();

        match self {
            Self::RecolorStaples => {
                Controller::fancy_recolor_staples(state);
            }
            Self::SetScaffoldSequence { sequence, shift } => {
                Controller::set_scaffold_sequence(state, sequence, shift);
            }
            Self::SetScaffoldShift(shift) => {
                Controller::set_scaffold_shift(state, shift);
            }
            Self::HelicesToGrid(selection) => {
                Controller::turn_selection_into_grid(state, selection)?;
            }
            Self::AddGrid(descriptor) => {
                Controller::add_grid(state, descriptor);
            }
            Self::ChangeColor { color, strands } => {
                Controller::change_color_strands(state, color, strands);
            }
            Self::SetHelicesPersistence {
                grid_ids,
                persistent,
            } => {
                Controller::set_helices_persistence(state, grid_ids, persistent);
            }
            Self::SetSmallSpheres { grid_ids, small } => {
                Controller::set_small_spheres(state, grid_ids, small);
            }
            Self::SnapHelices {
                pivots,
                translation,
            } => {
                Controller::snap_helices(state, pivots, translation);
            }
            Self::SetIsometry {
                helix,
                segment_idx,
                isometry,
            } => {
                Controller::set_isometry(state, helix, segment_idx, isometry);
            }
            Self::RotateHelices {
                helices,
                center,
                angle,
            } => {
                Controller::rotate_helices(state, helices, center, angle);
            }
            Self::ApplySymmetryToHelices {
                helices,
                centers,
                symmetry,
            } => {
                Controller::apply_symmetry_to_helices(state, helices, centers, symmetry);
            }
            Self::Translation(translation) => {
                Controller::apply_translation(state, translation)?;
            }
            Self::Rotation(rotation) => {
                Controller::apply_rotation(state, rotation)?;
            }
            Self::RequestStrandBuilders { nucls } => {
                Controller::request_strand_builders(state, nucls)?;
            }
            Self::MoveBuilders(n) => {
                Controller::move_strand_builders(state, n)?;
            }
            Self::Cut { nucl, .. } => {
                Controller::cut(state, nucl)?;
            }
            Self::AddGridHelix {
                position,
                length,
                start,
            } => {
                Controller::add_grid_helix(state, position, start, length)?;
            }
            Self::AddTwoPointsBezier { start, end } => {
                Controller::add_two_points_bezier(state, start, end)?;
            }
            Self::CrossCut {
                target_3prime,
                source_id,
                target_id,
                nucl,
            } => {
                Controller::apply_cross_cut(state, source_id, target_id, nucl, target_3prime)?;
            }
            Self::Xover {
                prime5_id,
                prime3_id,
            } => {
                Controller::apply_merge(state, prime5_id, prime3_id)?;
            }
            Self::GeneralXover { source, target } => {
                Controller::apply_general_cross_over(state, source, target)?;
            }
            Self::RmStrands { strand_ids } => {
                Controller::delete_strands(state, strand_ids)?;
            }
            Self::RmHelices { h_ids } => {
                Controller::delete_helices(state, h_ids)?;
            }
            Self::RmXovers { xovers } => {
                Controller::delete_xovers(state, &xovers)?;
            }
            Self::SetScaffoldId(s_id) => {
                state.design_mut().scaffold_id = s_id;
            }
            Self::HyperboloidOperation(op) => {
                Controller::apply_hyperboloid_operation(state, op)?;
            }
            Self::SetRollHelices { helices, roll } => {
                Controller::set_roll_helices(state, helices, roll)?;
            }
            Self::SetVisibilityHelix { helix, visible } => {
                Controller::set_visibility_helix(state, helix, visible)?;
            }
            Self::FlipHelixGroup { helix } => {
                Controller::flip_helix_group(state, helix)?;
            }
            Self::UpdateAttribute {
                attribute,
                elements,
            } => {
                Controller::update_attribute(state, attribute, elements)?;
            }
            Self::FlipAnchors { nucls } => {
                Controller::flip_anchors(state, nucls)?;
            }
            Self::CleanDesign => {
                //TODO
                return Err(OperationError::NotImplemented);
            }
            Self::AttachObject { object, grid, x, y } => {
                Controller::attach_object(state.design_mut(), object, grid, x, y)?;
            }
            Self::SetOrganizerTree(tree) => {
                state.design_mut().organizer_tree = Some(Arc::new(tree));
            }
            Self::SetStrandName { s_id, name } => {
                Controller::change_strand_name(state, s_id, name)?;
            }
            Self::SetGroupPivot { group_id, pivot } => {
                Controller::set_group_pivot(state.design_mut(), group_id, pivot)?;
            }
            Self::CreateNewCamera {
                position,
                orientation,
                pivot_position,
            } => {
                Controller::create_camera(
                    state.design_mut(),
                    position,
                    orientation,
                    pivot_position,
                );
            }
            Self::DeleteCamera(camera_id) => {
                Controller::delete_camera(state.design_mut(), camera_id)?;
            }
            Self::SetCameraName { camera_id, name } => {
                Controller::set_camera_name(state.design_mut(), camera_id, name)?;
            }
            Self::SetGridPosition { grid_id, position } => {
                Controller::set_grid_position(state.design_mut(), grid_id, position)?;
            }
            Self::SetGridOrientation {
                grid_id,
                orientation,
            } => {
                Controller::set_grid_orientation(state.design_mut(), grid_id, orientation)?;
            }
            Self::SetGridNbTurn { grid_id, nb_turn } => {
                Controller::set_grid_nb_turn(state.design_mut(), grid_id, nb_turn as f64)?;
            }
            Self::MakeSeveralXovers { xovers, doubled } => {
                Controller::apply_several_xovers(state, xovers, doubled)?;
            }

            Self::CheckXovers { xovers } => {
                Controller::check_xovers(state.design_mut(), xovers)?;
            }
            Self::SetRainbowScaffold(b) => {
                state.design_mut().rainbow_scaffold = b;
            }
            Self::SetGlobalHelixParameters {
                helix_parameters: parameters,
            } => {
                state.design_mut().helix_parameters = Some(parameters);
            }
            Self::SetInsertionLength {
                insertion_point,
                length,
            } => {
                Controller::update_insertion_length(state, insertion_point, length)?;
            }
            Self::AddBezierPlane { desc } => {
                Controller::add_bezier_plane(state.design_mut(), desc);
            }
            Self::CreateBezierPath { first_vertex } => {
                Controller::create_bezier_path(state, first_vertex);
            }
            Self::AppendVertexToPath { path_id, vertex } => {
                Controller::append_vertex_to_bezier_path(state, path_id, vertex)?;
            }
            Self::MoveBezierVertex { vertices, position } => {
                Controller::move_bezier_vertices(state, vertices, position)?;
            }
            Self::SetBezierVertexPosition {
                vertex_id,
                position,
            } => {
                Controller::set_bezier_vertex_position(state, vertex_id, position)?;
            }
            Self::TurnPathVerticesIntoGrid { path_id, grid_type } => {
                Controller::turn_bezier_path_into_grids(state, path_id, grid_type)?;
            }

            Self::ApplyHomothethyOnBezierPlane { homothethy } => {
                Controller::apply_homothethy_on_bezier_plane(state.design_mut(), homothethy);
            }
            Self::SetVectorOfBezierTangent(requested_vector) => {
                Controller::set_bezier_tangent(state, requested_vector)?;
            }
            Self::MakeBezierPathCyclic { path_id, cyclic } => {
                Controller::make_bezier_path_cyclic(state.design_mut(), path_id, cyclic)?;
            }
            Self::RmFreeGrids { grid_ids } => {
                Controller::delete_free_grids(state.design_mut(), grid_ids)?;
            }
            Self::RmBezierVertices { vertices } => {
                Controller::rm_bezier_vertices(state.design_mut(), vertices)?;
            }
            Self::Add3DObject {
                file_path,
                design_path,
            } => {
                Controller::add_3d_object(state.design_mut(), file_path, design_path)?;
            }
            Self::ImportSvgPath { path } => {
                Controller::import_svg_path(state.design_mut(), path)?;
            }
        }

        Ok(outcome)
    }
}

/// A rotation on an element of a design.
#[derive(Debug, Clone)]
pub struct DesignRotation {
    pub origin: Vec3,
    pub rotation: Rotor3,
    /// The element of the design on which the rotation will be applied.
    pub target: IsometryTarget,
    pub group_id: Option<GroupId>,
}

/// A translation of an element of a design.
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

/// A element on which an isometry must be applied.
#[derive(Clone, Debug)]
pub enum IsometryTarget {
    /// An helix of the design.
    Helices(Vec<usize>, bool),
    /// A grid of the design.
    Grids(Vec<GridId>),
    /// The pivot of a group.
    GroupPivot(GroupId),
    /// The control points of bezier curves.
    ControlPoint(Vec<(usize, BezierControlPoint)>),
}

#[derive(Clone, Debug, Copy)]
pub struct NewBezierTangentVector {
    pub vertex_id: BezierVertexId,
    /// Whether `new_vector` is the vector of the inward or outward tangent.
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
