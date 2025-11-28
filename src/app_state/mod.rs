//! This module defines the `AppState` struct which implements various traits used by the
//! different components of ENSnano.
//!
//! The role of AppState is to provide information about the global state of the program, for
//! example the current selection, or the current state of the design.
//!
//! Each component of ENSnano has specific needs and express them via its own `AppState` trait.

pub mod address_pointer;
pub mod design_interactor;
pub mod impl_app2d;
pub mod impl_app3d;
pub mod impl_gui;
pub mod transitions;

#[cfg(test)]
use crate::ensnano_design::Design;

use crate::ensnano_consts::{APP_NAME, ENS_BACKUP_EXTENSION, ENS_EXTENSION};
use crate::ensnano_design::{
    SavingInformation, bezier_plane::BezierPathId, group_attributes::GroupPivot, strands::Domain,
};
use crate::ensnano_exports::{ExportResult, ExportType};
use crate::ensnano_iced::ui_size::UiSize;
use crate::ensnano_interactor::{
    DesignOperation, PastingStatus, StrandBuildingStatus, WidgetBasis,
    app_state_parameters::{
        AppStateParameters, check_xovers_parameter::CheckXoversParameter,
        suggestion_parameters::SuggestionParameters,
    },
    graphics::{Background3D, HBondDisplay, RenderingMode},
    operation::Operation,
    selection::{ActionMode, CenterOfSelection, Selection, SelectionMode},
    surfaces::{RevolutionSurfaceRadius, UnrootedRevolutionSurfaceDescriptor},
};
use crate::ensnano_organizer::tree::GroupId;
use crate::{
    app_state::design_interactor::{
        controller::{
            InteractorNotification, clipboard::CopyOperation, simulations::SimulationOperation,
        },
        presenter::SimulationUpdate,
    },
    apply_update,
    controller::{LoadDesignError, SaveDesignError, channel_reader::ChannelReader},
};
use address_pointer::AddressPointer;
use design_interactor::{DesignInteractor, InteractorResult, controller::ErrOperation};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use transitions::OkOperation;
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
        ret = ret.updated();
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
        ret = ret.updated();
        let mut with_forgot_update = ret.0.clone_inner();
        with_forgot_update.updated_once = false;
        Ok(Self(AddressPointer::new(with_forgot_update)))
    }

    pub fn with_selection(
        &self,
        mut selection: Vec<Selection>,
        selected_group: Option<GroupId>,
    ) -> Self {
        selection.sort();
        selection.dedup();
        if self.0.selection.selection.content_equal(&selection)
            && selected_group == self.0.selection.selected_group
        {
            self.clone()
        } else {
            let mut new_state = (*self.0).clone();
            let selection_len = selection.len();
            new_state.selection = AppStateSelection {
                selection: AddressPointer::new(selection),
                selected_group,
                pivot: Arc::new(RwLock::new(None)),
                old_pivot: Arc::new(RwLock::new(None)),
            };
            // Set when the selection is modified, the center of selection is set to None. It is up
            // to the caller to set it to a certain value when applicable
            new_state.center_of_selection = None;
            let mut ret = Self(AddressPointer::new(new_state));
            if selection_len > 0 {
                ret = ret.notified(InteractorNotification::NewSelection);
            }
            ret
        }
    }

    pub fn with_center_of_selection(&self, center: Option<CenterOfSelection>) -> Self {
        if center == self.0.center_of_selection {
            self.clone()
        } else {
            let mut new_state = (*self.0).clone();
            new_state.center_of_selection = center;
            Self(AddressPointer::new(new_state))
        }
    }

    pub fn with_candidates(&self, mut candidates: Vec<Selection>) -> Self {
        candidates.sort();
        candidates.dedup();
        if self.0.candidates.content_equal(&candidates) {
            self.clone()
        } else {
            let mut new_state = (*self.0).clone();
            new_state.candidates = AddressPointer::new(candidates);
            Self(AddressPointer::new(new_state))
        }
    }

    pub fn with_selection_mode(&self, selection_mode: SelectionMode) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.selection_mode = selection_mode;
        Self(AddressPointer::new(new_state))
    }

    pub fn with_suggestion_parameters(&self, suggestion_parameters: SuggestionParameters) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.parameters.suggestion_parameters = suggestion_parameters;
        Self(AddressPointer::new(new_state))
    }

    pub fn with_ui_size(&self, ui_size: UiSize) -> Self {
        self.with_updated_parameters(|p| p.ui_size = ui_size)
    }

    pub fn with_action_mode(&self, action_mode: ActionMode) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.action_mode = action_mode;
        Self(AddressPointer::new(new_state))
    }

    pub fn with_strand_on_helix(&self, parameters: Option<(isize, usize)>) -> Self {
        let new_strand_parameters =
            parameters.map(|(start, length)| NewHelixStrand { length, start });
        if let ActionMode::BuildHelix { .. } = self.0.action_mode {
            let mut new_state = (*self.0).clone();
            let length = new_strand_parameters
                .as_ref()
                .map(|strand| strand.length)
                .unwrap_or_default();
            let start = new_strand_parameters
                .as_ref()
                .map(|strand| strand.start)
                .unwrap_or_default();
            new_state.strand_on_new_helix = new_strand_parameters;
            new_state.action_mode = ActionMode::BuildHelix {
                length,
                position: start,
            };
            Self(AddressPointer::new(new_state))
        } else {
            self.clone()
        }
    }

    pub fn exporting(&self, exporting: bool) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.exporting = exporting;
        Self(AddressPointer::new(new_state))
    }

    pub fn with_toggled_widget_basis(&self) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.widget_basis.toggle();
        Self(AddressPointer::new(new_state))
    }

    #[cfg(test)]
    pub fn update_design(&mut self, design: Design) {
        apply_update(self, |s| s.with_updated_design(design));
    }

    #[cfg(test)]
    pub fn with_updated_design(&self, design: Design) -> Self {
        let mut new_state = self.0.clone_inner();
        let new_interactor = new_state.design.with_updated_design(design);
        new_state.design = AddressPointer::new(new_interactor);
        Self(AddressPointer::new(new_state))
    }

    pub fn import_design(mut path: PathBuf) -> Result<Self, LoadDesignError> {
        let design_interactor = DesignInteractor::new_with_path(&path)?;
        if path.extension().map(|s| s.to_string_lossy()) != Some(ENS_BACKUP_EXTENSION.into()) {
            path.set_extension(ENS_EXTENSION);
        }
        Ok(Self(AddressPointer::new(AppState_ {
            design: AddressPointer::new(design_interactor),
            parameters: confy::load(APP_NAME, APP_NAME).unwrap_or_default(),
            path_to_current_design: Some(path.clone()),
            ..Default::default()
        }))
        .updated())
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

    pub(super) fn update(&mut self) {
        log::trace!("update");
        apply_update(self, Self::updated);
    }

    pub(super) fn apply_simulation_update(&mut self, update: Box<dyn SimulationUpdate>) {
        apply_update(self, |s| s.with_simulation_update_applied(update));
    }

    fn with_simulation_update_applied(self, update: Box<dyn SimulationUpdate>) -> Self {
        let mut design = self.0.design.clone_inner();
        design = design.with_simulation_update_applied(update);
        self.with_interactor(design)
    }

    fn updated(self) -> Self {
        let old_self = self.clone();
        let mut interactor = self.0.design.clone_inner();
        log::trace!("calling from updated!!");
        if self
            .0
            .design
            .design_need_update(&self.0.parameters.suggestion_parameters)
        {
            log::trace!("design need update");
            interactor =
                interactor.with_updated_design_reader(&self.0.parameters.suggestion_parameters);
            self.with_interactor(interactor)
        } else {
            old_self
        }
    }

    fn with_interactor(self, interactor: DesignInteractor) -> Self {
        let mut new_state = self.0.clone_inner();
        new_state.updated_once = true;
        new_state.design = AddressPointer::new(interactor);
        Self(AddressPointer::new(new_state))
    }

    pub(super) fn apply_design_op(
        &mut self,
        op: DesignOperation,
    ) -> Result<OkOperation, ErrOperation> {
        let result = self.0.design.apply_operation(op);
        self.handle_operation_result(result)
    }

    pub(super) fn apply_copy_operation(
        &mut self,
        op: CopyOperation,
    ) -> Result<OkOperation, ErrOperation> {
        let self_mut = self.0.make_mut();
        let design_mut = self_mut.design.make_mut();
        let result = design_mut.apply_copy_operation(op);
        self.handle_operation_result(result)
    }

    pub(super) fn update_pending_operation(
        &mut self,
        op: Arc<dyn Operation>,
    ) -> Result<OkOperation, ErrOperation> {
        let result = self.0.design.update_pending_operation(op);
        self.handle_operation_result(result)
    }

    pub(super) fn start_simulation(
        &mut self,
        operation: SimulationOperation,
    ) -> Result<OkOperation, ErrOperation> {
        let result = self.0.design.start_simulation(operation);
        self.handle_operation_result(result)
    }

    pub(super) fn update_simulation(
        &mut self,
        request: SimulationOperation,
    ) -> Result<OkOperation, ErrOperation> {
        let result = self.0.design.update_simulation(request);
        self.handle_operation_result(result)
    }

    fn handle_operation_result(
        &mut self,
        result: Result<InteractorResult, ErrOperation>,
    ) -> Result<OkOperation, ErrOperation> {
        log::trace!("handle operation result");
        match result {
            Ok(InteractorResult::Push {
                interactor: mut design,
                label,
            }) => {
                let new_selection = design.get_next_selection();
                let ret = Some(self.clone());
                let mut new_state = self.clone().with_interactor(design);
                if let Some(selection) = new_selection {
                    new_state = new_state.with_selection(selection, None);
                }
                *self = new_state;
                if let Some(state) = ret {
                    Ok(OkOperation::Undoable {
                        state,
                        label: label.into(),
                    })
                } else {
                    Ok(OkOperation::NotUndoable)
                }
            }
            Ok(InteractorResult::Replace(mut design)) => {
                let new_selection = design.get_next_selection();
                let mut new_state = self.clone().with_interactor(design);
                if let Some(selection) = new_selection {
                    new_state = new_state.with_selection(selection, None);
                }
                *self = new_state;
                Ok(OkOperation::NotUndoable)
            }
            Err(e) => {
                log::error!("error {e:?}");
                Err(e)
            }
        }
    }

    pub fn notified(&self, notification: InteractorNotification) -> Self {
        let new_interactor = self.0.design.notify(notification);
        self.clone().with_interactor(new_interactor)
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

    pub fn export(&self, export_path: &PathBuf, export_type: ExportType) -> ExportResult {
        self.get_design_interactor()
            .export(export_path, export_type)
    }

    pub fn get_selection(&self) -> impl AsRef<[Selection]> + use<> {
        self.0.selection.selection.clone()
    }

    fn is_changing_color(&self) -> bool {
        self.0.design.as_ref().is_changing_color()
    }

    pub(super) fn prepare_for_replacement(&mut self, source: &Self) {
        *self = self.with_candidates(vec![]);
        *self = self.with_action_mode(source.0.action_mode);
        *self = self.with_selection_mode(source.0.selection_mode);
        *self = self.with_suggestion_parameters(source.0.parameters.suggestion_parameters);
        *self = self.with_check_xovers_parameters(source.0.parameters.check_xover_parameters);
        *self = self.with_updated_parameters(|p| *p = source.0.parameters.clone());
    }

    pub fn with_check_xovers_parameters(
        &self,
        check_xover_parameters: CheckXoversParameter,
    ) -> Self {
        self.with_updated_parameters(|p| p.check_xover_parameters = check_xover_parameters)
    }

    pub fn with_follow_stereographic_camera(&self, follow: bool) -> Self {
        self.with_updated_parameters(|p| p.follow_stereography = follow)
    }

    pub fn with_show_stereographic_camera(&self, show: bool) -> Self {
        self.with_updated_parameters(|p| p.show_stereography = show)
    }

    pub fn with_show_h_bonds(&self, show: HBondDisplay) -> Self {
        self.with_updated_parameters(|p| p.show_h_bonds = show)
    }

    pub fn with_show_bezier_paths(&self, show: bool) -> Self {
        self.with_updated_parameters(|p| p.show_bezier_paths = show)
    }

    pub fn all_helices_on_axis(&self, on_axis: bool) -> Self {
        self.with_updated_parameters(|p| p.all_helices_on_axis = on_axis)
    }

    pub fn set_bezier_revolution_id(&self, id: Option<usize>) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.unrooted_surface.bezier_path_id = id.map(|id| BezierPathId(id as u32));
        Self(AddressPointer::new(new_state))
    }

    pub fn set_bezier_revolution_radius(&self, radius: f64) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.set_surface_revolution_radius(radius);
        Self(AddressPointer::new(new_state))
    }

    pub fn set_revolution_axis_position(&self, position: f64) -> Self {
        let mut new_state = (*self.0).clone();
        new_state.set_surface_axis_position(position);
        Self(AddressPointer::new(new_state))
    }

    pub fn set_unrooted_surface(
        &self,
        surface: Option<UnrootedRevolutionSurfaceDescriptor>,
    ) -> Self {
        if self.0.unrooted_surface.descriptor.as_ref() != surface.as_ref() {
            let mut new_state = (*self.0).clone();
            new_state.set_unrooted_surface(surface);
            Self(AddressPointer::new(new_state))
        } else {
            self.clone()
        }
    }

    pub fn with_toggled_all_helices_on_axis(&self) -> Self {
        self.with_updated_parameters(|p| p.all_helices_on_axis ^= true)
    }

    pub fn with_background3d(&self, bg: Background3D) -> Self {
        self.with_updated_parameters(|p| p.background3d = bg)
    }

    pub fn with_rendering_mode(&self, rendering_mode: RenderingMode) -> Self {
        self.with_updated_parameters(|p| p.rendering_mode = rendering_mode)
    }

    pub fn with_scroll_sensitivity(&self, sensitivity: f32) -> Self {
        self.with_updated_parameters(|p| p.scroll_sensitivity = sensitivity)
    }

    pub fn with_inverted_y_scroll(&self, inverted: bool) -> Self {
        self.with_updated_parameters(|p| p.inverted_y_scroll = inverted)
    }

    fn with_updated_parameters<F>(&self, update: F) -> Self
    where
        F: Fn(&mut AppStateParameters),
    {
        let mut new_state = (*self.0).clone();
        update(&mut new_state.parameters);
        if let Err(e) = confy::store(APP_NAME, APP_NAME, new_state.parameters.clone()) {
            log::error!("Could not save preferences {e:?}");
        }
        Self(AddressPointer::new(new_state))
    }

    pub(super) fn get_pasting_status(&self) -> PastingStatus {
        self.0.design.get_pasting_status()
    }

    pub(super) fn can_iterate_duplication(&self) -> bool {
        self.0.design.can_iterate_duplication()
    }

    pub(super) fn optimize_shift(
        &mut self,
        reader: &mut ChannelReader,
    ) -> Result<OkOperation, ErrOperation> {
        let result = self.0.design.optimize_shift(reader);
        self.handle_operation_result(result)
    }

    pub(super) fn is_in_stable_state(&self) -> bool {
        self.0.design.is_in_stable_state()
    }

    pub(super) fn set_visibility_sieve(
        &mut self,
        selection: Vec<Selection>,
        compl: bool,
    ) -> Result<OkOperation, ErrOperation> {
        let result = self
            .0
            .design
            .clone_inner()
            .with_visibility_sieve(selection, compl);
        self.handle_operation_result(Ok(result))
    }

    pub fn design_was_modified(&self, other: &Self) -> bool {
        self.0.design.has_different_design_than(&other.0.design)
            && (self.0.updated_once || other.0.updated_once)
    }

    fn get_strand_building_state(&self) -> Option<StrandBuildingStatus> {
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

    pub fn get_simulation_state(&self) -> crate::ensnano_interactor::SimulationState {
        self.0.design.get_simulation_state()
    }

    pub fn is_building_hyperboloid(&self) -> bool {
        self.0.design.is_building_hyperboloid()
    }

    pub fn with_expand_insertion_set(self, expand: bool) -> Self {
        let mut ret = (*self.0).clone();
        ret.show_insertion_discriminants = !expand;
        Self(AddressPointer::new(ret))
    }

    pub(super) fn get_new_selection(&self) -> Option<Vec<Selection>> {
        self.0.design.get_new_selection()
    }
}

#[derive(Clone, Default)]
pub struct AppState_ {
    /// The set of currently selected objects
    pub selection: AppStateSelection,
    /// The set of objects that are "one click away from being selected"
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
