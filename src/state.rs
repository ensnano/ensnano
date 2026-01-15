use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use ahash::HashMap;
use ensnano_design::{
    Camera, CameraId, MainDesignReaderExt, SavingInformation,
    bezier_plane::BezierPlaneDescriptor,
    grid::GridId,
    group_attributes::GroupPivot,
    interaction_modes::{ActionMode, SelectionMode},
    operation::{DesignOperation, DesignRotation, DesignTranslation, IsometryTarget},
    organizer_tree::GroupId,
    selection::{
        CenterOfSelection, Selection, extract_nucls_from_selection, extract_only_grids,
        extract_strands_from_selection, list_of_bezier_vertices, list_of_free_grids,
        list_of_helices, list_of_strands, list_of_xover_as_nucl_pairs,
    },
};
use ensnano_exports::{ExportResult, ExportType};
use ensnano_gui::{Gui, GuiAppState as _, IcedMessages, TopBarState};
use ensnano_physics::parameters::RapierParameters;
use ensnano_scene::{SceneAppState as _, data::design3d::SceneDesignReaderExt as _};
use ensnano_utils::{
    PastingStatus, RigidBodyConstants,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    application::{Application, Camera3D, Notification},
    consts::{ENS_BACKUP_EXTENSION, ENS_UNNAMED_FILE_NAME, SEC_BETWEEN_BACKUPS, SEC_PER_YEAR},
    graphics::{Background3D, GuiComponentType, HBondDisplay, RenderingMode, SplitMode},
    keyboard_priority::KeyboardPriorityId,
    operation::Operation,
    surfaces::{RevolutionSurfaceSystemDescriptor, UnrootedRevolutionSurfaceDescriptor},
    ui_size::UiSize,
};
use ultraviolet::{Rotor3, Vec3};
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{CursorIcon, Window},
};

use crate::{
    app_state::{
        AppState,
        design_interactor::{
            DesignInteractor,
            controller::{
                ErrOperation, InteractorNotification,
                clipboard::{CopyOperation, PastePosition},
                simulations::SimulationOperation,
            },
        },
        transitions::{AppStateTransition, OkOperation, TransitionLabel},
    },
    controller::{
        LoadDesignError, SaveDesignError,
        channel_reader::ChannelReader,
        normal_state::Action,
        set_scaffold_sequence::{
            SetScaffoldSequenceError, SetScaffoldSequenceOk, TargetScaffoldLength,
        },
    },
    multiplexer::Multiplexer,
    requests::Requests,
    scheduler::Scheduler,
};

