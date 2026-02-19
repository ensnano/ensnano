//! This module defines the `AppState` struct which implements various traits used by the
//! different components of ENSnano.
//!
//! The role of AppState is to provide information about the global state of the program, for
//! example the current selection, or the current state of the design.
//!
//! Each component of ENSnano has specific needs and express them via its own `AppState` trait.

pub mod action;
pub mod address_pointer;
pub mod channel_reader;
pub mod design_interactor;
pub mod impl_app2d;
pub mod impl_app3d;
pub mod impl_gui;
pub mod transitions;

use crate::{
    app_state::{
        address_pointer::AddressPointer,
        channel_reader::{ScaffoldShiftReader, SimulationInterfaceHandle},
        design_interactor::{
            DesignInteractor,
            controller::{
                Controller, InteractorNotification, OperationError, clipboard::CopyOperation,
                simulations::SimulationOperation,
            },
            presenter::SimulationUpdate,
        },
    },
    design::{
        operation::DesignOperation,
        selection::{CenterOfSelection, Selection},
    },
    operation::{AppStateOperationOutcome, AppStateOperationResult},
    utils::operation::SimpleOperation,
};
use ensnano_design::Design;
use ensnano_design::{
    SavingInformation,
    bezier_plane::BezierPathId,
    domains::Domain,
    group_attributes::GroupPivot,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::GroupId,
    scadnano::ScadnanoImportError,
};
use ensnano_utils::{
    PastingStatus, SimulationState, StrandBuildingStatus, WidgetBasis,
    app_state_parameters::{
        AppStateParameters, check_xovers_parameter::CheckXoversParameter,
        suggestion_parameters::SuggestionParameters,
    },
    consts::{APP_NAME, CANNOT_OPEN_DEFAULT_DIR, ENS_BACKUP_EXTENSION, ENS_EXTENSION},
    graphics::{Background3D, HBondDisplay, RenderingMode},
    surfaces::{RevolutionSurfaceRadius, UnrootedRevolutionSurfaceDescriptor},
    ui_size::UiSize,
};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use ultraviolet::{Rotor3, Vec3};

/// A structure containing the global state of the program.
///
/// At each event loop iteration, a new `AppState` may be created. Successive AppState are stored
/// on an undo/redo stack.
#[derive(Clone, PartialEq, Eq)]
pub struct AppState(pub AddressPointer<AppState_>);

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

impl std::fmt::Pointer for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ptr = self.0.get_ptr();
        std::fmt::Pointer::fmt(&ptr, f)
    }
}

impl Default for AppState {
    fn default() -> Self {
        let mut ret = Self(Default::default());
        log::trace!("call from default");
        // Synchronize all the pointers.
        // This turns updated_once to true so we must set it back to false afterwards
        ret.update();
        let mut with_forgot_update = ret.0.clone_inner();
        with_forgot_update.updated_once = false;
        Self(AddressPointer::new(with_forgot_update))
    }
}

impl AppState {
    pub fn with_preferred_parameters() -> Result<Self, confy::ConfyError> {
        let state: AppState_ = AppState_ {
            parameters: confy::load(APP_NAME, APP_NAME)?,
            ..Default::default()
        };
        let mut ret = Self(AddressPointer::new(state));
        log::trace!("call from default");
        // Synchronize all the pointers.
        // This turns updated_once to true so we must set it back to false afterwards
        ret.update();
        let mut with_forgot_update = ret.0.clone_inner();
        with_forgot_update.updated_once = false;
        Ok(Self(AddressPointer::new(with_forgot_update)))
    }

    // NOTE PACOME : this method is temporary while the overall structure
    // stays messy
    pub fn design_mut(&mut self) -> &mut Design {
        self.0.make_mut().design.make_mut().design.make_mut()
    }

    // NOTE PACOME : this method is temporary while the overall structure
    // stays messy
    pub fn design(&self) -> &Design {
        &self.0.design.design
    }

    // NOTE PACOME : this method is temporary while the structure
    // stays messy
    pub fn controller_mut(&mut self) -> &mut Controller {
        self.0.make_mut().design.make_mut().controller.make_mut()
    }

