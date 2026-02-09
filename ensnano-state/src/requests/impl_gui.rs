use crate::{
    app_state::action::Action,
    design::operation::{DesignOperation, HyperboloidRequest, InsertionPoint},
    gui::requests::RigidBodyParametersRequest,
    requests::Requests,
    utils::{application::Notification, operation::SimpleOperation},
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
    RigidBodyConstants, RollRequest,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    graphics::{Background3D, FogParameters, HBondDisplay, RenderingMode, SplitMode},
    keyboard_priority::PriorityRequest,
    overlay::OverlayType,
    surfaces::{RevolutionSurfaceSystemDescriptor, UnrootedRevolutionSurfaceDescriptor},
    ui_size::UiSize,
};
use std::{collections::BTreeSet, sync::Arc};
use ultraviolet::{Rotor3, Vec2, Vec3};

impl Requests {
    pub fn close_overlay(&mut self, overlay_type: OverlayType) {
        self.keep_proceed
            .push_back(Action::CloseOverlay(overlay_type));
    }

    pub fn change_strand_color(&mut self, color: u32) {
        self.strand_color_change = Some(color);
    }

    pub fn change_3d_background(&mut self, bg: Background3D) {
        self.background3d = Some(bg);
    }

    pub fn change_3d_rendering_mode(&mut self, rendering_mode: RenderingMode) {
        self.rendering_mode = Some(rendering_mode);
    }

    pub fn set_scaffold_from_selection(&mut self) {
        self.select_scaffold = Some(());
    }

    pub fn cancel_hyperboloid(&mut self) {
        self.cancel_hyperboloid = Some(());
    }

    pub fn invert_scroll(&mut self, invert: bool) {
        self.set_invert_y_scroll = Some(invert);
    }

    pub fn resize_2d_helices(&mut self, all: bool) {
        self.redim_2d_helices = Some(all);
    }

    pub fn make_all_elements_visible(&mut self) {
        self.all_visible = Some(());
    }

    pub fn toggle_visibility(&mut self, visible: bool) {
        self.toggle_visibility = Some(visible);
    }

    pub fn change_action_mode(&mut self, action_mode: ActionMode) {
        self.action_mode = Some(action_mode);
    }

    pub fn change_selection_mode(&mut self, selection_mode: SelectionMode) {
        self.selection_mode = Some(selection_mode);
    }

    pub fn toggle_widget_basis(&mut self) {
        self.toggle_widget_basis = Some(());
    }

    pub fn set_dna_sequences_visibility(&mut self, visible: bool) {
        self.toggle_text = Some(visible);
    }

    pub fn download_staples(&mut self) {
        self.keep_proceed.push_back(Action::DownloadStaplesRequest);
    }

    pub fn set_scaffold_sequence(&mut self, shift: usize) {
        self.keep_proceed
            .push_back(Action::SetScaffoldSequence { shift });
    }

    pub fn set_scaffold_shift(&mut self, shift: usize) {
        self.scaffold_shift = Some(shift);
    }

    pub fn set_ui_size(&mut self, size: UiSize) {
        self.keep_proceed.push_back(Action::ChangeUiSize(size));
    }

    pub fn finalize_hyperboloid(&mut self) {
        self.finalize_hyperboloid = Some(());
    }

    pub fn stop_roll_simulation(&mut self) {
        self.stop_roll = Some(());
    }

    pub fn start_roll_simulation(&mut self, roll_request: RollRequest) {
        self.roll_request = Some(roll_request);
    }

    pub fn request_rapier_simulation(&mut self, parameters: RapierParameters) {
        self.rapier_simulation_parameters = Some(parameters);
    }

    pub fn make_grid_from_selection(&mut self) {
        self.make_grids = Some(());
    }

    pub fn update_rigid_helices_simulation(&mut self, parameters: RigidBodyParametersRequest) {
        let rigid_body_parameters = rigid_parameters(parameters);
        self.rigid_helices_simulation = Some(rigid_body_parameters);
    }

    pub fn update_rigid_grids_simulation(&mut self, parameters: RigidBodyParametersRequest) {
        let rigid_body_parameters = rigid_parameters(parameters);
        self.rigid_grid_simulation = Some(rigid_body_parameters);
    }

    pub fn update_rigid_body_simulation_parameters(
        &mut self,
        parameters: RigidBodyParametersRequest,
    ) {
        let rigid_body_parameters = rigid_parameters(parameters);
        self.rigid_body_parameters = Some(rigid_body_parameters);
    }

    pub fn create_new_hyperboloid(&mut self, parameters: HyperboloidRequest) {
        self.new_hyperboloid = Some(parameters);
    }

    pub fn update_current_hyperboloid(&mut self, parameters: HyperboloidRequest) {
        self.hyperboloid_update = Some(parameters);
    }

    pub fn update_roll_of_selected_helices(&mut self, roll: f32) {
        self.helix_roll = Some(roll);
    }

    pub fn update_scroll_sensitivity(&mut self, sensitivity: f32) {
        self.scroll_sensitivity = Some(sensitivity);
    }

