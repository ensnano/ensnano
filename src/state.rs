use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use ahash::HashMap;
use ensnano_design::{
    Camera, SavingInformation,
    bezier_plane::BezierPlaneDescriptor,
    grid::GridId,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::GroupId,
};
use ensnano_physics::parameters::RapierParameters;
use ensnano_state::{
    design::{
        operation::DesignOperation,
        selection::{
            CenterOfSelection, Selection, extract_only_grids, extract_strands_from_selection,
            list_of_helices, list_of_xover_as_nucl_pairs,
        },
    },
    gui::messages::{GuiMessages, TopBarStateFlags},
    scene::{design_reader::SceneDesignReaderExt as _, state::SceneAppState as _},
    utils::{
        application::{Application, Camera3D, Notification},
        operation::Operation,
    },
};
use ensnano_utils::{
    PastingStatus, RigidBodyConstants,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    consts::{ENS_BACKUP_EXTENSION, ENS_UNNAMED_FILE_NAME, SEC_PER_YEAR},
    graphics::{Background3D, GuiComponentType, HBondDisplay, RenderingMode},
    keyboard_priority::KeyboardPriorityId,
    surfaces::{RevolutionSurfaceSystemDescriptor, UnrootedRevolutionSurfaceDescriptor},
};
use ultraviolet::{Rotor3, Vec3};
use winit::window::CursorIcon;

use crate::{
    app_state::{
        AppState, SaveDesignError,
        action::Action,
        channel_reader::ChannelReader,
        design_interactor::controller::{
            ErrOperation, InteractorNotification, clipboard::CopyOperation,
            simulations::SimulationOperation,
        },
        transitions::{AppStateTransition, OkOperation, TransitionLabel},
    },
    multiplexer::Multiplexer,
};

/// The state of the main event loop.
pub(crate) struct MainState {
    pub app_state: AppState,
    pub pending_actions: VecDeque<Action>,
    pub undo_stack: Vec<AppStateTransition>,
    pub redo_stack: Vec<AppStateTransition>,
    pub channel_reader: ChannelReader,
    pub messages: Arc<Mutex<GuiMessages<AppState>>>,
    pub applications: HashMap<GuiComponentType, Arc<Mutex<dyn Application<AppState = AppState>>>>,
    pub focused_component: Option<GuiComponentType>,
    /// Disable the interception of keyboard events, to let the user input text.
    /// Some(id) indicates that object id has the priority; None indicates none have the priority.
    pub keyboard_priority: Option<KeyboardPriorityId>,
    pub last_saved_state: AppState,

    /// The name of the file containing the current design.
    ///
    /// For example, if the design is stored in `/home/alice/designs/origami.ens`, `file_name` is
    /// `origami.ens`.
    pub file_name: Option<PathBuf>,

    pub wants_fit: bool,
    pub last_backup_date: Instant,
    pub last_backed_up_state: AppState,
    pub simulation_cursor: Option<CursorIcon>,
    pub applications_cursor: Option<CursorIcon>,
    pub gui_cursor: CursorIcon,
    pub cursor: CursorIcon,
}

impl MainState {
    pub(crate) fn new(messages: Arc<Mutex<GuiMessages<AppState>>>) -> Self {
        let app_state = AppState::with_preferred_parameters().unwrap_or_else(|e| {
            log::error!("Could not load preferences {e}");
            AppState::default()
        });

        Self {
            app_state: app_state.clone(),
            pending_actions: VecDeque::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            channel_reader: Default::default(),
            messages,
            applications: Default::default(),
            focused_component: None,
            keyboard_priority: None,
            last_saved_state: app_state.clone(),
            file_name: None,
            wants_fit: false,
            last_backup_date: Instant::now(),
            last_backed_up_state: app_state,
            simulation_cursor: None,
            applications_cursor: None,
            gui_cursor: Default::default(),
            cursor: Default::default(),
        }
    }