    // NOTE PACOME : this method is temporary while the structure
    // stays messy
    pub fn controller(&self) -> &Controller {
        &self.0.design.controller
    }

    // NOTE PACOME : this method is temporary while the structure
    // stays messy
    pub fn design_controller_mut(&mut self) -> (&mut Design, &mut Controller) {
        let binding = self.0.make_mut().design.make_mut();
        (binding.design.make_mut(), binding.controller.make_mut())
    }

    pub fn set_selection(
        &mut self,
        selection: &[Selection],
        selected_group: &Option<GroupId>,
    ) -> AppStateOperationResult {
        let mut selection = Vec::from(selection);
        let selected_group = *selected_group;
        selection.sort();
        selection.dedup();

        if self.0.selection.selection.content_equal(&selection)
            && selected_group == self.0.selection.selected_group
        {
            return Ok(AppStateOperationOutcome::NoOp);
        }

        let selection_len = selection.len();
        let state = self.0.make_mut();
        state.selection = AppStateSelection {
            selection: AddressPointer::new(selection),
            selected_group,
            pivot: Arc::new(RwLock::new(None)),
            old_pivot: Arc::new(RwLock::new(None)),
        };
        // Set when the selection is modified, the center of selection is set to None. It is up
        // to the caller to set it to a certain value when applicable
        state.center_of_selection = None;
        if selection_len > 0 {
            _ = self.notify(InteractorNotification::NewSelection);
        }

        Ok(AppStateOperationOutcome::Push {
            label: "Selection".into(),
        })
    }

