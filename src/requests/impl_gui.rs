//! Implements the [Requests](`ensnano_gui::Requests`) trait for [Requests](`super::Requests`).

use crate::controller::normal_state::Action;
use crate::requests::Requests;
use ensnano_design::{
    CameraId,
    bezier_plane::{BezierPathId, BezierVertexId},
    elements::{DesignElementKey, DnaAttribute},
    grid::{GridId, GridTypeDescr},
    parameters::HelixParameters,
};
use ensnano_exports::ExportType;
use ensnano_gui::{OverlayType, Requests as GuiRequests, left_panel::RigidBodyParametersRequest};
use ensnano_organizer::{
    keyboard_priority::PriorityRequest,
    tree::{GroupId, OrganizerTree},
};
use ensnano_physics::parameters::RapierParameters;
use ensnano_utils::{
    DesignOperation, HyperboloidRequest, InsertionPoint, RigidBodyConstants, RollRequest,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    application::Notification,
    graphics::{Background3D, FogParameters, HBondDisplay, RenderingMode, SplitMode},
    operation::Operation,
    selection::{ActionMode, SelectionMode},
    surfaces::{RevolutionSurfaceSystemDescriptor, UnrootedRevolutionSurfaceDescriptor},
    ui_size::UiSize,
};
use std::{collections::BTreeSet, sync::Arc};
use ultraviolet::{Rotor3, Vec2, Vec3};

impl GuiRequests for Requests {
    fn close_overlay(&mut self, overlay_type: OverlayType) {
        self.keep_proceed
            .push_back(Action::CloseOverlay(overlay_type));
    }

    fn change_strand_color(&mut self, color: u32) {
        self.strand_color_change = Some(color);
    }

    fn change_3d_background(&mut self, bg: Background3D) {
        self.background3d = Some(bg);
    }

    fn change_3d_rendering_mode(&mut self, rendering_mode: RenderingMode) {
        self.rendering_mode = Some(rendering_mode);
    }

    fn set_scaffold_from_selection(&mut self) {
        self.select_scaffold = Some(());
    }

    fn cancel_hyperboloid(&mut self) {
        self.cancel_hyperboloid = Some(());
    }

    fn invert_scroll(&mut self, invert: bool) {
        self.set_invert_y_scroll = Some(invert);
    }

    fn resize_2d_helices(&mut self, all: bool) {
        self.redim_2d_helices = Some(all);
    }

    fn make_all_elements_visible(&mut self) {
        self.all_visible = Some(());
    }

    fn toggle_visibility(&mut self, visible: bool) {
        self.toggle_visibility = Some(visible);
    }

    fn change_action_mode(&mut self, action_mode: ActionMode) {
        self.action_mode = Some(action_mode);
    }

    fn change_selection_mode(&mut self, selection_mode: SelectionMode) {
        self.selection_mode = Some(selection_mode);
    }

    fn toggle_widget_basis(&mut self) {
        self.toggle_widget_basis = Some(());
    }

    fn set_dna_sequences_visibility(&mut self, visible: bool) {
        self.toggle_text = Some(visible);
    }

    fn download_staples(&mut self) {
        self.keep_proceed.push_back(Action::DownloadStaplesRequest);
    }

    fn set_scaffold_sequence(&mut self, shift: usize) {
        self.keep_proceed
            .push_back(Action::SetScaffoldSequence { shift });
    }

    fn set_scaffold_shift(&mut self, shift: usize) {
        self.scaffold_shift = Some(shift);
    }

    fn set_ui_size(&mut self, size: UiSize) {
        self.keep_proceed.push_back(Action::ChangeUiSize(size));
    }

    fn finalize_hyperboloid(&mut self) {
        self.finalize_hyperboloid = Some(());
    }

    fn stop_roll_simulation(&mut self) {
        self.stop_roll = Some(());
    }

    fn start_roll_simulation(&mut self, roll_request: RollRequest) {
        self.roll_request = Some(roll_request);
    }

