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

//! This modules defines types and operations used by the graphical component of ENSnano to
//! interact with the design.

use ensnano_design::{
    self, BezierPathId, BezierPlaneDescriptor, BezierPlaneId, BezierVertex, BezierVertexId,
    CurveDescriptor2D, HelixParameters, Isometry3, Nucl,
    elements::{DesignElementKey, DnaAttribute},
    grid::{GridDescriptor, GridId, GridObject, GridTypeDescr, HelixGridPosition, Hyperboloid},
    group_attributes::GroupPivot,
};
use std::path::PathBuf;
use ultraviolet::{Isometry2, Rotor3, Vec2, Vec3};
pub mod app_state_parameters;
pub mod graphics;
mod selection;
pub use selection::*;
pub mod application;
pub mod operation;
mod strand_builder;
pub use strand_builder::*;
pub mod consts;
pub mod torsion;
use ensnano_organizer::GroupId;
mod operation_labels;
mod surfaces;
pub use surfaces::*;
mod multiplexer;
pub use multiplexer::Multiplexer;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ObjectType {
    /// A nucleotide identified by its identifier
    Nucleotide(u32),
    /// A bond, identified by the identifier of the two nucleotides that it binds.
    Bond(u32, u32),
    /// A bond, identified by the identifier of the four nucleotides prev_nucl, nucl1, nucl2, next_nucl. If prev == nucl1 or newt == nucl2, it needs a lid
    SlicedBond(u32, u32, u32, u32),
    /// A Helix cylinder, identified by the identifier of the two nucleotides at its ends.
    HelixCylinder(u32, u32),
    /// A Helix cylinder, identified by the identifier of the two nucleotides at its ends, together with the list of the colors of the slices.
    ColoredHelixCylinder(u32, u32, Vec<u32>),
}

impl ObjectType {
    pub fn is_bond(&self) -> bool {
        matches!(self, ObjectType::Bond(_, _))
    }

    pub fn is_helix_cylinder(&self) -> bool {
        matches!(self, ObjectType::HelixCylinder(_, _))
    }

