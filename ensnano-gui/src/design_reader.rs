use std::sync::Arc;

use ensnano_design::{
    CameraId,
    bezier_plane::{BezierPathId, BezierVertexId},
    design_element::DesignElement,
    grid::GridId,
    nucl::Nucl,
    organizer_tree::OrganizerTree,
};
use ensnano_state::design::{operation::InsertionPoint, selection::Selection};
use ultraviolet::{Rotor3, Vec2, Vec3};

pub trait GuiDesignReaderExt: 'static {
    fn grid_has_persistent_phantom(&self, g_id: GridId) -> bool;
    fn grid_has_small_spheres(&self, g_id: GridId) -> bool;
    fn get_strand_length(&self, s_id: usize) -> Option<usize>;
    fn is_id_of_scaffold(&self, s_id: usize) -> bool;
    fn length_decomposition(&self, s_id: usize) -> String;
    fn nucl_is_anchor(&self, nucl: Nucl) -> bool;
    fn get_dna_elements(&self) -> &[DesignElement];
    fn get_organizer_tree(&self) -> Option<Arc<OrganizerTree>>;
    fn strand_name(&self, s_id: usize) -> String;
    fn get_all_cameras(&self) -> Vec<(CameraId, &str)>;
    fn get_grid_position_and_orientation(&self, g_id: GridId) -> Option<(Vec3, Rotor3)>;
    fn get_grid_nb_turn(&self, g_id: GridId) -> Option<f32>;
    fn xover_length(&self, xover_id: usize) -> Option<(f32, Option<f32>)>;
    fn get_id_of_xover_involving_nucl(&self, nucl: Nucl) -> Option<usize>;
    fn rainbow_scaffold(&self) -> bool;
    fn get_insertion_length(&self, selection: &Selection) -> Option<usize>;
    fn get_insertion_point(&self, selection: &Selection) -> Option<InsertionPoint>;
    fn is_bezier_path_cyclic(&self, path_id: BezierPathId) -> Option<bool>;
    fn get_bezier_vertex_position(&self, vertex_id: BezierVertexId) -> Option<Vec2>;
    fn get_scaffold_sequence(&self) -> Option<&str>;
    fn get_current_length_of_relaxed_shape(&self) -> Option<usize>;
}