    fn request_rapier_simulation(&mut self, parameters: RapierParameters) {
        self.rapier_simulation_parameters = Some(parameters);
    }

    fn make_grid_from_selection(&mut self) {
        self.make_grids = Some(());
    }

    fn update_rigid_helices_simulation(&mut self, parameters: RigidBodyParametersRequest) {
        let rigid_body_parameters = rigid_parameters(parameters);
        self.rigid_helices_simulation = Some(rigid_body_parameters);
    }

    fn update_rigid_grids_simulation(&mut self, parameters: RigidBodyParametersRequest) {
        let rigid_body_parameters = rigid_parameters(parameters);
        self.rigid_grid_simulation = Some(rigid_body_parameters);
    }

    fn update_rigid_body_simulation_parameters(&mut self, parameters: RigidBodyParametersRequest) {
        let rigid_body_parameters = rigid_parameters(parameters);
        self.rigid_body_parameters = Some(rigid_body_parameters);
    }

    fn create_new_hyperboloid(&mut self, parameters: HyperboloidRequest) {
        self.new_hyperboloid = Some(parameters);
    }

    fn update_current_hyperboloid(&mut self, parameters: HyperboloidRequest) {
        self.hyperboloid_update = Some(parameters);
    }

    fn update_roll_of_selected_helices(&mut self, roll: f32) {
        self.helix_roll = Some(roll);
    }

    fn update_scroll_sensitivity(&mut self, sensitivity: f32) {
        self.scroll_sensitivity = Some(sensitivity);
    }

    fn set_fog_parameters(&mut self, parameters: FogParameters) {
        self.fog = Some(parameters);
    }

    fn set_camera_dir_up_vec(&mut self, direction: Vec3, up: Vec3) {
        self.camera_target = Some((direction, up));
    }

