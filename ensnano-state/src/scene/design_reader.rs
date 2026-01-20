use std::{collections::BTreeMap, sync::Arc};

use crate::design::selection::MainDesignReaderExt;
use ahash::{HashMap, HashSet};
use ensnano_design::{
    AdditionalStructure,
    bezier_plane::{BezierPathId, BezierPlaneId, BezierPlanes, BezierVertex, InstantiatedPath},
    curves::{
        CurveDescriptor, SurfaceInfo, SurfacePoint,
        bezier::{BezierControlPoint, CubicBezierConstructor},
    },
    external_3d_objects::External3DObjects,
    grid::{GridId, GridInstance, GridObject, GridPosition, HelixGridPosition},
    helices::HBond,
    nucl::Nucl,
    parameters::HelixParameters,
};
use ensnano_utils::{
    ObjectType, Referential,
    graphics::{LoopoutBond, LoopoutNucl},
};
use serde::{Deserialize, Serialize};
use ultraviolet::{Mat4, Rotor3, Vec2, Vec3};

#[derive(Serialize, Deserialize, Debug)]
pub struct StrandNucleotidesPositions {
    pub is_cyclic: bool,
    pub positions: Vec<[f32; 3]>,
    pub curvatures: Vec<f64>,
    pub torsions: Vec<f64>,
}