    pub fn same_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// The referential in which one wants to get an element's coordinates
#[derive(Debug, Clone, Copy)]
pub enum Referential {
    World,
    Model,
}

#[derive(Debug, Clone)]
/// An operation that can be performed on a design
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
        segment: usize,
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
    SetOrganizerTree(ensnano_design::OrganizerTree<DesignElementKey>),
    SetStrandName {
        s_id: usize,
        name: String,
    },
    SetGroupPivot {
        group_id: GroupId,
        pivot: GroupPivot,
    },
    DeleteCamera(ensnano_design::CameraId),
    CreateNewCamera {
        position: Vec3,
        orientation: Rotor3,
        pivot_position: Option<Vec3>,
    },
    SetCameraName {
        camera_id: ensnano_design::CameraId,
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

#[derive(Clone, Debug, Copy)]
pub struct NewBezierTangentVector {
    pub vertex_id: BezierVertexId,
    /// Whether `new_vector` is the vector of the inward or outward tangent
    pub tangent_in: bool,
    pub full_symmetry_other_tangent: bool,
    pub new_vector: Vec2,
}

#[derive(Clone, Debug, Copy)]
pub struct InsertionPoint {
    pub nucl: Nucl,
    pub nucl_is_prime5_of_insertion: bool,
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

impl ToString for IsometryTarget {
    fn to_string(&self) -> String {
        match self {
            Self::Helices(hs, _) => format!("Helices {:?}", hs),
            Self::Grids(gs) => format!("Grids {:?}", gs),
            Self::GroupPivot(_) => "Group pivot".into(),
            Self::ControlPoint(_) => "Bezier control point".into(),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Clone, Debug)]
pub struct RollRequest {
    pub target_helices: Option<Vec<usize>>,
}

#[derive(Clone, Copy, Debug)]
pub enum RapierSimulationRequest {
    Start,
}

#[derive(Clone, Debug)]
pub struct RigidBodyConstants {
    pub k_spring: f32,
    pub k_friction: f32,
    pub mass: f32,
    pub volume_exclusion: bool,
    pub brownian_motion: bool,
    pub brownian_rate: f32,
    pub brownian_amplitude: f32,
}

impl Default for RigidBodyConstants {
    fn default() -> Self {
        Self {
            k_friction: 1.,
            k_spring: 1.,
            mass: 1.,
            volume_exclusion: false,
            brownian_amplitude: 1.,
            brownian_rate: 1.,
            brownian_motion: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScaffoldInfo {
    pub id: usize,
    pub length: usize,
    pub starting_nucl: Option<Nucl>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    None,
    Rolling,
    RigidGrid,
    RigidHelices,
    Paused,
    Twisting { grid_id: GridId },
    Relaxing,
}

impl SimulationState {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_rolling(&self) -> bool {
        matches!(self, Self::Rolling)
    }

    pub fn simulating_grid(&self) -> bool {
        matches!(self, Self::RigidGrid)
    }

    pub fn simulating_helices(&self) -> bool {
        matches!(self, Self::RigidHelices)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused)
    }

    pub fn is_running(&self) -> bool {
        !matches!(self, Self::Paused | Self::None)
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WidgetBasis {
    World,
    Object,
}

impl WidgetBasis {
    pub fn toggle(&mut self) {
        if self.is_axis_aligned() {
            *self = WidgetBasis::Object
        } else {
            *self = WidgetBasis::World
        };
    }

    pub fn is_axis_aligned(&self) -> bool {
        match self {
            Self::World => true,
            Self::Object => false,
        }
    }
}

impl Default for WidgetBasis {
    fn default() -> Self {
        Self::World
    }
}

/// Information about the domain being elongated
#[derive(Debug, Clone)]
pub struct StrandBuildingStatus {
    pub nt_length: usize,
    pub nm_length: f32,
    pub prime3: Nucl,
    pub prime5: Nucl,
    pub dragged_nucl: Nucl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PastingStatus {
    Copy,
    Duplication,
    None,
}

impl PastingStatus {
    pub fn is_pasting(&self) -> bool {
        match self {
            Self::Copy | Self::Duplication => true,
            Self::None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BezierPlaneHomothethy {
    pub plane_id: BezierPlaneId,
    pub fixed_corner: Vec2,
    pub origin_moving_corner: Vec2,
    pub moving_corner: Vec2,
}

#[derive(Debug, Clone, Copy)]
/// One of the standard scaffold sequence shipped with ENSnano
pub enum StandardSequence {
    P4844,
    P7249,
    P7560,
    P8064,
}

impl StandardSequence {
    pub fn description(&self) -> &'static str {
        match self {
            Self::P4844 => "m13 p4844",
            Self::P7249 => "m13 p7249",
            Self::P7560 => "m13 p7560",
            Self::P8064 => "m13 p8064",
        }
    }

    pub fn sequence(&self) -> &'static str {
        match self {
            Self::P4844 => include_str!("../../txt/p4844-Tilibit.txt"),
            Self::P7249 => include_str!("../../txt/p7249-Tilibit.txt"),
            Self::P7560 => include_str!("../../txt/p7560.txt"),
            Self::P8064 => include_str!("../../txt/m13-p8064.txt"),
        }
    }

    /// Return the variant of Self whose associated sequence length is closest to n
    pub fn from_length(n: usize) -> Self {
        let mut best_score = isize::MAX;
        let mut ret = Self::default();
        for candidate in [Self::P4844, Self::P7249, Self::P7560, Self::P8064] {
            let score = (candidate.sequence().len() as isize - (n as isize)).abs();
            if score < best_score {
                best_score = score;
                ret = candidate;
            }
        }
        ret
    }
}

impl Default for StandardSequence {
    fn default() -> Self {
        Self::P7249
    }
}