    fn perform_camera_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.camera_rotation = Some((x, y, z));
    }

    fn create_grid(&mut self, grid_type_descriptor: GridTypeDescr) {
        self.new_grid = Some(grid_type_descriptor);
    }

    fn create_bezier_plane(&mut self) {
        self.new_bezier_plane = Some(());
    }

    fn set_candidates_keys(&mut self, candidates: Vec<DesignElementKey>) {
        self.organizer_candidates = Some(candidates);
    }

    fn set_selected_keys(
        &mut self,
        selection: Vec<DesignElementKey>,
        group_id: Option<GroupId>,
        new_group: bool,
    ) {
        self.organizer_selection = Some((selection, group_id, new_group));
    }

    fn update_organizer_tree(&mut self, tree: OrganizerTree<DesignElementKey>) {
        self.new_tree = Some(tree);
    }

    fn update_attribute_of_elements(
        &mut self,
        attribute: DnaAttribute,
        keys: BTreeSet<DesignElementKey>,
    ) {
        self.new_attribute = Some((attribute, keys.iter().copied().collect()));
    }

    fn change_split_mode(&mut self, split_mode: SplitMode) {
        self.keep_proceed.push_back(Action::ToggleSplit(split_mode));
    }

    fn export(&mut self, export_type: ExportType) {
        self.keep_proceed.push_back(Action::Export(export_type));
    }

    fn toggle_2d_view_split(&mut self) {
        self.split2d = Some(());
    }

    fn undo(&mut self) {
        self.undo = Some(());
    }

    fn redo(&mut self) {
        self.redo = Some(());
    }

    fn force_help(&mut self) {
        self.force_help = Some(());
    }

    fn show_tutorial(&mut self) {
        self.show_tutorial = Some(());
    }

    fn new_design(&mut self) {
        self.keep_proceed.push_back(Action::NewDesign);
    }

    fn save_as(&mut self) {
        self.keep_proceed.push_back(Action::SaveAs);
    }

    fn save(&mut self) {
        self.keep_proceed.push_back(Action::QuickSave);
    }

    fn open_file(&mut self) {
        self.keep_proceed.push_back(Action::LoadDesign(None));
    }

    fn fit_design_in_scenes(&mut self) {
        self.fitting = Some(());
    }

    fn update_current_operation(&mut self, operation: Arc<dyn Operation>) {
        self.operation_update = Some(operation);
        self.suspend_op = Some(());
    }

    fn set_scaffold_id(&mut self, s_id: Option<usize>) {
        self.set_scaffold_id = Some(s_id);
    }

    fn toggle_helices_persistence_of_grid(&mut self, persistent: bool) {
        self.toggle_persistent_helices = Some(persistent);
    }

    fn set_small_sphere(&mut self, small: bool) {
        self.small_spheres = Some(small);
    }

    fn finish_changing_color(&mut self) {
        self.keep_proceed.push_back(Action::FinishChangingColor);
    }

    fn stop_simulations(&mut self) {
        self.keep_proceed.push_back(Action::StopSimulation);
    }

    fn reset_simulations(&mut self) {
        self.keep_proceed.push_back(Action::ResetSimulation);
    }

    fn reload_file(&mut self) {
        self.keep_proceed.push_back(Action::ReloadFile);
    }

    fn add_double_strand_on_new_helix(&mut self, parameters: Option<(isize, usize)>) {
        self.new_double_strand_parameters = Some(parameters);
    }

    fn set_strand_name(&mut self, s_id: usize, name: String) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetStrandName {
                s_id,
                name,
            }));
    }

    fn create_new_camera(&mut self) {
        self.keep_proceed.push_back(Action::NewCamera);
    }

    fn delete_camera(&mut self, camera_id: CameraId) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::DeleteCamera(
                camera_id,
            )));
    }

    fn select_camera(&mut self, camera_id: CameraId) {
        self.keep_proceed.push_back(Action::SelectCamera(camera_id));
    }

    fn set_camera_name(&mut self, camera_id: CameraId, name: String) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetCameraName {
                camera_id,
                name,
            }));
    }

    fn set_suggestion_parameters(&mut self, param: SuggestionParameters) {
        self.new_suggestion_parameters = Some(param);
    }

    fn set_grid_position(&mut self, grid_id: GridId, position: Vec3) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetGridPosition {
                grid_id,
                position,
            }));
    }

    fn set_grid_orientation(&mut self, grid_id: GridId, orientation: Rotor3) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetGridOrientation {
                grid_id,
                orientation,
            },
        ));
    }

    fn toggle_2d(&mut self) {
        self.keep_proceed.push_back(Action::Toggle2D);
    }

    fn set_nb_turn(&mut self, grid_id: GridId, nb_turn: f32) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetGridNbTurn {
                grid_id,
                nb_turn,
            }));
    }

    fn set_check_xover_parameters(&mut self, parameters: CheckXoversParameter) {
        self.check_xover_parameters = Some(parameters);
    }

    fn follow_stereographic_camera(&mut self, follow: bool) {
        self.follow_stereographic_camera = Some(follow);
    }

    fn flip_split_views(&mut self) {
        self.keep_proceed.push_back(Action::FlipSplitViews);
    }

    fn set_rainbow_scaffold(&mut self, rainbow: bool) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetRainbowScaffold(rainbow),
        ));
    }

    fn set_show_stereographic_camera(&mut self, show: bool) {
        self.set_show_stereographic_camera = Some(show);
    }

    fn set_show_h_bonds(&mut self, show: HBondDisplay) {
        self.set_show_h_bonds = Some(show);
    }

    fn set_show_bezier_paths(&mut self, show: bool) {
        self.set_show_bezier_paths = Some(show);
    }

    fn set_all_helices_on_axis(&mut self, off_axis: bool) {
        // thick helices = normal helices; thin helices = only axis
        self.set_all_helices_on_axis = Some(off_axis);
    }

    fn start_twist_simulation(&mut self, grid_id: GridId) {
        self.twist_simulation = Some(grid_id);
    }

    fn align_horizon(&mut self) {
        self.horizon_targeted = Some(());
    }

    fn download_origamis(&mut self) {
        self.keep_proceed.push_back(Action::DownloadOrigamiRequest);
    }

    fn set_dna_parameters(&mut self, param: HelixParameters) {
        self.keep_proceed.push_back(Action::SetDnaParameters(param));
    }

    fn set_expand_insertions(&mut self, expand: bool) {
        self.keep_proceed
            .push_back(Action::SetExpandInsertions(expand));
    }

    fn set_insertion_length(&mut self, insertion_point: InsertionPoint, length: usize) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetInsertionLength {
                length,
                insertion_point,
            },
        ));
    }

    fn turn_path_into_grid(&mut self, path_id: BezierPathId, grid_type: GridTypeDescr) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::TurnPathVerticesIntoGrid { path_id, grid_type },
        ));
    }

    fn make_bezier_path_cyclic(&mut self, path_id: BezierPathId, cyclic: bool) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::MakeBezierPathCyclic { path_id, cyclic },
        ));
    }

    fn set_exporting(&mut self, exporting: bool) {
        self.keep_proceed.push_back(Action::SetExporting(exporting));
    }

    fn import_3d_object(&mut self) {
        self.keep_proceed.push_back(Action::Import3DObject);
    }

    fn set_position_of_bezier_vertex(&mut self, vertex_id: BezierVertexId, position: Vec2) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetBezierVertexPosition {
                vertex_id,
                position,
            },
        ));
    }

    fn optimize_scaffold_shift(&mut self) {
        self.keep_proceed.push_back(Action::OptimizeShift);
    }

    fn start_revolution_relaxation(&mut self, desc: RevolutionSurfaceSystemDescriptor) {
        self.keep_proceed
            .push_back(Action::RevolutionSimulation { desc });
    }

    fn finish_revolution_relaxation(&mut self) {
        self.keep_proceed
            .push_back(Action::FinishRelaxationSimulation);
    }

    fn load_svg(&mut self) {
        self.keep_proceed.push_back(Action::ImportSvg);
    }

    fn set_bezier_revolution_id(&mut self, id: Option<usize>) {
        self.new_bezier_revolution_id = Some(id);
    }

    fn set_bezier_revolution_radius(&mut self, radius: f64) {
        self.new_bezier_revolution_radius = Some(radius);
    }

    fn request_screenshot_2d(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::ScreenShot2D(path)
            }));
    }

    fn request_screenshot_3d(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::ScreenShot3D(path)
            }));
    }

    fn request_save_nucleotides_positions(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::SaveNucleotidesPositions(path)
            }));
    }

    fn set_unrooted_surface(&mut self, surface: Option<UnrootedRevolutionSurfaceDescriptor>) {
        self.new_unrooted_surface = Some(surface);
    }

    fn notify_revolution_tab(&mut self) {
        self.switched_to_revolution_tab = Some(());
    }

    fn request_stl_export(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::StlExport(path)
            }));
    }

    fn set_keyboard_priority(&mut self, priority: PriorityRequest) {
        self.set_keyboard_priority
            .get_or_insert_default()
            .push(priority);
    }
}

fn rigid_parameters(parameters: RigidBodyParametersRequest) -> RigidBodyConstants {
    let ret = RigidBodyConstants {
        k_spring: 10f32.powf(parameters.k_springs),
        k_friction: 10f32.powf(parameters.k_friction),
        mass: 10f32.powf(parameters.mass_factor),
        volume_exclusion: parameters.volume_exclusion,
        brownian_motion: parameters.brownian_motion,
        brownian_rate: 10f32.powf(parameters.brownian_rate),
        brownian_amplitude: parameters.brownian_amplitude,
    };
    log::info!("rigid parameters {ret:?}");
    ret
}
