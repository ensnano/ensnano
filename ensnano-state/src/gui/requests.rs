use std::{collections::BTreeSet, sync::Arc};

use crate::{
    design::operation::{HyperboloidRequest, InsertionPoint},
    utils::operation::Operation,
};
use ensnano_design::{
    CameraId,
    bezier_plane::{BezierPathId, BezierVertexId},
    design_element::{DesignElementKey, DnaAttribute},
    grid::{GridId, GridTypeDescr},
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::{GroupId, OrganizerTree},
    parameters::HelixParameters,
};
use ensnano_exports::ExportType;
use ensnano_physics::parameters::RapierParameters;
use ensnano_utils::{
    RollRequest,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    graphics::{Background3D, FogParameters, HBondDisplay, RenderingMode, SplitMode},
    keyboard_priority::PriorityRequest,
    overlay::OverlayType,
    surfaces::{RevolutionSurfaceSystemDescriptor, UnrootedRevolutionSurfaceDescriptor},
    ui_size::UiSize,
};
use ultraviolet::{Rotor3, Vec2, Vec3};

pub trait GuiRequests: 'static + Send {
    fn close_overlay(&mut self, overlay_type: OverlayType);
    /// Change the color of the selected strands
    fn change_strand_color(&mut self, color: u32);
    /// Change the background of the 3D scene
    fn change_3d_background(&mut self, bg: Background3D);
    /// Change the rendering mode
    fn change_3d_rendering_mode(&mut self, rendering_mode: RenderingMode);
    /// Set the selected strand as the scaffold
    fn set_scaffold_from_selection(&mut self);
    /// Cancel the current hyperboloid construction
    fn cancel_hyperboloid(&mut self);
    /// Change the scrolling direction
    fn invert_scroll(&mut self, invert: bool);
    /// Resize all the 2D helices, or only the selected ones
    fn resize_2d_helices(&mut self, all: bool);
    /// Make all elements of the design visible
    fn make_all_elements_visible(&mut self);
    /// Toggle the visibility of the selected elements
    fn toggle_visibility(&mut self, visible: bool);
    fn change_action_mode(&mut self, action_mode: ActionMode);
    fn change_selection_mode(&mut self, selection_mode: SelectionMode);
    /// Switch widget basis between world and object
    fn toggle_widget_basis(&mut self);
    /// Show/hide the DNA sequences
    fn set_dna_sequences_visibility(&mut self, visible: bool);
    /// Download the staples as an xlsx file
    fn download_staples(&mut self);
    fn set_scaffold_sequence(&mut self, shift: usize);
    fn set_scaffold_shift(&mut self, shift: usize);
    /// Change the size of the UI components
    fn set_ui_size(&mut self, size: UiSize);
    /// Finalize the currently edited hyperboloid grid
    fn finalize_hyperboloid(&mut self);
    fn stop_roll_simulation(&mut self);
    fn start_roll_simulation(&mut self, roll_request: RollRequest);
    /// Request a Rapier simulation of the current design
    fn request_rapier_simulation(&mut self, parameters: RapierParameters);
    /// Make a grid from the set of selected helices
    fn make_grid_from_selection(&mut self);
    /// Start of Update the rigid helices simulation
    fn update_rigid_helices_simulation(&mut self, parameters: RigidBodyParametersRequest);
    /// Start of Update the rigid grids simulation
    fn update_rigid_grids_simulation(&mut self, parameters: RigidBodyParametersRequest);
    fn start_twist_simulation(&mut self, grid_id: GridId);
    /// Update the parameters of the current simulation (rigid grids or helices)
    fn update_rigid_body_simulation_parameters(&mut self, parameters: RigidBodyParametersRequest);
    fn create_new_hyperboloid(&mut self, parameters: HyperboloidRequest);
    /// Update the parameters of the currently edited hyperboloid grid
    fn update_current_hyperboloid(&mut self, parameters: HyperboloidRequest);
    fn update_roll_of_selected_helices(&mut self, roll: f32);
    fn update_scroll_sensitivity(&mut self, sensitivity: f32);
    fn set_fog_parameters(&mut self, parameters: FogParameters);
    /// Set the direction and up vector of the 3D camera
    fn set_camera_dir_up_vec(&mut self, direction: Vec3, up: Vec3);
    fn perform_camera_rotation(&mut self, x: f32, y: f32, z: f32);
    /// Create a new grid in front of the 3D camera
    fn create_grid(&mut self, grid_type_descriptor: GridTypeDescr);
    fn set_candidates_keys(&mut self, candidates: Vec<DesignElementKey>);
    fn set_selected_keys(
        &mut self,
        selection: Vec<DesignElementKey>,
        group_id: Option<GroupId>,
        new_group: bool,
    );
    fn update_organizer_tree(&mut self, tree: OrganizerTree);
    /// Update one attribute of several Dna Elements
    fn update_attribute_of_elements(
        &mut self,
        attribute: DnaAttribute,
        keys: BTreeSet<DesignElementKey>,
    );
    fn change_split_mode(&mut self, split_mode: SplitMode);
    fn export(&mut self, export_type: ExportType);
    /// Split/Unsplit the 2D view
    fn toggle_2d_view_split(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
    /// Display the help message in the contextual panel, regardless of the selection
    fn force_help(&mut self);
    /// Show tutorial in the contextual panel
    fn show_tutorial(&mut self);
    fn new_design(&mut self);
    fn save_as(&mut self);
    fn save(&mut self);
    fn open_file(&mut self);
    /// Adjust the 2D and 3D cameras so that the design fit in screen
    fn fit_design_in_scenes(&mut self);
    /// Update the parameters of the current operation
    fn update_current_operation(&mut self, operation: Arc<dyn Operation>);
    /// Set the scaffold to be the some strand with id `s_id`, or none
    fn set_scaffold_id(&mut self, s_id: Option<usize>);
    /// make the spheres of the currently selected grid large/small
    fn toggle_helices_persistence_of_grid(&mut self, persistent: bool);
    /// make the spheres of the currently selected grid large/small
    fn set_small_sphere(&mut self, small: bool);
    fn finish_changing_color(&mut self);
    fn stop_simulations(&mut self);
    fn reset_simulations(&mut self);
    fn reload_file(&mut self);
    fn add_double_strand_on_new_helix(&mut self, parameters: Option<(isize, usize)>);
    fn set_strand_name(&mut self, s_id: usize, name: String);
    fn create_new_camera(&mut self);
    fn delete_camera(&mut self, camera_id: CameraId);
    fn select_camera(&mut self, camera_id: CameraId);
    fn set_camera_name(&mut self, camera_id: CameraId, name: String);
    fn set_suggestion_parameters(&mut self, param: SuggestionParameters);
    fn set_grid_position(&mut self, grid_id: GridId, position: Vec3);
    fn set_grid_orientation(&mut self, grid_id: GridId, orientation: Rotor3);
    fn toggle_2d(&mut self);
    fn set_nb_turn(&mut self, grid_id: GridId, nb_turn: f32);
    fn set_check_xover_parameters(&mut self, parameters: CheckXoversParameter);
    fn follow_stereographic_camera(&mut self, follow: bool);
    fn set_show_stereographic_camera(&mut self, show: bool);
    fn set_show_h_bonds(&mut self, show: HBondDisplay);
    fn flip_split_views(&mut self);
    fn set_rainbow_scaffold(&mut self, rainbow: bool);
    fn set_all_helices_on_axis(&mut self, off_axis: bool);
    fn align_horizon(&mut self);
    fn download_origamis(&mut self);
    fn set_dna_parameters(&mut self, param: HelixParameters);
    fn set_expand_insertions(&mut self, expand: bool);
    fn set_insertion_length(&mut self, insertion_point: InsertionPoint, length: usize);
    fn create_bezier_plane(&mut self);
    fn turn_path_into_grid(&mut self, path_id: BezierPathId, grid_type: GridTypeDescr);
    fn set_show_bezier_paths(&mut self, show: bool);
    fn make_bezier_path_cyclic(&mut self, path_id: BezierPathId, cyclic: bool);
    fn set_exporting(&mut self, exporting: bool);
    fn import_3d_object(&mut self);
    fn set_position_of_bezier_vertex(&mut self, vertex_id: BezierVertexId, position: Vec2);
    fn optimize_scaffold_shift(&mut self);
    fn start_revolution_relaxation(&mut self, desc: RevolutionSurfaceSystemDescriptor);
    fn finish_revolution_relaxation(&mut self);
    fn load_svg(&mut self);
    fn set_bezier_revolution_radius(&mut self, radius: f64);
    fn set_bezier_revolution_id(&mut self, id: Option<usize>);
    fn set_unrooted_surface(&mut self, surface: Option<UnrootedRevolutionSurfaceDescriptor>);
    /// Make a screenshot of the 2D flatscene.
    fn request_screenshot_2d(&mut self);
    /// Make a screenshot of the 3D scene.
    fn request_screenshot_3d(&mut self);
    fn request_save_nucleotides_positions(&mut self);
    fn notify_revolution_tab(&mut self);
    fn request_stl_export(&mut self);
    /// Set keyboard priority, i.e. whether activate keyboard shortcuts.
    fn set_keyboard_priority(&mut self, priority: PriorityRequest);
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