/// The state of the main event loop.
pub(crate) struct MainState {
    pub app_state: AppState,
    pub pending_actions: VecDeque<Action>,
    pub undo_stack: Vec<AppStateTransition>,
    pub redo_stack: Vec<AppStateTransition>,
    pub channel_reader: ChannelReader,
    pub messages: Arc<Mutex<IcedMessages<AppState>>>,
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
    pub(crate) fn new(messages: Arc<Mutex<IcedMessages<AppState>>>) -> Self {
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

    pub(crate) fn gui_state(&self, multiplexer: &Multiplexer) -> TopBarState {
        TopBarState {
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

/// A temporary view of the main state and the control flow.
pub(crate) struct MainStateView<'a> {
    pub(crate) main_state: &'a mut MainState,
    pub(crate) window_target: &'a EventLoopWindowTarget<()>,
    pub(crate) multiplexer: &'a mut Multiplexer,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) gui: &'a mut Gui<Requests, AppState>,
    pub(crate) window: &'a Window,
    pub(crate) resized: bool,
}

impl MainStateView<'_> {
    pub(crate) fn pop_action(&mut self) -> Option<Action> {
        if !self.main_state.pending_actions.is_empty() {
            log::debug!("pending actions {:?}", self.main_state.pending_actions);
        }
        self.main_state.pending_actions.pop_front()
    }

    pub(crate) fn check_backup(&mut self) {
        if !self
            .main_state
            .last_backed_up_state
            .design_was_modified(&self.main_state.app_state)
            || !self
                .main_state
                .last_saved_state
                .design_was_modified(&self.main_state.app_state)
        {
            self.main_state.last_backup_date = Instant::now();
        }
    }

    pub(crate) fn main_state(&mut self) -> &mut MainState {
        self.main_state
    }

    pub(crate) fn need_backup(&self) -> bool {
        self.main_state.last_backup_date.elapsed() > Duration::from_secs(SEC_BETWEEN_BACKUPS)
    }

    pub(crate) fn exit_control_flow(&self) {
        self.window_target.exit();
    }

    pub(crate) fn new_design(&mut self) {
        self.notify_apps(Notification::ClearDesigns);
        self.main_state.new_design();
    }

    pub(crate) fn export(&mut self, path: &PathBuf, export_type: ExportType) -> ExportResult {
        let ret = self.main_state.app_state.export(path, export_type);
        self.set_exporting(false);
        ret
    }

    pub(crate) fn load_design(&mut self, path: PathBuf) -> Result<(), LoadDesignError> {
        let state = AppState::import_design(path)?;
        self.notify_apps(Notification::ClearDesigns);
        self.main_state.clear_app_state(state);
        if let Some((position, orientation)) = self
            .main_state
            .app_state
            .get_design_interactor()
            .get_favorite_camera()
        {
            self.notify_apps(Notification::TeleportCamera(Camera3D {
                position,
                orientation,
                pivot_position: None,
            }));
        } else {
            self.main_state.wants_fit = true;
        }
        self.main_state.update_current_file_name();
        Ok(())
    }

    pub(crate) fn apply_operation(&mut self, operation: DesignOperation) {
        self.main_state.apply_operation(operation);
    }

    pub(crate) fn apply_silent_operation(&mut self, operation: DesignOperation) {
        self.main_state.apply_silent_operation(operation);
    }

    pub(crate) fn undo(&mut self) {
        self.main_state.undo();
    }

    pub(crate) fn redo(&mut self) {
        self.main_state.redo();
    }

    pub(crate) fn get_design_interactor(&self) -> DesignInteractor {
        self.main_state.app_state.get_design_interactor()
    }

    pub(crate) fn save_design(&mut self, path: &PathBuf) -> Result<(), SaveDesignError> {
        self.main_state.save_design(path)?;
        self.main_state.last_backup_date = Instant::now();
        Ok(())
    }

    pub(crate) fn save_backup(&mut self) -> Result<(), SaveDesignError> {
        self.main_state.save_backup()?;
        self.main_state.last_backup_date = Instant::now();
        Ok(())
    }

    pub(crate) fn toggle_split_mode(&mut self, mode: SplitMode) {
        self.multiplexer.change_split(mode);
        self.scheduler
            .forward_new_size(self.window.inner_size(), self.multiplexer);
        self.gui.resize(self.multiplexer, self.window);
    }

    pub(crate) fn change_ui_size(&mut self, ui_size: UiSize) {
        self.gui.new_ui_size(
            ui_size,
            self.window,
            self.multiplexer,
            &self.main_state.app_state,
            self.main_state.gui_state(self.multiplexer),
        );
        self.multiplexer.change_ui_size(ui_size, self.window);
        self.main_state
            .messages
            .lock()
            .unwrap()
            .new_ui_size(ui_size);
        self.main_state
            .modify_state(|s| s.with_ui_size(ui_size), None);
        self.resized = true;
    }

    pub(crate) fn notify_apps(&mut self, notification: Notification) {
        log::info!("Notify apps {notification:?}");
        for app in self.main_state.applications.values_mut() {
            app.lock().unwrap().on_notify(notification.clone());
        }
    }

    pub(crate) fn get_selection(&self) -> Box<dyn AsRef<[Selection]>> {
        Box::new(self.main_state.app_state.get_selection())
    }

    pub(crate) fn get_design_reader(&self) -> Box<dyn MainDesignReaderExt> {
        Box::new(self.main_state.app_state.get_design_interactor())
    }

    pub(crate) fn get_grid_creation_position(&self) -> Option<(Vec3, Rotor3)> {
        self.main_state.get_grid_creation_position()
    }

    pub(crate) fn get_bezier_sheet_creation_position(&self) -> Option<(Vec3, Rotor3)> {
        self.main_state.get_bezier_sheet_creation_position()
    }

    pub(crate) fn finish_operation(&mut self) {
        self.main_state.modify_state(
            |s| s.notified(InteractorNotification::FinishOperation),
            None,
        );
        self.main_state.app_state.finish_operation();
    }

    pub(crate) fn request_copy(&mut self) {
        self.main_state.request_copy();
    }

    pub(crate) fn init_paste(&mut self) {
        self.main_state
            .apply_copy_operation(CopyOperation::PositionPastingPoint(None));
    }

    pub(crate) fn apply_paste(&mut self) {
        self.main_state.apply_paste();
    }

    pub(crate) fn duplicate(&mut self) {
        self.main_state.request_duplication();
    }

    pub(crate) fn request_pasting_candidate(&mut self, candidate: Option<PastePosition>) {
        self.main_state
            .apply_copy_operation(CopyOperation::PositionPastingPoint(candidate));
    }

    pub(crate) fn delete_selection(&mut self) {
        let selection = self.get_selection();
        if let Some((_, nucl_pairs)) = list_of_xover_as_nucl_pairs(
            selection.as_ref().as_ref(),
            self.get_design_reader().as_ref(),
        ) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmXovers { xovers: nucl_pairs });
        } else if let Some((_, strand_ids)) = list_of_strands(selection.as_ref().as_ref()) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmStrands { strand_ids });
        } else if let Some((_, h_ids)) = list_of_helices(selection.as_ref().as_ref()) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmHelices { h_ids });
        } else if let Some(grid_ids) = list_of_free_grids(selection.as_ref().as_ref()) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmFreeGrids { grid_ids });
        } else if let Some(vertices) = list_of_bezier_vertices(selection.as_ref().as_ref()) {
            self.main_state.update_selection(vec![], None);
            self.main_state
                .apply_operation(DesignOperation::RmBezierVertices { vertices });
        }
    }

    pub(crate) fn scaffold_to_selection(&mut self) {
        let scaffold_id = self
            .main_state
            .get_app_state()
            .get_design_interactor()
            .get_scaffold_info()
            .map(|info| info.id);
        if let Some(s_id) = scaffold_id {
            self.main_state
                .update_selection(vec![Selection::Strand(0, s_id as u32)], None);
        }
    }

    pub(crate) fn start_helix_simulation(&mut self, parameters: RigidBodyConstants) {
        self.main_state.start_helix_simulation(parameters);
    }

    pub(crate) fn start_grid_simulation(&mut self, parameters: RigidBodyConstants) {
        self.main_state.start_grid_simulation(parameters);
    }

    pub(crate) fn start_revolution_simulation(&mut self, desc: RevolutionSurfaceSystemDescriptor) {
        self.main_state.start_revolution_simulation(desc);
    }

    pub(crate) fn start_roll_simulation(&mut self, target_helices: Option<Vec<usize>>) {
        self.main_state.start_roll_simulation(target_helices);
    }

    pub(crate) fn update_simulation(&mut self, request: SimulationOperation) {
        self.main_state.update_simulation(request);
    }

    pub(crate) fn set_roll_of_selected_helices(&mut self, roll: f32) {
        self.main_state.set_roll_of_selected_helices(roll);
    }

    pub(crate) fn turn_selection_into_anchor(&mut self) {
        let selection = self.get_selection();
        let nucls = extract_nucls_from_selection(selection.as_ref().as_ref());
        self.main_state
            .apply_operation(DesignOperation::FlipAnchors { nucls });
    }

    pub(crate) fn set_visibility_sieve(&mut self, compl: bool) {
        let selection = self.get_selection().as_ref().as_ref().to_vec();
        self.main_state.set_visibility_sieve(selection, compl);
    }

    pub(crate) fn clear_visibility_sieve(&mut self) {
        self.main_state.set_visibility_sieve(vec![], true);
    }

    pub(crate) fn need_save(&self) -> Option<Option<PathBuf>> {
        self.main_state
            .need_save()
            .then(|| self.get_current_file_name().map(Path::to_path_buf))
    }

    pub(crate) fn get_current_design_directory(&self) -> Option<&Path> {
        let mut ancestors = self
            .main_state
            .app_state
            .path_to_current_design()
            .as_ref()
            .map(|p| p.ancestors())?;
        let first_ancestor = ancestors.next()?;
        if first_ancestor.is_dir() {
            Some(first_ancestor)
        } else {
            let second_ancestor = ancestors.next()?;
            second_ancestor.is_dir().then_some(second_ancestor)
        }
    }

    pub(crate) fn get_current_file_name(&self) -> Option<&Path> {
        self.main_state.get_current_file_name()
    }

    pub(crate) fn get_design_path_and_notify(
        &mut self,
        notificator: fn(Option<Arc<Path>>) -> Notification,
    ) {
        if let Some(filename) = self.get_current_file_name() {
            self.main_state
                .push_action(Action::NotifyApps(notificator(Some(Arc::from(filename)))));
        } else {
            println!("Design has not been saved yet");
            self.main_state
                .push_action(Action::NotifyApps(notificator(None)));
        }
    }

    pub(crate) fn set_current_group_pivot(&mut self, pivot: GroupPivot) {
        if let Some(group_id) = self.main_state.app_state.get_current_group_id() {
            self.apply_operation(DesignOperation::SetGroupPivot { group_id, pivot });
        } else {
            self.main_state.app_state.set_current_group_pivot(pivot);
        }
    }

    pub(crate) fn translate_group_pivot(&mut self, translation: Vec3) {
        if let Some(group_id) = self.main_state.app_state.get_current_group_id() {
            self.apply_operation(DesignOperation::Translation(DesignTranslation {
                target: IsometryTarget::GroupPivot(group_id),
                translation,
                group_id: None,
            }));
        } else {
            self.main_state.app_state.translate_group_pivot(translation);
        }
    }

    pub(crate) fn rotate_group_pivot(&mut self, rotation: Rotor3) {
        if let Some(group_id) = self.main_state.app_state.get_current_group_id() {
            self.apply_operation(DesignOperation::Rotation(DesignRotation {
                target: IsometryTarget::GroupPivot(group_id),
                rotation,
                origin: Vec3::zero(),
                group_id: None,
            }));
        } else {
            self.main_state.app_state.rotate_group_pivot(rotation);
        }
    }

    pub(crate) fn create_new_camera(&mut self) {
        if let Some(camera) = self
            .main_state
            .applications
            .get(&GuiComponentType::Scene)
            .and_then(|s| s.lock().unwrap().get_camera())
        {
            self.main_state
                .apply_operation(DesignOperation::CreateNewCamera {
                    position: camera.0.position,
                    orientation: camera.0.orientation,
                    pivot_position: camera.0.pivot_position,
                });
        } else {
            log::error!("Could not get current camera position");
        }
    }

    pub(crate) fn select_camera(&mut self, camera_id: CameraId) {
        let reader = self.main_state.app_state.get_design_interactor();
        if let Some(camera) = reader.get_camera_with_id(camera_id) {
            self.notify_apps(Notification::TeleportCamera(camera));
        } else {
            log::error!("Could not get camera {camera_id:?}");
        }
    }

    pub(crate) fn select_favorite_camera(&mut self, n_camera: u32) {
        let reader = self.main_state.app_state.get_design_interactor();
        if let Some(camera) = reader.get_nth_camera(n_camera) {
            self.notify_apps(Notification::TeleportCamera(camera));
        } else {
            log::error!("Design has less than {} cameras", n_camera + 1);
        }
    }

    pub(crate) fn toggle_2d(&mut self) {
        self.multiplexer.toggle_2d();
        self.scheduler
            .forward_new_size(self.window.inner_size(), self.multiplexer);
    }

    pub(crate) fn make_all_suggested_xover(&mut self, doubled: bool) {
        let reader = self.main_state.app_state.get_design_interactor();
        let xovers = reader.get_suggestions();
        self.apply_operation(DesignOperation::MakeSeveralXovers { xovers, doubled });
    }

    pub(crate) fn flip_split_views(&mut self) {
        self.notify_apps(Notification::FlipSplitViews);
    }

    pub(crate) fn start_twist(&mut self, g_id: GridId) {
        self.main_state.start_twist(g_id);
    }

    pub(crate) fn set_expand_insertions(&mut self, expand: bool) {
        self.main_state
            .modify_state(|app| app.with_expand_insertion_set(expand), None);
    }

    pub(crate) fn set_exporting(&mut self, exporting: bool) {
        self.main_state
            .modify_state(|app| app.exporting(exporting), None);
    }

    pub(crate) fn load_3d_object(&mut self, path: PathBuf) {
        let design_path = self
            .get_current_design_directory()
            .map(Path::to_path_buf)
            .or_else(dirs::home_dir)
            .unwrap();
        self.apply_operation(DesignOperation::Add3DObject {
            file_path: path,
            design_path,
        });
    }

    pub(crate) fn load_svg(&mut self, path: PathBuf) {
        self.apply_operation(DesignOperation::ImportSvgPath { path });
    }

    pub(crate) fn set_scaffold_sequence(
        &mut self,
        sequence: String,
        shift: usize,
    ) -> Result<SetScaffoldSequenceOk, SetScaffoldSequenceError> {
        let len = sequence.chars().filter(|c| c.is_alphabetic()).count();
        match self
            .main_state
            .app_state
            .apply_design_op(DesignOperation::SetScaffoldSequence { sequence, shift })
        {
            Ok(OkOperation::Undoable { state, label }) => {
                self.main_state.save_old_state(state, label);
            }
            Ok(OkOperation::NotUndoable) => (),
            Err(e) => return Err(SetScaffoldSequenceError(format!("{e:?}"))),
        }
        let default_shift = self.get_design_interactor().default_shift();
        let scaffold_length = self.get_scaffold_length().unwrap_or(0);
        let target_scaffold_length = if len == scaffold_length {
            TargetScaffoldLength::Ok
        } else {
            TargetScaffoldLength::NotOk {
                design_length: scaffold_length,
                input_scaffold_length: len,
            }
        };
        Ok(SetScaffoldSequenceOk {
            default_shift,
            target_scaffold_length,
        })
    }

    pub(crate) fn optimize_shift(&mut self) {
        self.main_state.optimize_shift();
    }

    pub(crate) fn get_scaffold_length(&self) -> Option<usize> {
        self.main_state
            .app_state
            .get_scaffold_info()
            .map(|info| info.length)
    }
}