    pub fn set_center_of_selection(
        &mut self,
        center: Option<CenterOfSelection>,
    ) -> AppStateOperationResult {
        if center == self.0.center_of_selection {
            return Ok(AppStateOperationOutcome::NoOp);
        }

        self.0.make_mut().center_of_selection = center;

        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_candidates(&mut self, candidates: &[Selection]) -> AppStateOperationResult {
        let mut candidates = Vec::from(candidates);

        candidates.sort();
        candidates.dedup();

        if self.0.candidates.content_equal(&candidates) {
            return Ok(AppStateOperationOutcome::NoOp);
        }

        *self.0.make_mut().candidates.make_mut() = candidates;

        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_selection_mode(&mut self, selection_mode: SelectionMode) -> AppStateOperationResult {
        self.0.make_mut().selection_mode = selection_mode;
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_suggestion_parameters(
        &mut self,
        suggestion_parameters: SuggestionParameters,
    ) -> AppStateOperationResult {
        self.0.make_mut().parameters.suggestion_parameters = suggestion_parameters;
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_ui_size(&mut self, ui_size: UiSize) -> AppStateOperationResult {
        self.update_parameters(|p| p.ui_size = ui_size)
    }

    pub fn set_action_mode(&mut self, action_mode: ActionMode) -> AppStateOperationResult {
        self.0.make_mut().action_mode = action_mode;
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_strand_on_helix(
        &mut self,
        parameters: Option<(isize, usize)>,
    ) -> AppStateOperationResult {
        let new_strand_parameters =
            parameters.map(|(start, length)| NewHelixStrand { length, start });
        if let ActionMode::BuildHelix { .. } = self.0.action_mode {
            let length = new_strand_parameters
                .as_ref()
                .map(|strand| strand.length)
                .unwrap_or_default();
            let start = new_strand_parameters
                .as_ref()
                .map(|strand| strand.start)
                .unwrap_or_default();
            let state = self.0.make_mut();
            state.strand_on_new_helix = new_strand_parameters;
            state.action_mode = ActionMode::BuildHelix {
                length,
                position: start,
            };
            return Ok(AppStateOperationOutcome::Replace);
        }

        Ok(AppStateOperationOutcome::NoOp)
    }

    pub fn set_exporting(&mut self, exporting: bool) -> AppStateOperationResult {
        self.0.make_mut().exporting = exporting;
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn toggle_widget_basis(&mut self) -> AppStateOperationResult {
        self.0.make_mut().widget_basis.toggle();
        Ok(AppStateOperationOutcome::Replace)
    }

    #[cfg(test)]
    pub fn update_design(&mut self, design: Design) -> AppStateOperationResult {
        self.0.make_mut().design.make_mut().update_design(design);
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn import_design(mut path: PathBuf) -> Result<Self, LoadDesignError> {
        let design_interactor = DesignInteractor::new_with_path(&path)?;
        if path.extension().map(|s| s.to_string_lossy()) != Some(ENS_BACKUP_EXTENSION.into()) {
            path.set_extension(ENS_EXTENSION);
        }

        let mut result = Self(AddressPointer::new(AppState_ {
            design: AddressPointer::new(design_interactor),
            parameters: confy::load(APP_NAME, APP_NAME).unwrap_or_default(),
            path_to_current_design: Some(path.clone()),
            ..Default::default()
        }));

        result.update();

        Ok(result)
    }

    pub fn save_design(
        &mut self,
        path: &PathBuf,
        saving_info: SavingInformation,
    ) -> Result<(), SaveDesignError> {
        self.get_design_interactor()
            .save_design(path, saving_info)?;
        self.0.make_mut().path_to_current_design = Some(path.clone());
        Ok(())
    }

    pub fn path_to_current_design(&self) -> Option<&PathBuf> {
        self.0.path_to_current_design.as_ref()
    }

    pub fn apply_simulation_update(&mut self, update: Box<dyn SimulationUpdate>) {
        let mut design = self.0.design.clone_inner();
        design = design.with_simulation_update_applied(update);
        self.set_interactor(design);
    }

    pub fn update(&mut self) {
        log::trace!("calling from updated!!");
        if self
            .0
            .design
            .design_need_update(&self.0.parameters.suggestion_parameters)
        {
            log::trace!("design need update");
            let mut interactor = self.0.design.clone_inner();
            interactor =
                interactor.with_updated_design_reader(&self.0.parameters.suggestion_parameters);
            self.set_interactor(interactor);
        }
    }

    fn set_interactor(&mut self, interactor: DesignInteractor) {
        let state = self.0.make_mut();
        state.updated_once = true;
        state.design = AddressPointer::new(interactor);
    }

    pub fn apply_design_op(&mut self, op: DesignOperation) -> AppStateOperationResult {
        Controller::apply_operation(self, op)
    }

    pub fn apply_copy_operation(&mut self, op: CopyOperation) -> AppStateOperationResult {
        self.0.make_mut().design.make_mut().apply_copy_operation(op)
    }

    pub fn update_pending_operation(
        &mut self,
        op: Arc<dyn SimpleOperation>,
    ) -> AppStateOperationResult {
        DesignInteractor::update_pending_operation(self, op)
    }

    pub fn update_simulation(&mut self, operation: SimulationOperation) -> AppStateOperationResult {
        let (outcome, interface) = self
            .0
            .make_mut()
            .design
            .make_mut()
            .update_simulation(operation)?;

        if let Some(interface) = interface {
            self.0
                .make_mut()
                .simulation_interface_handle
                .attach_state(&interface);
        }

        Ok(outcome)
    }

    pub fn notify(&mut self, notification: InteractorNotification) -> AppStateOperationResult {
        self.0.make_mut().design.make_mut().notify(notification);

        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn finish_operation(&self) {
        let pivot = *self.0.selection.pivot.read().unwrap();
        log::info!("Setting pivot {pivot:?}");
        log::info!("was {:?}", self.0.selection.old_pivot.read().unwrap());
        *self.0.selection.old_pivot.write().unwrap() = pivot;
        log::info!("is {:?}", self.0.selection.old_pivot.read().unwrap());
        log::debug!(
            "old pivot after reset {:p}",
            Arc::as_ptr(&self.0.selection.old_pivot)
        );
    }

    pub fn get_design_interactor(&self) -> DesignInteractor {
        self.0.design.clone_inner()
    }

    // pub fn export(&self, export_path: &PathBuf, export_type: ExportType) -> ExportResult {
    //     self.get_design_interactor()
    //         .export(export_path, export_type)
    // }

    pub fn get_selection(&self) -> &[Selection] {
        &self.0.selection.selection
    }

    pub fn is_changing_color(&self) -> bool {
        self.0.design.as_ref().is_changing_color()
    }

    pub fn prepare_for_replacement(&mut self, source: &Self) -> AppStateOperationResult {
        _ = self.set_candidates(&[]);
        _ = self.set_action_mode(source.0.action_mode);
        _ = self.set_selection_mode(source.0.selection_mode);
        _ = self.set_suggestion_parameters(source.0.parameters.suggestion_parameters);
        _ = self.set_check_xovers_parameters(source.0.parameters.check_xover_parameters);
        _ = self.update_parameters(|p| *p = source.0.parameters.clone());
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_check_xovers_parameters(
        &mut self,
        check_xover_parameters: CheckXoversParameter,
    ) -> AppStateOperationResult {
        self.update_parameters(|p| p.check_xover_parameters = check_xover_parameters)
    }

    pub fn set_follow_stereographic_camera(&mut self, follow: bool) -> AppStateOperationResult {
        self.update_parameters(|p| p.follow_stereography = follow)
    }

    pub fn set_show_stereographic_camera(&mut self, show: bool) -> AppStateOperationResult {
        self.update_parameters(|p| p.show_stereography = show)
    }

    pub fn show_h_bonds(&mut self, show: HBondDisplay) -> AppStateOperationResult {
        self.update_parameters(|p| p.show_h_bonds = show)
    }

    pub fn show_bezier_paths(&mut self, show: bool) -> AppStateOperationResult {
        self.update_parameters(|p| p.show_bezier_paths = show)
    }

    pub fn set_all_helices_on_axis(&mut self, on_axis: bool) -> AppStateOperationResult {
        self.update_parameters(|p| p.all_helices_on_axis = on_axis)
    }

    pub fn set_bezier_revolution_id(&mut self, id: Option<usize>) -> AppStateOperationResult {
        self.0.make_mut().unrooted_surface.bezier_path_id = id.map(|id| BezierPathId(id as u32));
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_bezier_revolution_radius(&mut self, radius: f64) -> AppStateOperationResult {
        self.0.make_mut().set_surface_revolution_radius(radius);
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_revolution_axis_position(&mut self, position: f64) -> AppStateOperationResult {
        self.0.make_mut().set_surface_axis_position(position);
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn set_unrooted_surface(
        &mut self,
        surface: &Option<UnrootedRevolutionSurfaceDescriptor>,
    ) -> AppStateOperationResult {
        if self.0.unrooted_surface.descriptor.as_ref() != surface.as_ref() {
            self.0.make_mut().set_unrooted_surface(surface.clone());
            return Ok(AppStateOperationOutcome::Replace);
        }

        Ok(AppStateOperationOutcome::NoOp)
    }

    pub fn toggle_all_helices_on_axis(&mut self) -> AppStateOperationResult {
        self.update_parameters(|p| p.all_helices_on_axis ^= true)
    }

    pub fn set_background3d(&mut self, bg: Background3D) -> AppStateOperationResult {
        self.update_parameters(|p| p.background3d = bg)
    }

    pub fn set_rendering_mode(&mut self, rendering_mode: RenderingMode) -> AppStateOperationResult {
        self.update_parameters(|p| p.rendering_mode = rendering_mode)
    }

    pub fn set_scroll_sensitivity(&mut self, sensitivity: f32) -> AppStateOperationResult {
        self.update_parameters(|p| p.scroll_sensitivity = sensitivity)
    }

    pub fn set_inverted_y_scroll(&mut self, inverted: bool) -> AppStateOperationResult {
        self.update_parameters(|p| p.inverted_y_scroll = inverted)
    }

    fn update_parameters<F>(&mut self, update: F) -> AppStateOperationResult
    where
        F: Fn(&mut AppStateParameters),
    {
        update(&mut self.0.make_mut().parameters);
        if let Err(e) = confy::store(APP_NAME, APP_NAME, self.0.parameters.clone()) {
            log::error!("Could not save preferences {e:?}");
        }
        Ok(AppStateOperationOutcome::Replace)
    }

    pub fn get_pasting_status(&self) -> PastingStatus {
        self.0.design.get_pasting_status()
    }

    pub fn can_iterate_duplication(&self) -> bool {
        self.0.design.can_iterate_duplication()
    }

    pub fn optimize_shift(&mut self) -> AppStateOperationResult {
        let mut reader = self.0.channel_reader.clone();
        self.0
            .make_mut()
            .design
            .make_mut()
            .optimize_shift(&mut reader)
    }

    pub fn is_in_stable_state(&self) -> bool {
        self.0.design.is_in_stable_state()
    }

    pub fn set_visibility_sieve(
        &mut self,
        selection: Vec<Selection>,
        compl: bool,
    ) -> AppStateOperationResult {
        self.0
            .make_mut()
            .design
            .make_mut()
            .set_visibility_sieve(selection, compl)
    }

    pub fn design_was_modified(&self, other: &Self) -> bool {
        self.0.design.has_different_design_than(&other.0.design)
            && (self.0.updated_once || other.0.updated_once)
    }

    pub fn get_strand_building_state(&self) -> Option<StrandBuildingStatus> {
        let builders = self.0.design.get_strand_builders();
        builders.first().and_then(|b| {
            let domain_id = b.get_domain_identifier();
            let reader = self.get_design_interactor();
            let domain = reader.get_strand_domain(domain_id.strand, domain_id.domain)?;
            let param = self.0.design.get_dna_parameters();
            if let Domain::HelixDomain(interval) = domain {
                let prime5 = interval.prime5();
                let prime3 = interval.prime3();
                let nt_length = domain.length();
                Some(StrandBuildingStatus {
                    prime5,
                    prime3,
                    nt_length,
                    nm_length: param.rise * nt_length as f32,
                    dragged_nucl: b.moving_end,
                })
            } else {
                None
            }
        })
    }

    fn selection_content(&self) -> &AddressPointer<Vec<Selection>> {
        &self.0.selection.selection
    }

    pub fn get_current_group_id(&self) -> Option<GroupId> {
        self.0.selection.selected_group
    }

    pub fn set_current_group_pivot(&self, pivot: GroupPivot) {
        if self.0.selection.pivot.read().unwrap().is_none() {
            log::info!("resetting selection pivot {pivot:?}");
            *self.0.selection.pivot.write().unwrap() = Some(pivot);
            *self.0.selection.old_pivot.write().unwrap() = Some(pivot);
            log::debug!(
                "old pivot after reset {:p}",
                Arc::as_ptr(&self.0.selection.old_pivot)
            );
        }
    }

    pub fn translate_group_pivot(&self, translation: Vec3) {
        log::debug!("old pivot {:p}", Arc::as_ptr(&self.0.selection.old_pivot));
        log::info!("is {:?}", self.0.selection.old_pivot.read().unwrap());
        let new_pivot = {
            if let Some(Some(mut old_pivot)) =
                self.0.selection.old_pivot.read().as_deref().ok().copied()
            {
                old_pivot.position += translation;
                old_pivot
            } else {
                log::error!("Translating a pivot that does not exist");
                return;
            }
        };
        *self.0.selection.pivot.write().unwrap() = Some(new_pivot);
    }

    pub fn rotate_group_pivot(&self, rotation: Rotor3) {
        log::debug!("old pivot {:p}", Arc::as_ptr(&self.0.selection.old_pivot));
        log::info!("is {:?}", self.0.selection.old_pivot.read().unwrap());
        let new_pivot = {
            if let Some(Some(mut old_pivot)) =
                self.0.selection.old_pivot.read().as_deref().ok().copied()
            {
                old_pivot.orientation = rotation * old_pivot.orientation;
                old_pivot
            } else {
                log::error!("Rotating a pivot that does not exist");
                return;
            }
        };
        *self.0.selection.pivot.write().unwrap() = Some(new_pivot);
    }

    pub fn get_simulation_state(&self) -> SimulationState {
        self.0.design.get_simulation_state()
    }

    pub fn set_expand_insertion_set(&mut self, expand: bool) -> AppStateOperationResult {
        self.0.make_mut().show_insertion_discriminants = !expand;
        Ok(AppStateOperationOutcome::Replace)
    }
}

#[derive(Clone, Default)]
pub struct AppState_ {
    /// The set of currently selected objects.
    pub selection: AppStateSelection,
    /// The set of objects that are "one click away from being selected".
    pub candidates: AddressPointer<Vec<Selection>>,
    pub selection_mode: SelectionMode,
    /// A pointer to the design currently being edited. The pointed design is never mutated.
    /// Instead, when a modification is requested, the design is cloned and the `design` pointer is
    /// replaced by a pointer to a modified `Design`.
    pub design: AddressPointer<DesignInteractor>,
    pub action_mode: ActionMode,
    pub widget_basis: WidgetBasis,
    pub strand_on_new_helix: Option<NewHelixStrand>,
    pub center_of_selection: Option<CenterOfSelection>,
    pub updated_once: bool,
    pub parameters: AppStateParameters,
    pub show_insertion_discriminants: bool,
    pub exporting: bool,
    pub path_to_current_design: Option<PathBuf>,
    pub unrooted_surface: CurrentUnrootedSurface,
    pub simulation_interface_handle: SimulationInterfaceHandle,
    /// channel reader for simulations.
    pub channel_reader: ScaffoldShiftReader,
}

#[derive(Clone, Default)]
pub struct CurrentUnrootedSurface {
    descriptor: Option<UnrootedRevolutionSurfaceDescriptor>,
    bezier_path_id: Option<BezierPathId>,
    area: Option<f64>,
}

impl AppState_ {
    fn set_unrooted_surface(&mut self, surface: Option<UnrootedRevolutionSurfaceDescriptor>) {
        self.unrooted_surface.area = surface
            .as_ref()
            .and_then(|s| s.approx_surface_area(1_000, 1_000));
        self.unrooted_surface.descriptor = surface;
    }

    fn set_surface_revolution_radius(&mut self, radius: f64) {
        let mut new_surface = self.unrooted_surface.descriptor.clone();
        if let Some(s) = new_surface.as_mut() {
            s.revolution_radius = RevolutionSurfaceRadius::from_signed_f64(radius);
        }
        self.set_unrooted_surface(new_surface);
    }

    fn set_surface_axis_position(&mut self, position: f64) {
        let mut new_surface = self.unrooted_surface.descriptor.clone();
        if let Some(s) = new_surface.as_mut() {
            s.set_axis_position(position);
        }
        self.set_unrooted_surface(new_surface);
    }
}

#[derive(Clone, Default)]
pub struct AppStateSelection {
    selection: AddressPointer<Vec<Selection>>,
    selected_group: Option<GroupId>,
    pivot: Arc<RwLock<Option<GroupPivot>>>,
    old_pivot: Arc<RwLock<Option<GroupPivot>>>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct NewHelixStrand {
    length: usize,
    start: isize,
}
pub enum LoadDesignError {
    JsonError(serde_json::Error),
    ScadnanoImportError(ScadnanoImportError),
    IncompatibleVersion { current: String, required: String },
}

impl std::fmt::Display for LoadDesignError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::JsonError(e) => write!(f, "Json error: {e}"),
            Self::ScadnanoImportError(e) => {
                write!(
                    f,
                    "Scadnano file detected but the following error was encountered:
                {e:?}",
                )
            }
            Self::IncompatibleVersion { current, required } => {
                write!(
                    f,
                    "Your ENSnano version is too old to load this design.
                Your version: {current},
                Required version: {required}"
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct SaveDesignError(pub String);

impl<E: std::error::Error> From<E> for SaveDesignError {
    fn from(e: E) -> Self {
        Self(format!("{e}"))
    }
}

impl SaveDesignError {
    pub fn cannot_open_default_dir() -> Self {
        Self(CANNOT_OPEN_DEFAULT_DIR.to_owned())
    }
}