    pub(crate) fn update_cursor(&mut self, multiplexer: &Multiplexer) -> bool {
        self.update_simulation_cursor();
        // Useful to remember to finish hyperboloid before trying to edit
        if self.app_state.is_building_hyperboloid()
            && multiplexer.focused_element().is_some_and(|e| e.is_scene())
        {
            self.applications_cursor = Some(CursorIcon::NotAllowed);
        }
        let new_cursor = if self.simulation_cursor.is_some() {
            multiplexer
                .icon
                .or_else(|| Some(self.gui_cursor).filter(|c| c != &Default::default()))
                .or(self.simulation_cursor)
                .unwrap_or_default()
        } else {
            self.applications_cursor
                .or(multiplexer.icon)
                .unwrap_or(self.gui_cursor)
        };
        let ret = self.cursor != new_cursor;
        self.cursor = new_cursor;
        ret
    }

    pub(crate) fn update_simulation_cursor(&mut self) {
        self.simulation_cursor = self
            .app_state
            .get_simulation_state()
            .is_running()
            .then_some(CursorIcon::Progress);
    }

    pub(crate) fn push_action(&mut self, action: Action) {
        self.pending_actions.push_back(action);
    }

    pub(crate) fn get_app_state(&self) -> AppState {
        self.app_state.clone()
    }

    pub(crate) fn new_design(&mut self) {
        self.clear_app_state(Default::default());
        self.update_current_file_name();
    }