pub trait SceneDesignReaderExt: 'static + MainDesignReaderExt {
    /// Return the identifier of all the visible nucleotides
    fn get_all_visible_nucl_ids(&self) -> Vec<u32>;
    /// Return the identifier of all the visible bounds
    fn get_all_visible_bond_ids(&self) -> Vec<u32>;
    fn get_all_nucl_ids(&self) -> Vec<u32>;
    fn get_all_bond_ids(&self) -> Vec<u32>;
    fn get_pasted_position(&self) -> Vec<(Vec<Vec3>, bool)>;
    /// If e_id is the identifier of a nucleotide, return the position on which the
    /// nucleotide's symbols must be displayed
    fn get_symbol_position(&self, e_id: u32) -> Option<Vec3>;
    /// If e_id is the identifier of a nucleotide, return the symbol associated to the
    /// nucleotide.
    fn get_symbol(&self, e_id: u32) -> Option<char>;
    fn get_model_matrix(&self) -> Mat4;
    fn get_scalebar(&self) -> Option<Scalebar>;
    /// Return the list of pairs of nucleotides that can be linked by a cross-over
    fn get_suggestions(&self) -> Vec<(Nucl, Nucl)>;
    fn get_position_of_nucl_on_helix(
        &self,
        nucl: Nucl,
        referential: Referential,
        on_axis: bool,
    ) -> Option<Vec3>;
    fn get_object_type(&self, id: u32) -> Option<ObjectType>;
    fn get_grid_position(&self, g_id: GridId) -> Option<Vec3>;
    fn get_grid_lattice_position(&self, position: GridPosition) -> Option<Vec3>;
    fn get_element_position(&self, e_id: u32, referential: Referential) -> Option<Vec3>;
    fn get_element_axis_position(&self, e_id: u32, referential: Referential) -> Option<Vec3>;
    fn get_element_graphic_position(&self, e_id: u32, referential: Referential) -> Option<Vec3>;
    fn get_color(&self, e_id: u32) -> Option<u32>;
    fn get_radius(&self, e_id: u32) -> Option<f32>;
    fn get_xover_coloring(&self, e_id: u32) -> Option<bool>;
    fn get_with_cones(&self, e_id: u32) -> Option<bool>;
    fn get_id_of_strand_containing(&self, e_id: u32) -> Option<usize>;
    fn get_id_of_helix_containing(&self, e_id: u32) -> Option<usize>;
    fn get_ids_of_elements_belonging_to_strand(&self, s_id: usize) -> Vec<u32>;
    fn get_ids_of_elements_belonging_to_helix(&self, h_id: usize) -> Vec<u32>;
    fn get_helix_basis(&self, h_id: u32) -> Option<Rotor3>;
    fn get_identifier_nucl(&self, nucl: &Nucl) -> Option<u32>;
    fn get_identifier_bond(&self, n1: Nucl, n2: Nucl) -> Option<u32>;
    fn get_nucl_with_id(&self, e_id: u32) -> Option<Nucl>;
    /// Return the nucleotide with id e_id or the 5' end of the bond with id e_id
    fn get_nucl_with_id_relaxed(&self, e_id: u32) -> Option<Nucl>;
    fn can_start_builder_at(&self, nucl: &Nucl) -> bool;
    fn get_grid_instances(&self) -> BTreeMap<GridId, GridInstance>;
    fn get_helices_on_grid(&self, g_id: GridId) -> Option<HashSet<usize>>;
    fn get_used_coordinates_on_grid(&self, g_id: GridId) -> Option<Vec<(isize, isize)>>;
    fn get_helices_grid_key_coord(&self, g_id: GridId) -> Option<Vec<((isize, isize), usize)>>;
    fn get_helix_id_at_grid_coord(&self, position: GridPosition) -> Option<u32>;
    fn get_persistent_phantom_helices_id(&self) -> HashSet<u32>;
    fn get_grid_basis(&self, g_id: GridId) -> Option<Rotor3>;
    fn get_helix_grid_position(&self, h_id: u32) -> Option<HelixGridPosition>;
    fn prime5_of_which_strand(&self, nucl: Nucl) -> Option<usize>;
    fn prime3_of_which_strand(&self, nucl: Nucl) -> Option<usize>;
    fn get_curve_range(&self, h_id: usize) -> Option<std::ops::RangeInclusive<isize>>;
    fn get_checked_xovers_ids(&self, checked: bool) -> Vec<u32>;
    fn get_id_of_xover_involving_nucl(&self, nucl: Nucl) -> Option<usize>;
    fn get_grid_object(&self, position: GridPosition) -> Option<GridObject>;
    fn get_position_of_bezier_control(
        &self,
        helix: usize,
        control: BezierControlPoint,
    ) -> Option<Vec3>;
    fn get_cubic_bezier_controls(&self, helix: usize) -> Option<CubicBezierConstructor>;
    fn get_piecewise_bezier_controls(&self, helix: usize) -> Option<Vec<Vec3>>;
    fn get_curve_descriptor(&self, helix: usize) -> Option<&CurveDescriptor>;
    fn get_all_h_bonds(&self) -> &[HBond];
    fn get_all_loopout_nucl(&self) -> &[LoopoutNucl];
    fn get_all_loopout_bonds(&self) -> &[LoopoutBond];
    fn get_insertion_length(&self, bond_id: u32) -> usize;
    fn get_expected_bond_length(&self) -> f32;
    fn get_bezier_planes(&self) -> &BezierPlanes;
    fn get_parameters(&self) -> HelixParameters;
    fn get_bezier_paths(&self) -> Option<&BTreeMap<BezierPathId, Arc<InstantiatedPath>>>;
    fn get_bezier_vertex(&self, path_id: BezierPathId, vertex_id: usize) -> Option<BezierVertex>;
    fn get_corners_of_plane(&self, plane_id: BezierPlaneId) -> [Vec2; 4];
    fn get_optimal_xover_around(&self, source: Nucl, target: Nucl) -> Option<(Nucl, Nucl)>;
    fn get_bezier_grid_used_by_helix(&self, h_id: usize) -> Vec<GridId>;
    fn get_external_objects(&self) -> &External3DObjects;
    fn get_surface_info_nucl(&self, nucl: Nucl) -> Option<SurfaceInfo>;
    fn get_surface_info(&self, point: SurfacePoint) -> Option<SurfaceInfo>;
    fn get_additional_structure(&self) -> Option<&dyn AdditionalStructure>;
    fn get_nucleotides_positions_by_strands(&self) -> HashMap<usize, StrandNucleotidesPositions>;
}

pub type Scalebar = (f32, f32, fn(f32, f32, f32) -> u32);