    pub fn set_fog_parameters(&mut self, parameters: FogParameters) {
        self.fog = Some(parameters);
    }

    pub fn set_camera_dir_up_vec(&mut self, direction: Vec3, up: Vec3) {
        self.camera_target = Some((direction, up));
    }

    pub fn perform_camera_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.camera_rotation = Some((x, y, z));
    }

    pub fn create_grid(&mut self, grid_type_descriptor: GridTypeDescr) {
        self.new_grid = Some(grid_type_descriptor);
    }

    pub fn create_bezier_plane(&mut self) {
        self.new_bezier_plane = Some(());
    }

    pub fn set_candidates_keys(&mut self, candidates: Vec<DesignElementKey>) {
        self.organizer_candidates = Some(candidates);
    }

    pub fn set_selected_keys(
        &mut self,
        selection: Vec<DesignElementKey>,
        group_id: Option<GroupId>,
        new_group: bool,
    ) {
        self.organizer_selection = Some((selection, group_id, new_group));
    }

    pub fn update_organizer_tree(&mut self, tree: OrganizerTree) {
        self.new_tree = Some(tree);
    }

    pub fn update_attribute_of_elements(
        &mut self,
        attribute: DnaAttribute,
        keys: BTreeSet<DesignElementKey>,
    ) {
        self.new_attribute = Some((attribute, keys.iter().copied().collect()));
    }

    pub fn change_split_mode(&mut self, split_mode: SplitMode) {
        self.keep_proceed.push_back(Action::ToggleSplit(split_mode));
    }

    pub fn export(&mut self, export_type: ExportType) {
        self.keep_proceed.push_back(Action::Export(export_type));
    }

    pub fn toggle_2d_view_split(&mut self) {
        self.split2d = Some(());
    }

    pub fn undo(&mut self) {
        self.undo = Some(());
    }

    pub fn redo(&mut self) {
        self.redo = Some(());
    }

    pub fn force_help(&mut self) {
        self.force_help = Some(());
    }

    pub fn show_tutorial(&mut self) {
        self.show_tutorial = Some(());
    }

    pub fn new_design(&mut self) {
        self.keep_proceed.push_back(Action::NewDesign);
    }

    pub fn save_as(&mut self) {
        self.keep_proceed.push_back(Action::SaveAs);
    }

    pub fn save(&mut self) {
        self.keep_proceed.push_back(Action::QuickSave);
    }

    pub fn open_file(&mut self) {
        self.keep_proceed.push_back(Action::LoadDesign(None));
    }

    pub fn fit_design_in_scenes(&mut self) {
        self.fitting = Some(());
    }

    pub fn update_current_operation(&mut self, operation: Arc<dyn SimpleOperation>) {
        self.operation_update = Some(operation);
        self.suspend_op = Some(());
    }

    pub fn set_scaffold_id(&mut self, s_id: Option<usize>) {
        self.set_scaffold_id = Some(s_id);
    }

    pub fn toggle_helices_persistence_of_grid(&mut self, persistent: bool) {
        self.toggle_persistent_helices = Some(persistent);
    }

    pub fn set_small_sphere(&mut self, small: bool) {
        self.small_spheres = Some(small);
    }

    pub fn finish_changing_color(&mut self) {
        self.keep_proceed.push_back(Action::FinishChangingColor);
    }

    pub fn stop_simulations(&mut self) {
        self.keep_proceed.push_back(Action::StopSimulation);
    }

    pub fn reset_simulations(&mut self) {
        self.keep_proceed.push_back(Action::ResetSimulation);
    }

    pub fn reload_file(&mut self) {
        self.keep_proceed.push_back(Action::ReloadFile);
    }

    pub fn add_double_strand_on_new_helix(&mut self, parameters: Option<(isize, usize)>) {
        self.new_double_strand_parameters = Some(parameters);
    }

    pub fn set_strand_name(&mut self, s_id: usize, name: String) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetStrandName {
                s_id,
                name,
            }));
    }

    pub fn create_new_camera(&mut self) {
        self.keep_proceed.push_back(Action::NewCamera);
    }

    pub fn delete_camera(&mut self, camera_id: CameraId) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::DeleteCamera(
                camera_id,
            )));
    }

    pub fn select_camera(&mut self, camera_id: CameraId) {
        self.keep_proceed.push_back(Action::SelectCamera(camera_id));
    }

    pub fn set_camera_name(&mut self, camera_id: CameraId, name: String) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetCameraName {
                camera_id,
                name,
            }));
    }

    pub fn set_suggestion_parameters(&mut self, param: SuggestionParameters) {
        self.new_suggestion_parameters = Some(param);
    }

    pub fn set_grid_position(&mut self, grid_id: GridId, position: Vec3) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetGridPosition {
                grid_id,
                position,
            }));
    }

    pub fn set_grid_orientation(&mut self, grid_id: GridId, orientation: Rotor3) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetGridOrientation {
                grid_id,
                orientation,
            },
        ));
    }

    pub fn toggle_2d(&mut self) {
        self.keep_proceed.push_back(Action::Toggle2D);
    }

    pub fn set_nb_turn(&mut self, grid_id: GridId, nb_turn: f32) {
        self.keep_proceed
            .push_back(Action::DesignOperation(DesignOperation::SetGridNbTurn {
                grid_id,
                nb_turn,
            }));
    }

    pub fn set_check_xover_parameters(&mut self, parameters: CheckXoversParameter) {
        self.check_xover_parameters = Some(parameters);
    }

    pub fn set_follow_stereographic_camera(&mut self, follow: bool) {
        self.follow_stereographic_camera = Some(follow);
    }

    pub fn flip_split_views(&mut self) {
        self.keep_proceed.push_back(Action::FlipSplitViews);
    }

    pub fn set_rainbow_scaffold(&mut self, rainbow: bool) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetRainbowScaffold(rainbow),
        ));
    }

    pub fn set_show_stereographic_camera(&mut self, show: bool) {
        self.set_show_stereographic_camera = Some(show);
    }

    pub fn set_show_h_bonds(&mut self, show: HBondDisplay) {
        self.set_show_h_bonds = Some(show);
    }

    pub fn set_show_bezier_paths(&mut self, show: bool) {
        self.set_show_bezier_paths = Some(show);
    }

    pub fn set_all_helices_on_axis(&mut self, off_axis: bool) {
        // thick helices = normal helices; thin helices = only axis
        self.set_all_helices_on_axis = Some(off_axis);
    }

    pub fn start_twist_simulation(&mut self, grid_id: GridId) {
        self.twist_simulation = Some(grid_id);
    }

    pub fn align_horizon(&mut self) {
        self.horizon_targeted = Some(());
    }

    pub fn download_origamis(&mut self) {
        self.keep_proceed.push_back(Action::DownloadOrigamiRequest);
    }

    pub fn set_dna_parameters(&mut self, param: HelixParameters) {
        self.keep_proceed.push_back(Action::SetDnaParameters(param));
    }

    pub fn set_expand_insertions(&mut self, expand: bool) {
        self.keep_proceed
            .push_back(Action::SetExpandInsertions(expand));
    }

    pub fn set_insertion_length(&mut self, insertion_point: InsertionPoint, length: usize) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetInsertionLength {
                length,
                insertion_point,
            },
        ));
    }

    pub fn turn_path_into_grid(&mut self, path_id: BezierPathId, grid_type: GridTypeDescr) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::TurnPathVerticesIntoGrid { path_id, grid_type },
        ));
    }

    pub fn make_bezier_path_cyclic(&mut self, path_id: BezierPathId, cyclic: bool) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::MakeBezierPathCyclic { path_id, cyclic },
        ));
    }

    pub fn set_exporting(&mut self, exporting: bool) {
        self.keep_proceed.push_back(Action::SetExporting(exporting));
    }

    pub fn import_3d_object(&mut self) {
        self.keep_proceed.push_back(Action::Import3DObject);
    }

    pub fn set_position_of_bezier_vertex(&mut self, vertex_id: BezierVertexId, position: Vec2) {
        self.keep_proceed.push_back(Action::DesignOperation(
            DesignOperation::SetBezierVertexPosition {
                vertex_id,
                position,
            },
        ));
    }

    pub fn optimize_scaffold_shift(&mut self) {
        self.keep_proceed.push_back(Action::OptimizeShift);
    }

    pub fn start_revolution_relaxation(&mut self, desc: RevolutionSurfaceSystemDescriptor) {
        self.keep_proceed
            .push_back(Action::RevolutionSimulation { desc });
    }

    pub fn finish_revolution_relaxation(&mut self) {
        self.keep_proceed
            .push_back(Action::FinishRelaxationSimulation);
    }

    pub fn load_svg(&mut self) {
        self.keep_proceed.push_back(Action::ImportSvg);
    }

    pub fn set_bezier_revolution_id(&mut self, id: Option<usize>) {
        self.new_bezier_revolution_id = Some(id);
    }

    pub fn set_bezier_revolution_radius(&mut self, radius: f64) {
        self.new_bezier_revolution_radius = Some(radius);
    }

    pub fn request_screenshot_2d(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::ScreenShot2D(path)
            }));
    }

    pub fn request_screenshot_3d(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::ScreenShot3D(path)
            }));
    }

    pub fn request_save_nucleotides_positions(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::SaveNucleotidesPositions(path)
            }));
    }

    pub fn set_unrooted_surface(&mut self, surface: Option<UnrootedRevolutionSurfaceDescriptor>) {
        self.new_unrooted_surface = Some(surface);
    }

    pub fn notify_revolution_tab(&mut self) {
        self.switched_to_revolution_tab = Some(());
    }

    pub fn request_stl_export(&mut self) {
        self.keep_proceed
            .push_back(Action::GetDesignPathAndNotify(|path| {
                Notification::StlExport(path)
            }));
    }

    pub fn set_keyboard_priority(&mut self, priority: PriorityRequest) {
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