    pub(crate) fn clear_app_state(&mut self, new_state: AppState) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.app_state = new_state.clone();
        self.last_saved_state = new_state;
    }

    pub(crate) fn update(&mut self) {
        log::trace!("call from main state");
        if let Some(camera_ptr) = self
            .applications
            .get(&GuiComponentType::StereographicScene)
            .and_then(|s| s.lock().unwrap().get_camera())
        {
            self.applications[&GuiComponentType::Scene]
                .lock()
                .unwrap()
                .on_notify(Notification::NewStereographicCamera(camera_ptr));
        }
        self.app_state.update();
    }

    pub(crate) fn update_candidates(&mut self, candidates: Vec<Selection>) {
        self.modify_state(|s| s.with_candidates(candidates), None);
    }

    pub(crate) fn transfer_selection_pivot_to_group(&mut self, group_id: GroupId) {
        let scene_pivot = self
            .applications
            .get(&GuiComponentType::Scene)
            .and_then(|app| app.lock().unwrap().get_current_selection_pivot());
        if let Some(pivot) = self.app_state.get_current_group_pivot().or(scene_pivot) {
            self.apply_operation(DesignOperation::SetGroupPivot { group_id, pivot });
        }
    }

    pub(crate) fn update_selection(
        &mut self,
        selection: Vec<Selection>,
        group_id: Option<GroupId>,
    ) {
        self.modify_state(
            |s| s.with_selection(selection, group_id),
            Some("Selection".into()),
        );
    }

    pub(crate) fn update_center_of_selection(&mut self, center: Option<CenterOfSelection>) {
        self.modify_state(|s| s.with_center_of_selection(center), None);
    }

    pub(crate) fn apply_copy_operation(&mut self, operation: CopyOperation) {
        let result = self.app_state.apply_copy_operation(operation);
        self.apply_operation_result(result);
    }

    pub(crate) fn apply_operation(&mut self, operation: DesignOperation) {
        log::debug!("Applying operation {operation:?}");
        let result = self.app_state.apply_design_op(operation.clone());
        if matches!(result, Err(ErrOperation::FinishFirst)) {
            self.modify_state(
                |s| s.notified(InteractorNotification::FinishOperation),
                None,
            );
            self.apply_operation(operation);
        } else {
            self.apply_operation_result(result);
        }
    }

    pub(crate) fn start_helix_simulation(&mut self, parameters: RigidBodyConstants) {
        let presenter = self.app_state.0.design.presenter.clone();
        let result = self
            .app_state
            .start_simulation(SimulationOperation::StartHelices {
                presenter: presenter.as_ref(),
                parameters,
                reader: &mut self.channel_reader,
            });
        self.apply_operation_result(result);
    }

    pub(crate) fn start_grid_simulation(&mut self, parameters: RigidBodyConstants) {
        let presenter = self.app_state.0.design.presenter.clone();
        let result = self
            .app_state
            .start_simulation(SimulationOperation::StartGrids {
                presenter: presenter.as_ref(),
                parameters,
                reader: &mut self.channel_reader,
            });
        self.apply_operation_result(result);
    }

    pub(crate) fn start_revolution_simulation(&mut self, desc: RevolutionSurfaceSystemDescriptor) {
        let result = self
            .app_state
            .start_simulation(SimulationOperation::RevolutionRelaxation {
                system: desc,
                reader: &mut self.channel_reader,
            });
        self.apply_operation_result(result);
    }

    pub(crate) fn start_twist(&mut self, grid_id: GridId) {
        let presenter = self.app_state.0.design.presenter.clone();
        let result = self
            .app_state
            .start_simulation(SimulationOperation::StartTwist {
                presenter: presenter.as_ref(),
                reader: &mut self.channel_reader,
                grid_id,
            });
        self.apply_operation_result(result);
    }

    pub(crate) fn start_roll_simulation(&mut self, target_helices: Option<Vec<usize>>) {
        let presenter = self.app_state.0.design.presenter.clone();
        let result = self
            .app_state
            .start_simulation(SimulationOperation::StartRoll {
                presenter: presenter.as_ref(),
                reader: &mut self.channel_reader,
                target_helices,
            });
        self.apply_operation_result(result);
    }

    pub(crate) fn update_rapier_parameters(&mut self, parameters: RapierParameters) {
        let presenter = self.app_state.0.design.presenter.clone();
        let result = self
            .app_state
            .start_simulation(SimulationOperation::UpdateRapierParameters {
                presenter: presenter.as_ref(),
                reader: &mut self.channel_reader,
                parameters,
            });
        self.apply_operation_result(result);
    }

    // NOTE : rename to apply_simulation_operation
    pub(crate) fn update_simulation(&mut self, request: SimulationOperation) {
        let result = self.app_state.update_simulation(request);
        self.apply_operation_result(result);
    }

    pub(crate) fn apply_silent_operation(&mut self, operation: DesignOperation) {
        match self.app_state.apply_design_op(operation.clone()) {
            Ok(_) => (),
            Err(ErrOperation::FinishFirst) => {
                self.modify_state(
                    |s| s.notified(InteractorNotification::FinishOperation),
                    None,
                );
                self.apply_silent_operation(operation);
            }
            Err(e) => log::warn!("{e:?}"),
        }
    }

    pub(crate) fn save_old_state(&mut self, old_state: AppState, label: TransitionLabel) {
        let camera_3d = self.get_camera_3d();
        self.undo_stack.push(AppStateTransition {
            state: old_state,
            label,
            camera_3d,
        });
        self.redo_stack.clear();
    }

    pub(crate) fn set_roll_of_selected_helices(&mut self, roll: f32) {
        if let Some((_, helices)) = list_of_helices(self.app_state.get_selection().as_ref()) {
            self.apply_operation(DesignOperation::SetRollHelices { helices, roll });
        }
    }

    pub(crate) fn undo(&mut self) {
        if let Some(mut transition) = self.undo_stack.pop() {
            transition.state.prepare_for_replacement(&self.app_state);
            let mut redo_state = std::mem::replace(&mut self.app_state, transition.state);
            redo_state = redo_state.notified(InteractorNotification::FinishOperation);
            self.set_camera_3d(transition.camera_3d.clone());
            self.messages
                .lock()
                .unwrap()
                .push_message(format!("UNDO: {}", transition.label.as_ref()));
            if redo_state.is_in_stable_state() {
                self.redo_stack.push(AppStateTransition {
                    state: redo_state,
                    label: transition.label,
                    camera_3d: transition.camera_3d,
                });
            }
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(mut transition) = self.redo_stack.pop() {
            transition.state.prepare_for_replacement(&self.app_state);
            let undo_state = std::mem::replace(&mut self.app_state, transition.state);
            self.set_camera_3d(transition.camera_3d.clone());
            self.messages
                .lock()
                .unwrap()
                .push_message(format!("REDO: {}", transition.label.as_ref()));
            self.undo_stack.push(AppStateTransition {
                state: undo_state,
                camera_3d: transition.camera_3d,
                label: transition.label,
            });
        }
    }

    pub(crate) fn modify_state<F>(&mut self, modification: F, undo_label: Option<TransitionLabel>)
    where
        F: FnOnce(AppState) -> AppState,
    {
        let state = std::mem::take(&mut self.app_state);
        let old_state = state.clone();
        self.app_state = modification(state);
        if let Some(label) = undo_label
            && old_state != self.app_state
            && old_state.is_in_stable_state()
        {
            let camera_3d = self.get_camera_3d();
            self.undo_stack.push(AppStateTransition {
                state: old_state,
                label,
                camera_3d,
            });
            self.redo_stack.clear();
        }
    }

    pub(crate) fn update_pending_operation(&mut self, operation: Arc<dyn Operation>) {
        let result = self.app_state.update_pending_operation(operation.clone());
        if matches!(result, Err(ErrOperation::FinishFirst)) {
            self.modify_state(
                |s| s.notified(InteractorNotification::FinishOperation),
                None,
            );
            self.update_pending_operation(operation);
        }
        self.apply_operation_result(result);
    }

    pub(crate) fn optimize_shift(&mut self) {
        let reader = &mut self.channel_reader;
        let result = self.app_state.optimize_shift(reader);
        self.apply_operation_result(result);
    }

    pub(crate) fn apply_operation_result(&mut self, result: Result<OkOperation, ErrOperation>) {
        match result {
            Ok(OkOperation::Undoable { state, label }) => self.save_old_state(state, label),
            Ok(OkOperation::NotUndoable) => (),
            Err(e) => log::warn!("{e:?}"),
        }
        if let Some(new_selection) = self.app_state.get_new_selection() {
            self.modify_state(|s| s.with_selection(new_selection, None), None);
        }
    }

    pub(crate) fn request_copy(&mut self) {
        let reader = self.app_state.get_design_interactor();
        let selection = self.app_state.get_selection();
        if let Some((_, xover_ids)) = list_of_xover_as_nucl_pairs(selection.as_ref(), &reader) {
            self.apply_copy_operation(CopyOperation::CopyXovers(xover_ids));
        } else if let Some(grid_ids) = extract_only_grids(selection.as_ref()) {
            self.apply_copy_operation(CopyOperation::CopyGrids(grid_ids));
        } else if let Some((_, helices)) = list_of_helices(selection.as_ref()) {
            self.apply_copy_operation(CopyOperation::CopyHelices(helices));
        } else {
            let strand_ids =
                extract_strands_from_selection(self.app_state.get_selection().as_ref());
            self.apply_copy_operation(CopyOperation::CopyStrands(strand_ids));
        }
    }

    pub(crate) fn apply_paste(&mut self) {
        log::info!("apply paste");
        match self.app_state.get_pasting_status() {
            PastingStatus::Copy => self.apply_copy_operation(CopyOperation::Paste),
            PastingStatus::Duplication => self.apply_copy_operation(CopyOperation::Duplicate),
            PastingStatus::None => log::info!("Not pasting"),
        }
    }

    pub(crate) fn request_duplication(&mut self) {
        if self.app_state.can_iterate_duplication() {
            self.apply_copy_operation(CopyOperation::Duplicate);
        } else if let Some((_, nucl_pairs)) = list_of_xover_as_nucl_pairs(
            self.app_state.get_selection().as_ref(),
            &self.app_state.get_design_interactor(),
        ) {
            self.apply_copy_operation(CopyOperation::InitXoverDuplication(nucl_pairs));
        } else if let Some((_, helices)) = list_of_helices(self.app_state.get_selection().as_ref())
        {
            self.apply_copy_operation(CopyOperation::InitHelicesDuplication(helices));
        } else {
            let strand_ids =
                extract_strands_from_selection(self.app_state.get_selection().as_ref());
            self.apply_copy_operation(CopyOperation::InitStrandsDuplication(strand_ids));
        }
    }

    pub(crate) fn save_design(&mut self, path: &PathBuf) -> Result<(), SaveDesignError> {
        let camera = self
            .applications
            .get(&GuiComponentType::Scene)
            .and_then(|s| s.lock().unwrap().get_camera())
            .map(|camera| Camera {
                id: Default::default(),
                name: String::from("Saved Camera"),
                position: camera.0.position,
                orientation: camera.0.orientation,
                pivot_position: camera.0.pivot_position,
            });
        let save_info = SavingInformation { camera };
        self.app_state.save_design(path, save_info)?;

        if self.app_state.is_in_stable_state() {
            self.last_saved_state = self.app_state.clone();
        }
        self.update_current_file_name();
        Ok(())
    }

    pub(crate) fn save_backup(&mut self) -> Result<(), SaveDesignError> {
        let camera = self
            .applications
            .get(&GuiComponentType::Scene)
            .and_then(|s| s.lock().unwrap().get_camera())
            .map(|camera| Camera {
                id: Default::default(),
                name: String::from("Saved Camera"),
                position: camera.0.position,
                orientation: camera.0.orientation,
                pivot_position: camera.0.pivot_position,
            });
        let save_info = SavingInformation { camera };
        let path = if let Some(mut path) = self.app_state.path_to_current_design().cloned() {
            path.set_extension(ENS_BACKUP_EXTENSION);
            path
        } else {
            let mut ret = dirs::document_dir()
                .or_else(dirs::home_dir)
                .ok_or_else(|| {
                    self.last_backup_date = Instant::now() + Duration::from_secs(SEC_PER_YEAR);
                    SaveDesignError::cannot_open_default_dir()
                })?;
            ret.push(ENS_UNNAMED_FILE_NAME);
            ret.set_extension(ENS_BACKUP_EXTENSION);
            ret
        };

        if self.app_state.is_in_stable_state() {
            self.app_state.save_design(&path, save_info)?;
            self.last_backed_up_state = self.app_state.clone();
            log::warn!("Saved backup to {}", path.to_string_lossy());
        }

        Ok(())
    }

    pub(crate) fn change_selection_mode(&mut self, mode: SelectionMode) {
        self.modify_state(|s| s.with_selection_mode(mode), None);
    }

    pub(crate) fn change_action_mode(&mut self, mode: ActionMode) {
        self.modify_state(|s| s.with_action_mode(mode), None);
    }

    pub(crate) fn change_double_strand_parameters(&mut self, parameters: Option<(isize, usize)>) {
        self.modify_state(|s| s.with_strand_on_helix(parameters), None);
    }

    pub(crate) fn toggle_widget_basis(&mut self) {
        self.modify_state(|s| s.with_toggled_widget_basis(), None);
    }

    pub(crate) fn set_visibility_sieve(&mut self, selection: Vec<Selection>, compl: bool) {
        let result = self.app_state.set_visibility_sieve(selection, compl);
        self.apply_operation_result(result);
    }

    pub(crate) fn need_save(&self) -> bool {
        self.app_state.design_was_modified(&self.last_saved_state)
    }

    pub(crate) fn get_current_file_name(&self) -> Option<&Path> {
        self.file_name.as_ref().map(AsRef::as_ref)
    }

    pub(crate) fn update_current_file_name(&mut self) {
        self.file_name = self
            .app_state
            .path_to_current_design()
            .as_ref()
            .filter(|p| p.is_file())
            .map(Into::into);
    }

    pub(crate) fn set_suggestion_parameters(&mut self, param: SuggestionParameters) {
        self.modify_state(|s| s.with_suggestion_parameters(param), None);
    }

    pub(crate) fn set_check_xovers_parameters(&mut self, param: CheckXoversParameter) {
        self.modify_state(|s| s.with_check_xovers_parameters(param), None);
    }

    pub(crate) fn set_follow_stereographic_camera(&mut self, follow: bool) {
        self.modify_state(|s| s.with_follow_stereographic_camera(follow), None);
    }

    pub(crate) fn set_show_stereographic_camera(&mut self, show: bool) {
        self.modify_state(|s| s.with_show_stereographic_camera(show), None);
    }

    pub(crate) fn set_show_h_bonds(&mut self, show: HBondDisplay) {
        self.modify_state(|s| s.with_show_h_bonds(show), None);
    }

    pub(crate) fn set_show_bezier_paths(&mut self, show: bool) {
        self.modify_state(|s| s.with_show_bezier_paths(show), None);
    }

    pub(crate) fn set_all_helices_on_axis(&mut self, off_axis: bool) {
        self.modify_state(|s| s.all_helices_on_axis(off_axis), None);
    }

    pub(crate) fn set_bezier_revolution_id(&mut self, id: Option<usize>) {
        self.modify_state(|s| s.set_bezier_revolution_id(id), None);
    }

    pub(crate) fn set_bezier_revolution_radius(&mut self, radius: f64) {
        self.modify_state(|s| s.set_bezier_revolution_radius(radius), None);
    }

    pub(crate) fn set_revolution_axis_position(&mut self, position: f64) {
        self.modify_state(|s| s.set_revolution_axis_position(position), None);
    }

    /// Create a bezier plane where the user is looking at if there are no bezier plane yet.
    pub(crate) fn create_default_bezier_plane(&mut self) {
        if self
            .app_state
            .get_design_interactor()
            .get_bezier_planes()
            .is_empty()
            && let Some((position, orientation)) = self.get_bezier_sheet_creation_position()
        {
            self.apply_operation(DesignOperation::AddBezierPlane {
                desc: BezierPlaneDescriptor {
                    position,
                    orientation,
                },
            });
        }
    }

    pub(crate) fn set_unrooted_surface(
        &mut self,
        surface: Option<UnrootedRevolutionSurfaceDescriptor>,
    ) {
        self.modify_state(|s| s.set_unrooted_surface(surface), None);
    }

    pub(crate) fn get_grid_creation_position(&self) -> Option<(Vec3, Rotor3)> {
        self.applications
            .get(&GuiComponentType::Scene)
            .and_then(|s| s.lock().unwrap().get_position_for_new_grid())
    }

    pub(crate) fn get_bezier_sheet_creation_position(&self) -> Option<(Vec3, Rotor3)> {
        self.get_grid_creation_position()
            .map(|(position, orientation)| {
                (
                    position - 30. * Vec3::unit_x().rotated_by(orientation),
                    orientation,
                )
            })
    }

    pub(crate) fn toggle_all_helices_on_axis(&mut self) {
        self.modify_state(|s| s.with_toggled_all_helices_on_axis(), None);
    }

    pub(crate) fn set_background_3d(&mut self, bg: Background3D) {
        self.modify_state(|s| s.with_background3d(bg), None);
    }

    pub(crate) fn set_rendering_mode(&mut self, rendering_mode: RenderingMode) {
        self.modify_state(|s| s.with_rendering_mode(rendering_mode), None);
    }

    pub(crate) fn set_scroll_sensitivity(&mut self, sensitivity: f32) {
        self.modify_state(|s| s.with_scroll_sensitivity(sensitivity), None);
    }

    pub(crate) fn set_invert_y_scroll(&mut self, inverted: bool) {
        self.modify_state(|s| s.with_inverted_y_scroll(inverted), None);
    }

    pub(crate) fn gui_state(&self, multiplexer: &Multiplexer) -> TopBarStateFlags {
        TopBarStateFlags {
            can_undo: !self.undo_stack.is_empty(),
            can_redo: !self.redo_stack.is_empty(),
            need_save: self.need_save(),
            can_reload: self.get_current_file_name().is_some(),
            can_split_2d: multiplexer.is_showing(&GuiComponentType::FlatScene),
            is_split_2d: self
                .applications
                .get(&GuiComponentType::FlatScene)
                .is_some_and(|app| app.lock().unwrap().is_split()),
            can_toggle_2d: multiplexer.is_showing(&GuiComponentType::FlatScene)
                || multiplexer.is_showing(&GuiComponentType::StereographicScene),
        }
    }

    pub(crate) fn get_camera_3d(&self) -> Camera3D {
        self.applications
            .get(&GuiComponentType::Scene)
            .expect("Could not get scene element")
            .lock()
            .unwrap()
            .get_camera()
            .unwrap()
            .as_ref()
            .clone()
            .0
    }

    pub(crate) fn set_camera_3d(&self, camera: Camera3D) {
        self.applications
            .get(&GuiComponentType::Scene)
            .expect("Could not get scene element")
            .lock()
            .unwrap()
            .on_notify(Notification::TeleportCamera(camera));
    }
}
