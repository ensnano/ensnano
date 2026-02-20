pub mod clipboard;
pub mod shift_optimization;
pub mod simulations;
pub mod update_insertion_length;

use self::{
    clipboard::{Clipboard, CopyOperation, PastePosition, PastedStrand, StrandClipboard},
    simulations::{
        GridSystemInterface, GridsSystemThread, HelixSystemInterface, HelixSystemThread,
        SimulationOperation,
        rapier::{RapierInterface, RapierPhysicalSystem},
        revolutions::{RevolutionSystemInterface, RevolutionSystemThread},
        roller::{PhysicalSystem, RollInterface},
        twister::{TwistInterface, Twister},
    },
};
use crate::{
    app_state::{
        AddressPointer, AppState, channel_reader::ScaffoldShiftReader,
        design_interactor::controller::simulations::SimulationInterface,
    },
    design::{
        operation::{
            BezierPlaneHomothethy, DesignOperation, DesignRotation, DesignTranslation,
            HyperboloidOperation, IsometryTarget, NewBezierTangentVector,
        },
        selection::{Selection, list_of_helices},
    },
    operation::{AppStateOperationOutcome, AppStateOperationResult},
    utils::operation::{SimpleOperation, TranslateBezierPathVertex},
};
use ensnano_design::{
    CameraId, Design,
    bezier_plane::{
        BezierPathId, BezierPlaneDescriptor, BezierPlaneId, BezierVertex, BezierVertexId,
        import_from_svg::{SvgImportError, read_first_svg_path},
    },
    curves::{
        CurveDescriptor,
        bezier::{BezierControlPoint, BezierEnd},
    },
    design_element::{DesignElementKey, DnaAttribute},
    design_operations::{
        DesignOperationError, attach_object_to_grid, make_grid_from_helices, rotate_helices_3d,
        translate_helices,
    },
    domains::{Domain, helix_interval::HelixInterval},
    drawing_style::{DrawingAttribute, DrawingStyle},
    external_3d_objects::{External3DObject, External3DObjectDescriptor},
    grid::{
        Edge, GridDescriptor, GridDivision as _, GridId, GridObject, GridPosition, GridTypeDescr,
        HelixGridPosition, copy_grid::GridCopyError, grid_collection::FreeGridId,
        hyperboloid::Hyperboloid,
    },
    group_attributes::GroupPivot,
    helices::{Helices, Helix, NuclCollection},
    mutate_in_arc, mutate_one_helix,
    nucl::Nucl,
    organizer_tree::GroupId,
    strands::{DomainJunction, Strand, Strands},
};
use ensnano_utils::{
    PastingStatus, SimulationState,
    clipboard::ClipboardContent,
    colors::{new_color, random_color_with_shade},
    strand_builder::{DomainIdentifier, NeighborDescriptor, StrandBuilder, get_neighbor_nucl},
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    f32::consts::PI,
    path::PathBuf,
    str::FromStr as _,
    sync::{Arc, Mutex},
};
use ultraviolet::{Isometry2, Rotor2, Rotor3, Vec2, Vec3};

const ALLOW_SAME_HELIX_CROSSOVERS: bool = true;

type DuplicationEdge = (Edge, isize);

#[derive(Clone, Default)]
pub struct Controller {
    color_idx: usize,
    state: ControllerState,
    clipboard: AddressPointer<Clipboard>,
    // pub next_selection: Option<Vec<Selection>>,
}

impl Controller {
    fn new_color(color_idx: &mut usize) -> u32 {
        let color = {
            let hue = (*color_idx as f64 * (1. + 5f64.sqrt()) / 2.).fract() * 360.;
            let saturation = (*color_idx as f64 * 7. * (1. + 5f64.sqrt() / 2.)).fract() * 0.4 + 0.4;
            let value = (*color_idx as f64 * 11. * (1. + 5f64.sqrt() / 2.)).fract() * 0.7 + 0.1;
            let hsv = color_space::Hsv::new(hue, saturation, value);
            let rgb = color_space::Rgb::from(hsv);
            (0xFF << 24) | ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
        };
        *color_idx += 1;
        color
    }

    /// Apply an operation to the design. This will either produce a modified copy of the design,
    /// or result in an error that could be shown to the user to explain why the requested
    /// operation could not be applied.
    pub fn apply_operation(
        // &mut self,
        // design: &mut Design,
        state: &mut AppState,
        operation: DesignOperation,
    ) -> AppStateOperationResult {
        log::debug!("operation {operation:?}");
        match state.controller().check_compatibility(&operation) {
            OperationCompatibility::Incompatible => {
                return Err(OperationError::IncompatibleState(
                    state.controller().state.state_name().into(),
                ));
            }
            OperationCompatibility::FinishFirst => return Err(OperationError::FinishFirst),
            OperationCompatibility::Compatible => (),
        }
        log::debug!("applicable");

        operation.apply(state)
    }

    pub fn update_pending_operation(
        state: &mut AppState,
        operation: Arc<dyn SimpleOperation>,
    ) -> AppStateOperationResult {
        let effect = operation.effect();

        let result = Self::apply_operation(state, effect)?;
        state.controller_mut().state.update_operation(operation);

        Ok(result)
    }

    pub fn apply_copy_operation(
        &mut self,
        design: &mut Design,
        operation: CopyOperation,
    ) -> AppStateOperationResult {
        operation.apply(self, design)
    }

    fn check_state_compatible_with_simulation(&self) -> Result<(), OperationError> {
        if self.is_in_persistent_state().is_transitory() {
            return Err(OperationError::IncompatibleState(
                "Cannot launch simulation while editing".into(),
            ));
        }

        Ok(())
    }

    #[expect(clippy::complexity)]
    pub fn apply_simulation_operation(
        &mut self,
        design: &mut Design,
        operation: SimulationOperation,
    ) -> Result<
        (
            AppStateOperationOutcome,
            Option<Arc<Mutex<dyn SimulationInterface>>>,
        ),
        OperationError,
    > {
        let mut returned_interface = None;
        // let mut ret = self.clone();
        match operation {
            SimulationOperation::RevolutionRelaxation { system } => {
                self.check_state_compatible_with_simulation()?;
                let interface = RevolutionSystemThread::start_new(system)?;
                let dyn_interface: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
                returned_interface = Some(dyn_interface);
                self.state = ControllerState::Relaxing {
                    interface,
                    _initial_design: AddressPointer::new(design.clone()),
                };
                //reader.attach_state(&ret_dyn);
            }
            SimulationOperation::StartHelices {
                presenter,
                parameters,
            } => {
                self.check_state_compatible_with_simulation()?;
                let interface = HelixSystemThread::start_new(presenter, parameters)?;
                let dyn_interface: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
                returned_interface = Some(dyn_interface);
                self.state = ControllerState::Simulating {
                    interface,
                    initial_design: AddressPointer::new(design.clone()),
                };
            }
            SimulationOperation::StartGrids {
                presenter,
                parameters,
            } => {
                self.check_state_compatible_with_simulation()?;
                let interface = GridsSystemThread::start_new(presenter, parameters)?;
                let dyn_interface: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
                returned_interface = Some(dyn_interface);
                self.state = ControllerState::SimulatingGrids {
                    interface,
                    _initial_design: AddressPointer::new(design.clone()),
                };
            }
            SimulationOperation::UpdateRapierParameters {
                presenter,
                parameters,
            } => {
                // if no simulation is requested, no operation is necessary
                if parameters.is_simulation_running {
                    // we don't do this step to be able to update our parameters
                    // on the fly
                    //self.check_state_compatible_with_simulation()?;
                    if let ControllerState::RapierSimulating { interface, .. } = &self.state {
                        // the simulation is ongoing, we preserve the initial design
                        if let Ok(mut interface) = interface.lock() {
                            interface.parameters = parameters;
                        }
                    } else {
                        // the simulation is starting, we save the initial design
                        let interface = RapierPhysicalSystem::start_new(presenter, parameters);
                        let dyn_interface: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
                        returned_interface = Some(dyn_interface);

                        self.state = ControllerState::RapierSimulating {
                            interface,
                            initial_design: AddressPointer::new(design.clone()),
                        };
                    }
                }
            }
            SimulationOperation::StartRoll {
                presenter,
                target_helices,
            } => {
                self.check_state_compatible_with_simulation()?;
                let interface = PhysicalSystem::start_new(presenter, target_helices);
                let dyn_interface: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
                returned_interface = Some(dyn_interface);
                self.state = ControllerState::Rolling {
                    _interface: interface,
                    _initial_design: AddressPointer::new(design.clone()),
                };
            }
            SimulationOperation::StartTwist { grid_id, presenter } => {
                self.check_state_compatible_with_simulation()?;
                let interface = Twister::start_new(presenter, grid_id)
                    .ok_or(OperationError::GridDoesNotExist(grid_id))?;
                let dyn_interface: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
                returned_interface = Some(dyn_interface);
                self.state = ControllerState::Twisting {
                    _interface: interface,
                    _initial_design: AddressPointer::new(design.clone()),
                    grid_id,
                };
            }
            SimulationOperation::UpdateParameters { new_parameters } => {
                if let ControllerState::Simulating { interface, .. } = &self.state {
                    interface.lock().unwrap().parameters_update = Some(new_parameters);
                } else if let ControllerState::SimulatingGrids { interface, .. } = &self.state {
                    interface.lock().unwrap().parameters_update = Some(new_parameters);
                } else {
                    return Err(OperationError::IncompatibleState(
                        "No simulation running".into(),
                    ));
                }
            }
            SimulationOperation::Stop => match &self.state {
                ControllerState::Simulating { initial_design, .. } => {
                    self.state = ControllerState::WithPausedSimulation {
                        initial_design: initial_design.clone(),
                    };
                }
                ControllerState::SimulatingGrids { .. }
                | ControllerState::Rolling { .. }
                | ControllerState::Twisting { .. } => {
                    self.state = ControllerState::Normal;
                }
                ControllerState::Relaxing { interface, .. } => {
                    interface.lock().unwrap().kill();
                    design.additional_structure = None;
                    self.state = ControllerState::Normal;
                }
                ControllerState::RapierSimulating {
                    interface,
                    initial_design,
                } => {
                    interface.lock().unwrap().kill();
                    self.state = ControllerState::WithPausedSimulation {
                        initial_design: initial_design.clone(),
                    }
                }
                _ => (),
            },
            SimulationOperation::Reset => {
                if let ControllerState::WithPausedSimulation { initial_design } = &self.state {
                    *design = initial_design.clone_inner();
                    self.state = ControllerState::Normal;
                }
            }
            SimulationOperation::FinishRelaxation => {
                if let ControllerState::Relaxing { interface, .. } = &self.state {
                    interface.lock().unwrap().finish();
                }
            }
        }

        Ok((
            AppStateOperationOutcome::Push {
                label: "Simulation".into(),
            },
            returned_interface,
        ))
    }

    pub fn change_strand_name(
        state: &mut AppState,
        s_id: usize,
        name: String,
    ) -> Result<(), OperationError> {
        let (design, controller) = state.design_controller_mut();
        let strand = design
            .strands
            .get_mut(&s_id)
            .ok_or(OperationError::StrandDoesNotExist(s_id))?;
        controller.state = ControllerState::ChangingStrandName { strand_id: s_id };
        strand.set_name(name);
        Ok(())
    }

    fn add_hyperboloid_helices(
        state: &mut AppState,
        hyperboloid: &Hyperboloid,
        position: Vec3,
        orientation: Rotor3,
    ) -> Result<(), OperationError> {
        // the hyperboloid grid is always the last one that was added to the design
        let grid_id = state
            .design()
            .free_grids
            .keys()
            .max()
            .copied()
            .ok_or(OperationError::GridDoesNotExist(GridId::FreeGrid(0)))?;
        let helix_parameters = state.design().helix_parameters.unwrap_or_default();
        let (helices, nb_nucl) = hyperboloid.make_helices(&helix_parameters);
        let nb_nucl = nb_nucl.min(5000);
        let mut helices_mut = state.design_mut().helices.make_mut();
        let mut keys = Vec::with_capacity(helices.len());
        for (i, mut h) in helices.into_iter().enumerate() {
            let origin = hyperboloid.origin_helix(&helix_parameters, i as isize, 0);
            let z_vec = Vec3::unit_z().rotated_by(orientation);
            let y_vec = Vec3::unit_y().rotated_by(orientation);
            h.position = position + origin.x * z_vec + origin.y * y_vec;
            h.orientation =
                orientation * hyperboloid.orientation_helix(&helix_parameters, i as isize, 0);
            if let Some(curve) = h.curve.as_mut() {
                mutate_in_arc(curve, |c| {
                    if let CurveDescriptor::Twist(twist) = c {
                        twist.orientation = orientation;
                        twist.position = position;
                    }
                });
            }
            h.grid_position = Some(HelixGridPosition {
                grid: grid_id.to_grid_id(),
                x: i as isize,
                y: 0,
                axis_pos: 0,
                roll: 0.,
            });
            let key = helices_mut.push_helix(h);
            keys.push(key);
        }
        drop(helices_mut);
        for key in keys {
            for b in &[true, false] {
                let new_key = Self::add_strand(state, key, 0, *b);
                if let Domain::HelixDomain(ref mut dom) = state
                    .design_mut()
                    .strands
                    .get_mut(&new_key)
                    .unwrap()
                    .domains[0]
                {
                    dom.end = dom.start + nb_nucl as isize;
                }
            }
        }
        Ok(())
    }

    pub fn set_roll_helices(
        state: &mut AppState,
        helices: Vec<usize>,
        roll: f32,
    ) -> Result<(), OperationError> {
        let mut helices_mut = state.design_mut().helices.make_mut();
        for h in &helices {
            if let Some(helix) = helices_mut.get_mut(h) {
                helix.roll = roll;
            } else {
                return Err(OperationError::HelixDoesNotExists(*h));
            }
        }
        drop(helices_mut);

        state.controller_mut().state = ControllerState::SettingRollHelices;
        Ok(())
    }

    pub fn set_visibility_helix(
        state: &mut AppState,
        helix: usize,
        visible: bool,
    ) -> Result<(), OperationError> {
        mutate_one_helix(state.design_mut(), helix, |h| h.visible = visible)
            .ok_or(OperationError::HelixDoesNotExists(helix))?;
        Ok(())
    }

    pub fn flip_helix_group(state: &mut AppState, helix: usize) -> Result<(), OperationError> {
        let mut new_groups = BTreeMap::clone(state.design().groups.as_ref());
        log::info!("setting group {:?}", new_groups.get(&helix));
        match new_groups.remove(&helix) {
            None => {
                new_groups.insert(helix, false);
            }
            Some(false) => {
                new_groups.insert(helix, true);
            }
            Some(true) => (),
        }
        state.design_mut().groups = Arc::new(new_groups);
        Ok(())
    }

    pub fn set_group_pivot(
        design: &mut Design,
        group_id: GroupId,
        pivot: GroupPivot,
    ) -> Result<(), OperationError> {
        let attributes = design.group_attributes.entry(group_id).or_default();
        if attributes.pivot.is_none() {
            attributes.pivot = Some(pivot);
        }
        Ok(())
    }

    pub fn update_attribute(
        state: &mut AppState,
        attribute: DnaAttribute,
        elements: Vec<DesignElementKey>,
    ) -> Result<(), OperationError> {
        log::info!("updating attribute {attribute:?}, {elements:?}");
        for elt in &elements {
            match attribute {
                DnaAttribute::Visible(b) => Self::make_element_visible(state, elt, b)?,
                DnaAttribute::XoverGroup(g) => Self::set_xover_group_of_elt(state, elt, g)?,
                DnaAttribute::LockedForSimulations(locked) => {
                    Self::set_lock_during_simulation(state, elt, locked)?;
                }
            }
        }
        Ok(())
    }

    pub fn flip_anchors(state: &mut AppState, nucls: Vec<Nucl>) -> Result<(), OperationError> {
        let new_anchor_status = !nucls.iter().all(|n| state.design().anchors.contains(n));
        if new_anchor_status {
            for n in nucls {
                state.design_mut().anchors.insert(n);
            }
        } else {
            for n in &nucls {
                state.design_mut().anchors.remove(n);
            }
        }
        Ok(())
    }

    fn make_element_visible(
        state: &mut AppState,
        element: &DesignElementKey,
        visible: bool,
    ) -> Result<(), OperationError> {
        match element {
            DesignElementKey::Helix(helix) => {
                mutate_one_helix(state.design_mut(), *helix, |h| h.visible = visible)
                    .ok_or(OperationError::HelixDoesNotExists(*helix))?;
            }
            DesignElementKey::Grid(g_id) => {
                let mut grids_mut = state.design_mut().free_grids.make_mut();
                let g_id = FreeGridId(*g_id);
                let grid = grids_mut
                    .get_mut(&g_id)
                    .ok_or_else(|| OperationError::GridDoesNotExist(g_id.to_grid_id()))?;
                grid.invisible = !grid.invisible;
                drop(grids_mut);
            }
            _ => (),
        }
        Ok(())
    }

    fn set_xover_group_of_elt(
        state: &mut AppState,
        element: &DesignElementKey,
        group: Option<bool>,
    ) -> Result<(), OperationError> {
        if let DesignElementKey::Helix(h_id) = element {
            if !state.design().helices.contains_key(h_id) {
                return Err(OperationError::HelixDoesNotExists(*h_id));
            }
            let mut new_groups = BTreeMap::clone(state.design().groups.as_ref());
            if let Some(group) = group {
                new_groups.insert(*h_id, group);
            } else {
                new_groups.remove(h_id);
            }
            state.design_mut().groups = Arc::new(new_groups);
        }
        Ok(())
    }

    fn set_lock_during_simulation(
        state: &mut AppState,
        element: &DesignElementKey,
        locked: bool,
    ) -> Result<(), OperationError> {
        if let DesignElementKey::Helix(h_id) = element {
            if !state.design().helices.contains_key(h_id) {
                return Err(OperationError::HelixDoesNotExists(*h_id));
            }
            mutate_one_helix(state.design_mut(), *h_id, |h| {
                h.locked_for_simulations = locked;
            });
        }
        Ok(())
    }

    pub fn apply_hyperboloid_operation(
        state: &mut AppState,
        operation: HyperboloidOperation,
    ) -> Result<(), OperationError> {
        match operation {
            HyperboloidOperation::New {
                position,
                orientation,
                request,
            } => {
                let initial_design = state.interactor().design.clone();

                state.controller_mut().state = ControllerState::MakingHyperboloid {
                    position,
                    orientation,
                    initial_design,
                };

                let hyperboloid = request.to_grid();
                let grid_descriptor =
                    GridDescriptor::hyperboloid(position, orientation, hyperboloid.clone());
                Self::add_grid(state, grid_descriptor);
                Self::add_hyperboloid_helices(state, &hyperboloid, position, orientation)?;
                Ok(())
            }
            HyperboloidOperation::Update(request) => {
                if let ControllerState::MakingHyperboloid {
                    position,
                    orientation,
                    initial_design,
                } = state.controller().state.clone()
                {
                    state.interactor_mut().design = initial_design;
                    let hyperboloid = request.to_grid();
                    let grid_descriptor =
                        GridDescriptor::hyperboloid(position, orientation, hyperboloid.clone());
                    Self::add_grid(state, grid_descriptor);
                    Self::add_hyperboloid_helices(state, &hyperboloid, position, orientation)?;
                    Ok(())
                } else {
                    Err(OperationError::IncompatibleState(
                        "Not making hyperboloid".into(),
                    ))
                }
            }
            HyperboloidOperation::Cancel => {
                if let ControllerState::MakingHyperboloid { initial_design, .. } =
                    state.controller().state.clone()
                {
                    state.interactor_mut().design = initial_design;
                    state.controller_mut().state = ControllerState::Normal;
                    Ok(())
                } else {
                    Err(OperationError::IncompatibleState(
                        "Not making hyperboloid".into(),
                    ))
                }
            }
            HyperboloidOperation::Finalize => {
                if let ControllerState::MakingHyperboloid { .. } = state.controller().state {
                    state
                        .0
                        .make_mut()
                        .design
                        .make_mut()
                        .controller
                        .make_mut()
                        .state = ControllerState::Normal;
                    Ok(())
                } else {
                    Err(OperationError::IncompatibleState(
                        "Not making hyperboloid".into(),
                    ))
                }
            }
        }
    }

    pub fn is_building_hyperboloid(&self) -> bool {
        matches!(&self.state, ControllerState::MakingHyperboloid { .. })
    }

    pub fn can_iterate_duplication(&self) -> bool {
        matches!(
            self.state,
            ControllerState::WithPendingStrandDuplication { .. }
                | ControllerState::WithPendingXoverDuplication { .. }
                | ControllerState::WithPendingHelicesDuplication { .. }
        )
    }

    pub fn optimize_shift(
        &mut self,
        chanel_reader: &mut ScaffoldShiftReader,
        nucl_collection: Arc<NuclCollection>,
        design: &Design,
    ) -> AppStateOperationResult {
        match self.check_compatibility(&DesignOperation::SetScaffoldShift(0)) {
            OperationCompatibility::Incompatible => Err(OperationError::IncompatibleState(
                self.state.state_name().to_owned(),
            )),
            OperationCompatibility::Compatible | OperationCompatibility::FinishFirst => {
                self.start_shift_optimization(design, chanel_reader, nucl_collection);
                Ok(AppStateOperationOutcome::NoOp)
            }
        }
    }

    fn start_shift_optimization(
        &mut self,
        design: &Design,
        chanel_reader: &mut ScaffoldShiftReader,
        nucl_collection: Arc<NuclCollection>,
    ) {
        self.state = ControllerState::OptimizingScaffoldPosition;
        shift_optimization::optimize_shift(
            Arc::new(design.clone()),
            nucl_collection,
            chanel_reader,
        );
    }

    pub fn get_clipboard_content(&self) -> ClipboardContent {
        let n = self.clipboard.size();
        match self.clipboard.as_ref() {
            Clipboard::Empty => ClipboardContent::Empty,
            Clipboard::Grids(_) => ClipboardContent::Grids(n),
            Clipboard::Strands(_) => ClipboardContent::Strands(n),
            Clipboard::Helices(_) => ClipboardContent::Helices(n),
            Clipboard::Xovers(_) => ClipboardContent::Xovers(n),
        }
    }

    pub fn get_pasting_status(&self) -> PastingStatus {
        match self.state {
            ControllerState::PositioningStrandDuplicationPoint { .. }
            | ControllerState::DoingFirstXoversDuplication { .. }
            | ControllerState::PositioningHelicesDuplicationPoint { .. } => {
                PastingStatus::Duplication
            }
            ControllerState::PositioningStrandPastingPoint { .. }
            | ControllerState::PastingXovers { .. }
            | ControllerState::PositioningHelicesPastingPoint { .. } => PastingStatus::Copy,
            _ => PastingStatus::None,
        }
    }

    pub fn notify(&mut self, notification: InteractorNotification) {
        match notification {
            InteractorNotification::FinishOperation => self.state.finish(),
            InteractorNotification::NewSelection => {
                self.state.acknowledge_new_selection();
            }
        }
    }

    fn check_compatibility(&self, operation: &DesignOperation) -> OperationCompatibility {
        match self.state {
            ControllerState::MakingHyperboloid { .. } => {
                if let DesignOperation::HyperboloidOperation(op) = operation {
                    if let HyperboloidOperation::New { .. } = op {
                        OperationCompatibility::Incompatible
                    } else {
                        OperationCompatibility::Compatible
                    }
                } else {
                    OperationCompatibility::Incompatible
                }
            }
            ControllerState::ChangingColor => {
                if let DesignOperation::ChangeColor { .. } = operation {
                    OperationCompatibility::Compatible
                } else {
                    OperationCompatibility::Incompatible
                }
            }
            ControllerState::SettingRollHelices => {
                if let DesignOperation::SetRollHelices { .. } = operation {
                    OperationCompatibility::Compatible
                } else {
                    OperationCompatibility::FinishFirst
                }
            }
            ControllerState::BuildingStrand { initializing, .. } => {
                if let DesignOperation::MoveBuilders(_) = operation {
                    OperationCompatibility::Compatible
                } else if initializing {
                    OperationCompatibility::FinishFirst
                } else {
                    OperationCompatibility::Incompatible
                }
            }
            ControllerState::OptimizingScaffoldPosition => {
                if let DesignOperation::SetScaffoldShift(_) = operation {
                    OperationCompatibility::Compatible
                } else {
                    OperationCompatibility::Incompatible
                }
            }
            ControllerState::ChangingStrandName {
                strand_id: current_s_id,
            } => {
                if let DesignOperation::SetStrandName { s_id, .. } = operation {
                    if current_s_id == *s_id {
                        OperationCompatibility::Compatible
                    } else {
                        OperationCompatibility::FinishFirst
                    }
                } else {
                    OperationCompatibility::FinishFirst
                }
            }
            ControllerState::WithPausedSimulation { .. } => OperationCompatibility::FinishFirst,
            ControllerState::WithPendingStrandDuplication { .. }
            | ControllerState::WithPendingHelicesDuplication { .. }
            | ControllerState::WithPendingXoverDuplication { .. }
            | ControllerState::Normal
            | ControllerState::WithPendingOp { .. }
            | ControllerState::ApplyingOperation { .. } => OperationCompatibility::Compatible,
            ControllerState::PositioningStrandPastingPoint { .. }
            | ControllerState::PositioningStrandDuplicationPoint { .. }
            | ControllerState::PositioningHelicesPastingPoint { .. }
            | ControllerState::PositioningHelicesDuplicationPoint { .. }
            | ControllerState::DoingFirstXoversDuplication { .. }
            | ControllerState::PastingXovers { .. }
            | ControllerState::Simulating { .. }
            | ControllerState::RapierSimulating { .. }
            | ControllerState::SimulatingGrids { .. }
            | ControllerState::Relaxing { .. }
            | ControllerState::Rolling { .. }
            | ControllerState::Twisting { .. } => OperationCompatibility::Incompatible,
        }
    }

    fn update_state_not_design(&mut self) {
        if !matches!(self.state, ControllerState::ApplyingOperation { .. }) {
            self.state = ControllerState::ApplyingOperation { operation: None };
        }
    }

    pub fn get_simulation_state(&self) -> SimulationState {
        match self.state {
            ControllerState::Simulating { .. } => SimulationState::RigidHelices,
            ControllerState::WithPausedSimulation { .. } => SimulationState::Paused,
            ControllerState::SimulatingGrids { .. } => SimulationState::RigidGrid,
            ControllerState::Rolling { .. } => SimulationState::Rolling,
            ControllerState::Twisting { grid_id, .. } => SimulationState::Twisting { grid_id },
            ControllerState::Relaxing { .. } => SimulationState::Relaxing,
            _ => SimulationState::None,
        }
    }

    pub fn is_in_persistent_state(&self) -> StatePersistence {
        match self.state {
            ControllerState::Normal
            | ControllerState::WithPendingOp { .. }
            | ControllerState::WithPendingStrandDuplication { .. }
            | ControllerState::WithPendingXoverDuplication { .. }
            | ControllerState::WithPendingHelicesDuplication { .. } => StatePersistence::Persistent,
            ControllerState::WithPausedSimulation { .. }
            | ControllerState::SettingRollHelices
            | ControllerState::ChangingStrandName { .. } => StatePersistence::NeedFinish,
            ControllerState::MakingHyperboloid { .. }
            | ControllerState::BuildingStrand { .. }
            | ControllerState::ChangingColor
            | ControllerState::ApplyingOperation { .. }
            | ControllerState::PositioningStrandPastingPoint { .. }
            | ControllerState::PositioningHelicesPastingPoint { .. }
            | ControllerState::PastingXovers { .. }
            | ControllerState::DoingFirstXoversDuplication { .. }
            | ControllerState::OptimizingScaffoldPosition
            | ControllerState::Simulating { .. }
            | ControllerState::RapierSimulating { .. }
            | ControllerState::SimulatingGrids { .. }
            | ControllerState::Relaxing { .. }
            | ControllerState::Rolling { .. }
            | ControllerState::Twisting { .. }
            | ControllerState::PositioningStrandDuplicationPoint { .. }
            | ControllerState::PositioningHelicesDuplicationPoint { .. } => {
                StatePersistence::Transitory
            }
        }
    }

    pub fn turn_selection_into_grid(
        state: &mut AppState,
        selection: Vec<Selection>,
    ) -> Result<(), OperationError> {
        let helices = list_of_helices(&selection).ok_or(OperationError::BadSelection)?;
        make_grid_from_helices(state.design_mut(), &helices.1)?;
        Ok(())
    }

    pub fn add_grid(state: &mut AppState, descriptor: GridDescriptor) {
        let mut new_grids = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .free_grids
            .make_mut();
        new_grids.push(descriptor);
    }

    pub fn add_bezier_plane(design: &mut Design, descriptor: BezierPlaneDescriptor) {
        design.bezier_planes.make_mut().push(descriptor);
    }

    // pub fn create_bezier_path(&mut self, design: &mut Design, first_vertex: BezierVertex) {
    pub fn create_bezier_path(state: &mut AppState, first_vertex: BezierVertex) {
        let path_id = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .bezier_paths
            .make_mut()
            .create_path(first_vertex);

        state
            .0
            .make_mut()
            .design
            .make_mut()
            .controller
            .make_mut()
            .state = ControllerState::ApplyingOperation {
            operation: Some(Arc::new(TranslateBezierPathVertex {
                vertices: vec![BezierVertexId {
                    path_id,
                    vertex_id: 0,
                }],
                x: first_vertex.position.x,
                y: first_vertex.position.y,
            })),
        };

        let selection = vec![Selection::BezierVertex(BezierVertexId {
            path_id,
            vertex_id: 0,
        })];

        _ = state.set_selection(&selection, &None);
    }

    pub fn append_vertex_to_bezier_path(
        state: &mut AppState,
        path_id: BezierPathId,
        vertex: BezierVertex,
    ) -> Result<(), OperationError> {
        let vertex_id = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .bezier_paths
            .make_mut()
            .get_mut(&path_id)
            .ok_or(OperationError::PathDoesNotExist(path_id))
            .map(|p| p.add_vertex(vertex))?;

        state
            .0
            .make_mut()
            .design
            .make_mut()
            .controller
            .make_mut()
            .state = ControllerState::ApplyingOperation {
            operation: Some(Arc::new(TranslateBezierPathVertex {
                vertices: vec![BezierVertexId { path_id, vertex_id }],
                x: vertex.position.x,
                y: vertex.position.y,
            })),
        };

        let selection = vec![Selection::BezierVertex(BezierVertexId {
            path_id,
            vertex_id,
        })];

        state.set_selection(&selection, &None)?;

        Ok(())
    }

    pub fn rm_bezier_vertices(
        design: &mut Design,
        mut vertices: Vec<BezierVertexId>,
    ) -> Result<(), OperationError> {
        vertices.sort();
        let mut paths_id: Vec<_> = vertices.iter().map(|v| v.path_id).collect();
        paths_id.dedup();

        let mut new_paths = design.bezier_paths.make_mut();
        let mut iterator = vertices.into_iter().rev();

        let mut vertex_id = iterator.next();
        for p_id in paths_id.into_iter().rev() {
            let path = new_paths
                .get_mut(&p_id)
                .ok_or(OperationError::PathDoesNotExist(p_id))?;
            while let Some(BezierVertexId {
                vertex_id: v_id, ..
            }) = vertex_id.filter(|v_id| v_id.path_id == p_id)
            {
                path.remove_vertex(v_id)
                    .ok_or(OperationError::VertexDoesNotExist(p_id, v_id))?;
                vertex_id = iterator.next();
            }
            if path.vertices().is_empty() {
                new_paths
                    .remove_path(&p_id)
                    .ok_or(OperationError::PathDoesNotExist(p_id))?;
            }
        }

        drop(new_paths);
        Ok(())
    }

    /// Move a bezier vertex to a given position and transition to a transitory state.
    pub fn move_bezier_vertices(
        state: &mut AppState,
        mut vertices: Vec<BezierVertexId>,
        position: Vec2,
    ) -> Result<(), OperationError> {
        if let Some(BezierVertexId { path_id, vertex_id }) = vertices.first().copied() {
            vertices.sort();
            vertices.dedup();

            let path = state
                .0
                .design
                .design
                .bezier_paths
                .get(&path_id)
                .ok_or(OperationError::PathDoesNotExist(path_id))?;
            let vertex = path
                .vertices()
                .get(vertex_id)
                .ok_or(OperationError::VertexDoesNotExist(path_id, vertex_id))?;

            let translation = position - vertex.position;

            let mut new_paths = state
                .0
                .make_mut()
                .design
                .make_mut()
                .design
                .make_mut()
                .bezier_paths
                .make_mut();
            let mut next_selection = Vec::new();
            for BezierVertexId { path_id, vertex_id } in vertices {
                let path = new_paths
                    .get_mut(&path_id)
                    .ok_or(OperationError::PathDoesNotExist(path_id))?;
                let vertex = path
                    .get_vertex_mut(vertex_id)
                    .ok_or(OperationError::VertexDoesNotExist(path_id, vertex_id))?;
                let old_tangent_in = vertex.position_in.map(|p| p - vertex.position);
                let old_tangent_out = vertex.position_out.map(|p| p - vertex.position);
                vertex.position += translation;
                vertex.position_out = old_tangent_out.map(|t| vertex.position + t);
                vertex.position_in = old_tangent_in.map(|t| vertex.position + t);
                next_selection.push(Selection::BezierVertex(BezierVertexId {
                    path_id,
                    vertex_id,
                }));
            }
            drop(new_paths);

            state.set_selection(&next_selection, &None)?;

            Ok(())
        } else {
            Err(OperationError::NotImplemented)
        }
    }

    /// Set the position of a bezier vertex as a single revertible operation.
    pub fn set_bezier_vertex_position(
        state: &mut AppState,
        vertex_id: BezierVertexId,
        position: Vec2,
    ) -> Result<(), OperationError> {
        let BezierVertexId { vertex_id, path_id } = vertex_id;

        let mut binding = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .bezier_paths
            .make_mut();
        let vertex = binding
            .get_mut(&path_id)
            .ok_or(OperationError::PathDoesNotExist(path_id))
            .and_then(|v| {
                v.get_vertex_mut(vertex_id)
                    .ok_or(OperationError::VertexDoesNotExist(path_id, vertex_id))
            })?;

        let old_tangent_in = vertex.position_in.map(|p| p - vertex.position);
        let old_tangent_out = vertex.position_out.map(|p| p - vertex.position);
        vertex.position = position;
        vertex.position_out = old_tangent_out.map(|t| vertex.position + t);
        vertex.position_in = old_tangent_in.map(|t| vertex.position + t);
        Ok(())
    }

    pub fn make_bezier_path_cyclic(
        design: &mut Design,
        path_id: BezierPathId,
        cyclic: bool,
    ) -> Result<(), OperationError> {
        let mut new_paths = design.bezier_paths.make_mut();
        let path = new_paths
            .get_mut(&path_id)
            .ok_or(OperationError::PathDoesNotExist(path_id))?;
        path.is_cyclic = cyclic;
        drop(new_paths);
        Ok(())
    }

    pub fn set_bezier_tangent(
        state: &mut AppState,
        request: NewBezierTangentVector,
    ) -> Result<(), OperationError> {
        let (design, controller) = state.design_controller_mut();
        controller.update_state_not_design();

        let mut new_paths = design.bezier_paths.make_mut();
        let path_id = request.vertex_id.path_id;
        let vertex_id = request.vertex_id.vertex_id;
        let path = new_paths
            .get_mut(&path_id)
            .ok_or(OperationError::PathDoesNotExist(path_id))?;
        let vertex = path
            .get_vertex_mut(vertex_id)
            .ok_or(OperationError::VertexDoesNotExist(path_id, vertex_id))?;
        if request.tangent_in {
            vertex.position_in = Some(vertex.position + request.new_vector);
            if request.full_symmetry_other_tangent {
                vertex.position_out = Some(vertex.position - request.new_vector);
            } else {
                let norm = match vertex.position_out {
                    Some(p) => (vertex.position - p).mag(),
                    None => request.new_vector.mag(),
                };
                let out_vec = request.new_vector.normalized() * -norm;
                log::info!("norm {norm:?}");
                log::info!("new vec {:?}", request.new_vector);
                log::info!("out vec {out_vec:?}");
                vertex.position_out = Some(vertex.position + out_vec);
            }
        } else {
            vertex.position_out = Some(vertex.position + request.new_vector);
            if request.full_symmetry_other_tangent {
                vertex.position_in = Some(vertex.position - request.new_vector);
            } else {
                let norm = match vertex.position_in {
                    Some(p) => (vertex.position - p).mag(),
                    None => request.new_vector.mag(),
                };
                let in_vec = request.new_vector.normalized() * -norm;
                vertex.position_in = Some(vertex.position + in_vec);
            }
        }
        Ok(())
    }

    pub fn turn_bezier_path_into_grids(
        state: &mut AppState,
        path_id: BezierPathId,
        desc: GridTypeDescr,
    ) -> Result<(), OperationError> {
        let mut binding = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .bezier_paths
            .make_mut();
        let path = binding
            .get_mut(&path_id)
            .ok_or(OperationError::PathDoesNotExist(path_id))?;
        path.grid_type = Some(desc);
        Ok(())
    }

    pub fn apply_homothethy_on_bezier_plane(
        design: &mut Design,
        homothethy: BezierPlaneHomothethy,
    ) {
        log::info!("Applying homothethy {homothethy:?}");
        let mut paths_mut = design.bezier_paths.make_mut();
        let angle_origin = {
            let ab = homothethy.origin_moving_corner - homothethy.fixed_corner;
            ab.y.atan2(ab.x)
        };
        let angle_now = {
            let ab = homothethy.moving_corner - homothethy.fixed_corner;
            ab.y.atan2(ab.x)
        };
        let angle = angle_now - angle_origin;
        let scale = (homothethy.moving_corner - homothethy.fixed_corner).mag()
            / (homothethy.origin_moving_corner - homothethy.fixed_corner).mag();
        for path in paths_mut.values_mut() {
            for v in path.vertices_mut() {
                if v.plane_id == homothethy.plane_id {
                    let transform = |v: &mut Vec2| {
                        let vec = *v - homothethy.fixed_corner;
                        if vec.mag() > 1e-6 {
                            let new_norm = vec.mag() * scale;
                            *v = vec.normalized().rotated_by(Rotor2::from_angle(angle)) * new_norm
                                + homothethy.fixed_corner;
                        }
                    };
                    transform(&mut v.position);
                    if let Some(vec) = v.position_out.as_mut() {
                        transform(vec);
                    }
                    if let Some(vec) = v.position_in.as_mut() {
                        transform(vec);
                    }
                }
            }
        }
        drop(paths_mut);
    }

    pub fn create_camera(
        design: &mut Design,
        position: Vec3,
        orientation: Rotor3,
        pivot_position: Option<Vec3>,
    ) {
        design.add_camera(position, orientation, pivot_position);
    }

    pub fn delete_camera(design: &mut Design, id: CameraId) -> Result<(), OperationError> {
        if !design.rm_camera(id) {
            Err(OperationError::CameraDoesNotExist(id))
        } else {
            Ok(())
        }
    }

    pub fn set_camera_name(
        design: &mut Design,
        id: CameraId,
        name: String,
    ) -> Result<(), OperationError> {
        if let Some(camera) = design.get_camera_mut(id) {
            camera.name = name;
            Ok(())
        } else {
            Err(OperationError::CameraDoesNotExist(id))
        }
    }

    pub fn is_changing_color(&self) -> bool {
        matches!(self.state, ControllerState::ChangingColor)
    }

    pub fn get_strand_builders(&self) -> &[StrandBuilder] {
        if let ControllerState::BuildingStrand { builders, .. } = &self.state {
            builders.as_slice()
        } else {
            &[]
        }
    }

    pub fn apply_translation(
        state: &mut AppState,
        translation: DesignTranslation,
    ) -> Result<(), OperationError> {
        match translation.target {
            IsometryTarget::Helices(helices, snap) => {
                Self::do_translate_helices(state, snap, helices, translation.translation)?;
            }
            IsometryTarget::Grids(grid_ids) => {
                Self::translate_grids(state, grid_ids, translation.translation)?;
            }
            IsometryTarget::GroupPivot(group_id) => {
                Self::translate_group_pivot(state, translation.translation, group_id)?;
            }
            IsometryTarget::ControlPoint(control_points) => {
                Self::translate_control_points(state, control_points, translation.translation)?;
            }
        }

        if let Some(group_id) = translation.group_id {
            let pivot = state
                .0
                .make_mut()
                .design
                .make_mut()
                .design
                .make_mut()
                .group_attributes
                .get_mut(&group_id)
                .and_then(|attributes| attributes.pivot.as_mut())
                .ok_or(OperationError::GroupHasNoPivot(group_id))?;
            pivot.position += translation.translation;
        }
        Ok(())
    }

    fn translate_group_pivot(
        state: &mut AppState,
        translation: Vec3,
        group_id: GroupId,
    ) -> Result<(), OperationError> {
        let pivot = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .group_attributes
            .get_mut(&group_id)
            .and_then(|attributes| attributes.pivot.as_mut())
            .ok_or(OperationError::GroupHasNoPivot(group_id))?;
        pivot.position += translation;
        Ok(())
    }

    fn rotate_group_pivot(
        state: &mut AppState,
        rotation: Rotor3,
        group_id: GroupId,
    ) -> Result<(), OperationError> {
        let pivot = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .group_attributes
            .get_mut(&group_id)
            .and_then(|attributes| attributes.pivot.as_mut())
            .ok_or(OperationError::GroupHasNoPivot(group_id))?;
        pivot.orientation = rotation * pivot.orientation;
        Ok(())
    }

    pub fn attach_object(
        design: &mut Design,
        object: GridObject,
        grid: GridId,
        x: isize,
        y: isize,
    ) -> Result<(), OperationError> {
        attach_object_to_grid(design, object, grid, x, y)?;

        Ok(())
    }

    pub fn apply_rotation(
        state: &mut AppState,
        rotation: DesignRotation,
    ) -> Result<(), OperationError> {
        match rotation.target {
            IsometryTarget::GroupPivot(g_id) => {
                Self::rotate_group_pivot(state, rotation.rotation, g_id)?;
            }
            IsometryTarget::Helices(helices, snap) => {
                Self::rotate_helices_3d(state, snap, helices, rotation.rotation, rotation.origin)?;
            }
            IsometryTarget::Grids(grid_ids) => {
                Self::rotate_grids(state, grid_ids, rotation.rotation, rotation.origin);
            }
            IsometryTarget::ControlPoint(_) => {
                return Err(OperationError::NotImplemented);
            }
        }

        if let Some(group_id) = rotation.group_id {
            let pivot = state
                .0
                .make_mut()
                .design
                .make_mut()
                .design
                .make_mut()
                .group_attributes
                .get_mut(&group_id)
                .and_then(|attributes| attributes.pivot.as_mut())
                .ok_or(OperationError::GroupHasNoPivot(group_id))?;
            pivot.orientation = rotation.rotation * pivot.orientation;
        }
        Ok(())
    }

    pub fn do_translate_helices(
        state: &mut AppState,
        snap: bool,
        helices: Vec<usize>,
        translation: Vec3,
    ) -> Result<(), DesignOperationError> {
        translate_helices(state.design_mut(), snap, helices, translation)
    }

    fn translate_control_points(
        state: &mut AppState,
        control_points: Vec<(usize, BezierControlPoint)>,
        translation: Vec3,
    ) -> Result<(), OperationError> {
        let grid_data = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .get_updated_grid_data();
        let translations: Vec<_> = control_points
            .iter()
            .copied()
            .map(|cp| grid_data.translate_bezier_point(cp, translation))
            .collect();
        let mut new_helices = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .helices
            .make_mut();
        for ((h_id, control), translation) in control_points.iter().zip(translations.iter()) {
            let translation = translation.ok_or(OperationError::BadSelection)?;
            if let Some(helix) = new_helices.get_mut(h_id) {
                helix.translate_bezier_point(*control, translation)?;
            }
        }
        Ok(())
    }

    fn rotate_helices_3d(
        state: &mut AppState,
        snap: bool,
        helices: Vec<usize>,
        rotation: Rotor3,
        origin: Vec3,
    ) -> Result<(), DesignOperationError> {
        rotate_helices_3d(state.design_mut(), snap, helices, rotation, origin)
    }

    fn translate_grids(
        state: &mut AppState,
        grid_ids: Vec<GridId>,
        translation: Vec3,
    ) -> Result<(), OperationError> {
        let mut new_paths = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .bezier_paths
            .make_mut();
        for g_id in &grid_ids {
            if let GridId::BezierPathGrid(vertex_id) = g_id {
                let path = new_paths
                    .get_mut(&vertex_id.path_id)
                    .ok_or(OperationError::PathDoesNotExist(vertex_id.path_id))?;
                let vertex = path.get_vertex_mut(vertex_id.vertex_id).ok_or(
                    OperationError::VertexDoesNotExist(vertex_id.path_id, vertex_id.vertex_id),
                )?;
                vertex.add_translation(translation);
            }
        }
        drop(new_paths);
        let mut new_grids = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .free_grids
            .make_mut();
        for g_id in grid_ids {
            if let Some(desc) =
                FreeGridId::try_from_grid_id(g_id).and_then(|g_id| new_grids.get_mut(&g_id))
            {
                desc.position += translation;
            }
        }
        Ok(())
    }

    fn rotate_grids(state: &mut AppState, grid_ids: Vec<GridId>, rotation: Rotor3, origin: Vec3) {
        let bezier_paths = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .get_up_to_date_paths();
        let mut new_vectors_out = BTreeMap::new();

        for g_id in &grid_ids {
            if let GridId::BezierPathGrid(vertex_id) = g_id
                && let Some(old_vector_out) = bezier_paths.get_vector_out(*vertex_id)
            {
                let new_vector_out = old_vector_out.rotated_by(rotation);
                new_vectors_out.insert(vertex_id, new_vector_out);
            }
        }

        let bezier_planes = state.design().bezier_planes.clone();
        let mut new_paths = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .bezier_paths
            .make_mut();
        for (vertex_id, new_vector_out) in new_vectors_out {
            if let Some(path) = new_paths.get_mut(&vertex_id.path_id) {
                path.set_vector_out(vertex_id.vertex_id, new_vector_out, &bezier_planes);
            }
        }

        drop(new_paths);

        let mut new_grids = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .free_grids
            .make_mut();
        for g_id in grid_ids {
            if let Some(desc) = new_grids.get_mut_g_id(&g_id) {
                desc.position -= origin;
                desc.orientation = rotation * desc.orientation;
                desc.position = rotation * desc.position;
                desc.position += origin;
            }
        }
    }

    pub fn fancy_recolor_staples(state: &mut AppState) {
        let mut drawing_styles = HashMap::<DesignElementKey, DrawingStyle>::default();

        if let Some(t) = &state.design().organizer_tree {
            // Read drawing style -> this should be a function on its own, the exact same code is used in design-content
            let prefix = "style:"; // PREFIX SHOULD BELONG TO CONST.RS
            let h = t.get_hashmap_to_all_group_names_with_prefix(prefix);
            for (e, names) in h {
                let drawing_attributes = names
                    .iter()
                    .flat_map(|x| {
                        x.split(&[' ', ':'])
                            .flat_map(DrawingAttribute::from_str)
                            .collect::<Vec<DrawingAttribute>>()
                    })
                    .collect::<Vec<DrawingAttribute>>();
                let style = DrawingStyle::from(drawing_attributes);
                drawing_styles.insert(e, style);
            }
        }

        let scaffold_id = state.design().scaffold_id;

        let (design, controller) = state.design_controller_mut();

        for (s_id, strand) in design.strands.iter_mut() {
            // recoloring only concerns the non-scaffold strands
            if Some(*s_id) != scaffold_id {
                // Compute strand drawing style
                let strand_style = drawing_styles
                    .get(&DesignElementKey::Strand(*s_id))
                    .copied()
                    .unwrap_or_default();

                let color = if let Some(shade) = strand_style.color_shade {
                    random_color_with_shade(shade, strand_style.hue_range)
                } else {
                    new_color(&mut controller.color_idx)
                };

                strand.color = color;
            }
        }
    }

    pub fn set_scaffold_sequence(design: &mut Design, sequence: String, shift: usize) {
        design.scaffold_sequence = Some(sequence);
        design.scaffold_shift = Some(shift);
    }

    pub fn set_scaffold_shift(state: &mut AppState, shift: usize) {
        if matches!(
            state.controller().state,
            ControllerState::OptimizingScaffoldPosition
        ) {
            state
                .0
                .make_mut()
                .design
                .make_mut()
                .controller
                .make_mut()
                .state = ControllerState::Normal;
        }
        state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .scaffold_shift = Some(shift);
    }

    pub fn change_color_strands(state: &mut AppState, color: u32, strands: Vec<usize>) {
        state
            .0
            .make_mut()
            .design
            .make_mut()
            .controller
            .make_mut()
            .state = ControllerState::ChangingColor;
        for s_id in &strands {
            if let Some(strand) = state
                .0
                .make_mut()
                .design
                .make_mut()
                .design
                .make_mut()
                .strands
                .get_mut(s_id)
            {
                strand.color = color;
            }
        }
    }

    pub fn set_helices_persistence(state: &mut AppState, grid_ids: Vec<GridId>, persistent: bool) {
        for g_id in grid_ids {
            if persistent {
                Arc::make_mut(
                    &mut state
                        .0
                        .make_mut()
                        .design
                        .make_mut()
                        .design
                        .make_mut()
                        .no_phantoms,
                )
                .remove(&g_id);
            } else {
                Arc::make_mut(
                    &mut state
                        .0
                        .make_mut()
                        .design
                        .make_mut()
                        .design
                        .make_mut()
                        .no_phantoms,
                )
                .insert(g_id);
            }
        }
    }

    pub fn set_small_spheres(state: &mut AppState, grid_ids: Vec<GridId>, small: bool) {
        for g_id in grid_ids {
            if small {
                Arc::make_mut(&mut state.design_mut().small_spheres).insert(g_id);
            } else {
                Arc::make_mut(&mut state.design_mut().small_spheres).remove(&g_id);
            }
        }
    }

    pub fn snap_helices(state: &mut AppState, pivots: Vec<(Nucl, usize)>, translation: Vec2) {
        let mut new_helices = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .helices
            .make_mut();
        for (p, segment_idx) in &pivots {
            if let Some(old_pos) = nucl_pos_2d(new_helices.as_ref(), p, *segment_idx)
                && let Some(h) = new_helices.get_mut(&p.helix)
            {
                let position = old_pos + translation;
                let position = Vec2::new(position.x.round(), position.y.round());
                let isometry = if *segment_idx > 0 {
                    h.additional_isometries
                        .get_mut(segment_idx - 1)
                        .and_then(|i| i.additional_isometry.as_mut())
                } else {
                    h.isometry2d.as_mut()
                };
                if let Some(isometry) = isometry {
                    isometry.append_translation(position - old_pos);
                }
            }
        }
    }

    pub fn set_isometry(
        state: &mut AppState,
        h_id: usize,
        segment_idx: usize,
        isometry: Isometry2,
    ) {
        log::info!("setting isometry {h_id} {segment_idx} {isometry:?}");
        let mut new_helices = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .helices
            .make_mut();
        if segment_idx == 0 {
            if let Some(h) = new_helices.get_mut(&h_id) {
                h.isometry2d = Some(isometry);
            }
        } else if let Some(i) = new_helices
            .get_mut(&h_id)
            .and_then(|h| h.additional_isometries.get_mut(segment_idx - 1))
        {
            i.additional_isometry = Some(isometry);
        }
    }

    pub fn apply_symmetry_to_helices(
        state: &mut AppState,
        helices_id: Vec<usize>,
        centers: Vec<Vec2>,
        symmetry: Vec2,
    ) {
        let mut new_helices = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .helices
            .make_mut();
        for (h_id, center) in helices_id.iter().zip(centers.iter()) {
            if let Some(h) = new_helices.get_mut(h_id) {
                if let Some(isometry) = h.isometry2d.as_mut() {
                    isometry.translation -= *center;
                    isometry.translation.rotate_by(isometry.rotation.reversed());
                    let mut new_rotation = isometry.rotation.into_matrix().into_homogeneous();
                    new_rotation[0] *= symmetry.x;
                    new_rotation[1] *= symmetry.y;
                    isometry.translation = new_rotation.transform_vec2(isometry.translation);
                    isometry.translation += *center;
                }
                h.symmetry *= symmetry;
            }
        }
        drop(new_helices);
    }

    pub fn rotate_helices(state: &mut AppState, helices: Vec<usize>, center: Vec2, angle: f32) {
        let step = PI / 12.; // 15 degrees
        let angle = {
            let k = (angle / step).round();
            k * step
        };
        let mut new_helices = state
            .0
            .make_mut()
            .design
            .make_mut()
            .design
            .make_mut()
            .helices
            .make_mut();
        for h_id in &helices {
            if let Some(h) = new_helices.get_mut(h_id)
                && let Some(isometry) = h.isometry2d.as_mut()
            {
                isometry.append_translation(-center);
                isometry.append_rotation(Rotor2::from_angle(angle));
                isometry.append_translation(center);
            }
        }
        drop(new_helices);
    }

    pub fn request_strand_builders(
        state: &mut AppState,
        nucls: Vec<Nucl>,
    ) -> Result<(), OperationError> {
        let mut builders = Vec::with_capacity(nucls.len());
        let ignored_domains: Vec<_> = nucls
            .iter()
            .filter_map(|nucl| {
                get_neighbor_nucl(state.design_mut(), *nucl).map(|neighbor| neighbor.identifier)
            })
            .collect();
        for nucl in nucls {
            builders.push(
                Self::request_one_builder(state, nucl, &ignored_domains)
                    .ok_or(OperationError::CannotBuildOn(nucl))?,
            );
        }
        log::info!("Ignored domains: {ignored_domains:?}");
        let initial_design = state.interactor().design.clone();
        state.controller_mut().state = ControllerState::BuildingStrand {
            builders,
            initializing: true,
            // The initial design is indeed the one AFTER adding the new strands
            initial_design,
            ignored_domains,
        };

        Ok(())
    }

    fn request_one_builder(
        state: &mut AppState,
        nucl: Nucl,
        ignored_domains: &[DomainIdentifier],
    ) -> Option<StrandBuilder> {
        // if there is a strand that passes through the nucleotide
        if state.design().strands.get_strand_nucl(&nucl).is_some() {
            Self::strand_builder_on_existing(state, nucl, ignored_domains)
        } else {
            Self::new_strand_builder(state, nucl)
        }
    }

    fn strand_builder_on_existing(
        state: &AppState,
        nucl: Nucl,
        ignored_domains: &[DomainIdentifier],
    ) -> Option<StrandBuilder> {
        let left = get_neighbor_nucl(state.design(), nucl.left())
            .filter(|n| !ignored_domains.contains(&n.identifier));
        let right = get_neighbor_nucl(state.design(), nucl.right())
            .filter(|n| !ignored_domains.contains(&n.identifier));
        let axis = state
            .design()
            .helices
            .get(&nucl.helix)
            .map(|h| h.get_axis(&state.design().helix_parameters.unwrap_or_default()))?;
        let desc = get_neighbor_nucl(state.design(), nucl)?;
        let strand_id = desc.identifier.strand;
        let filter = |d: &NeighborDescriptor| !(d.identifier.is_same_domain_than(&desc.identifier));
        let neighbor_desc = left.filter(filter).or_else(|| right.filter(filter));
        // stick to the neighbor if it is its direct neighbor. This is because we want don't want
        // to create a gap between neighboring domains
        let stick = neighbor_desc
            .filter(|d| {
                (d.identifier.domain as isize - desc.identifier.domain as isize).abs() < 1
                    && d.identifier.strand == desc.identifier.strand
            })
            .is_some();
        log::info!("stick {stick}");
        if left
            .filter(filter)
            .and_then(|_| right.filter(filter))
            .is_some()
        {
            // TODO: maybe we should do something else ?
            return None;
        }
        let other_end = desc
            .identifier
            .other_end()
            .filter(|d| !ignored_domains.contains(d))
            .is_some()
            .then_some(desc.fixed_end);
        match state.design().strands.get(&strand_id).map(Strand::length) {
            Some(n) if n > 1 => Some(StrandBuilder::init_existing(
                desc.identifier,
                nucl,
                axis.to_owned(),
                other_end,
                neighbor_desc,
                stick,
            )),
            _ => Some(StrandBuilder::init_empty(
                DomainIdentifier {
                    strand: strand_id,
                    domain: 0,
                    start: None,
                },
                nucl,
                axis.to_owned(),
                neighbor_desc,
            )),
        }
    }

    fn new_strand_builder(state: &mut AppState, nucl: Nucl) -> Option<StrandBuilder> {
        let left = get_neighbor_nucl(state.design(), nucl.left());
        let right = get_neighbor_nucl(state.design(), nucl.right());
        if left.is_some() && right.is_some() {
            return None;
        }
        let new_key = Self::init_strand(state, nucl);
        let axis = state
            .design()
            .helices
            .get(&nucl.helix)
            .map(|h| h.get_axis(&state.design().helix_parameters.unwrap_or_default()))?;
        Some(StrandBuilder::init_empty(
            DomainIdentifier {
                strand: new_key,
                domain: 0,
                start: None,
            },
            nucl,
            axis.to_owned(),
            left.or(right),
        ))
    }

    fn init_strand(state: &mut AppState, nucl: Nucl) -> usize {
        let s_id = state.design().strands.keys().max().map_or(0, |n| n + 1);
        let color = new_color(&mut state.controller_mut().color_idx);
        state.design_mut().strands.insert(
            s_id,
            Strand::init(nucl.helix, nucl.position, nucl.forward, color),
        );
        s_id
    }

    fn add_strand(state: &mut AppState, helix: usize, position: isize, forward: bool) -> usize {
        let new_key = if let Some(k) = state.design().strands.keys().max() {
            *k + 1
        } else {
            0
        };
        let color = new_color(&mut state.controller_mut().color_idx);
        state
            .design_mut()
            .strands
            .insert(new_key, Strand::init(helix, position, forward, color));
        new_key
    }

    pub fn move_strand_builders(state: &mut AppState, n: isize) -> Result<(), OperationError> {
        let current_design = state.design().clone();

        if let ControllerState::BuildingStrand {
            initial_design,
            builders,
            initializing,
            ignored_domains,
        } = &mut state.controller_mut().state
        {
            let delta = builders
                .first()
                .map_or(0, |b| n - b.get_moving_end_position());
            let mut design = initial_design.clone_inner();
            if builders.len() > 1 {
                let sign = delta.signum();
                let mut blocked = false;
                if delta != 0 {
                    for i in 0..(sign * delta) {
                        let mut copy_builder = builders.clone();
                        for builder in &mut copy_builder {
                            if (sign > 0 && !builder.try_incr(&current_design, ignored_domains))
                                || (sign < 0 && !builder.try_decr(&current_design, ignored_domains))
                            {
                                blocked = true;
                                break;
                            }
                        }
                        if blocked {
                            if i == 0 {
                                return Ok(());
                            }
                            break;
                        }
                        *builders = copy_builder;
                        for builder in builders.iter_mut() {
                            builder.update(&mut design);
                        }
                    }
                } else {
                    return Ok(());
                }
            } else {
                for builder in builders.iter_mut() {
                    let to = builder.get_moving_end_position() + delta;
                    builder.move_to(to, &mut design, ignored_domains);
                }
            }
            *initializing = false;
            *state.design_mut() = design;
            Ok(())
        } else {
            Err(OperationError::IncompatibleState(
                "Not building strand".into(),
            ))
        }
    }

    pub fn delete_xovers(
        state: &mut AppState,
        xovers: &[(Nucl, Nucl)],
    ) -> Result<(), OperationError> {
        let (design, controller) = state.design_controller_mut();
        for (n1, _) in xovers {
            let _ = Self::split_strand(&mut design.strands, n1, None, &mut controller.color_idx)?;
        }
        Ok(())
    }

    pub fn cut(state: &mut AppState, nucl: Nucl) -> Result<(), OperationError> {
        let (design, controller) = state.design_controller_mut();
        let _ = Self::split_strand(&mut design.strands, &nucl, None, &mut controller.color_idx)?;
        Ok(())
    }

    /// Split a strand at nucl, and return the id of the newly created strand.
    ///
    /// The part of the strand that contains nucl is given the original
    /// strand's id, the other part is given a new id.
    ///
    /// If `force_end` is `Some(true)`, nucl will be on the 3 prime half of the split.
    /// If `force_end` is `Some(false)` nucl will be on the 5 prime half of the split.
    /// If `force_end` is `None`, nucl will be on the 5 prime half of the split unless nucl is the 3
    /// prime extremity of a crossover, in which case nucl will be on the 3 prime half of the
    /// split.
    pub fn split_strand(
        strands: &mut Strands,
        nucl: &Nucl,
        force_end: Option<bool>,
        color_idx: &mut usize,
    ) -> Result<usize, OperationError> {
        let id = strands
            .get_strand_nucl(nucl)
            .ok_or(OperationError::CutNonExistentStrand)?;

        let strand = strands.remove(&id).expect("strand");
        let name = strand.name.clone();
        if strand.is_cyclic {
            let new_strand = Self::break_cycle(strand.clone(), *nucl, force_end);
            strands.insert(id, new_strand);
            return Ok(id);
        }
        if strand.length() <= 1 {
            // return without putting the strand back
            return Err(OperationError::CutNonExistentStrand);
        }
        let mut i = strand.domains.len();
        let mut prim5_domains = Vec::new();
        let mut len_prim5 = 0;
        let mut domains = None;
        let mut on_3prime = force_end.unwrap_or(false);
        let mut prev_helix = None;
        let mut prime5_junctions: Vec<DomainJunction> = Vec::new();
        let mut prime3_junctions: Vec<DomainJunction> = Vec::new();
        let mut prim3_domains = Vec::new();

        log::info!("Splitting");
        log::info!("{:?}", strand.domains);
        log::info!("{:?}", strand.junctions);

        for (d_id, domain) in strand.domains.iter().enumerate() {
            if domain.prime5_end() == Some(*nucl)
                && prev_helix != domain.half_helix()
                && force_end != Some(false)
            {
                // nucl is the 5' end of the next domain so it is the on the 3' end of a xover.
                // nucl is not required to be on the 5' half of the split, so we put it on the 3'
                // half
                on_3prime = true;
                i = d_id;
                let move_last_insertion = if let Some(Domain::Insertion {
                    attached_to_prime3,
                    ..
                }) = prim5_domains.last()
                {
                    *attached_to_prime3
                } else {
                    false
                };

                // the insertion is currently
                if move_last_insertion {
                    prim3_domains = vec![prim5_domains.pop().unwrap()];
                    prime3_junctions.push(DomainJunction::Adjacent);
                }

                if let Some(j) = prime5_junctions.last_mut() {
                    *j = DomainJunction::Prime3;
                }
                break;
            } else if domain.prime3_end() == Some(*nucl) && force_end != Some(true) {
                // nucl is the 3' end of the current domain so it is the on the 5' end of a xover.
                // nucl is not required to be on the 3' half of the split, so we put it on the 5'
                // half
                i = d_id + 1;
                prim5_domains.push(domain.clone());
                len_prim5 += domain.length();
                prime5_junctions.push(DomainJunction::Prime3);
                break;
            } else if let Some(n) = domain.has_nucl(nucl) {
                let n = if force_end == Some(true) { n - 1 } else { n };
                i = d_id;
                len_prim5 += n;
                domains = domain.split(n);
                prime5_junctions.push(DomainJunction::Prime3);
                prime3_junctions.push(strand.junctions[d_id].clone());
                break;
            }

            len_prim5 += domain.length();
            prim5_domains.push(domain.clone());
            prime5_junctions.push(strand.junctions[d_id].clone());
            prev_helix = domain.half_helix();
        }

        if let Some(domains) = &domains {
            prim5_domains.push(domains.0.clone());
            prim3_domains.push(domains.1.clone());
            i += 1;
        }

        for n in i..strand.domains.len() {
            let domain = &strand.domains[n];
            prim3_domains.push(domain.clone());
            prime3_junctions.push(strand.junctions[n].clone());
        }

        let seq_prim5;
        let seq_prim3;
        if let Some(seq) = strand.sequence {
            let seq = seq.into_owned();
            let chars = seq.chars();
            seq_prim5 = Some(Cow::Owned(chars.clone().take(len_prim5).collect()));
            seq_prim3 = Some(Cow::Owned(chars.clone().skip(len_prim5).collect()));
        } else {
            seq_prim3 = None;
            seq_prim5 = None;
        }

        log::info!("prime5 {prim5_domains:?}");
        log::info!("prime5 {prime5_junctions:?}");
        log::info!("prime3 {prim3_domains:?}");
        log::info!("prime3 {prime3_junctions:?}");

        let mut strand_5prime = Strand {
            domains: prim5_domains,
            color: strand.color,
            junctions: prime5_junctions,
            is_cyclic: false,
            sequence: seq_prim5,
            name: name.clone(),
        };

        let mut strand_3prime = Strand {
            domains: prim3_domains,
            color: strand.color,
            is_cyclic: false,
            junctions: prime3_junctions,
            sequence: seq_prim3,
            name,
        };
        let new_id = (*strands.keys().max().unwrap_or(&0)).max(id) + 1;
        log::info!("new id {new_id}; id {id}");
        let (id_5prime, id_3prime) = if !on_3prime {
            strand_3prime.color = Self::new_color(color_idx);
            (id, new_id)
        } else {
            strand_5prime.color = Self::new_color(color_idx);
            (new_id, id)
        };
        if !strand_5prime.domains.is_empty() {
            strands.insert(id_5prime, strand_5prime);
        }
        if !strand_3prime.domains.is_empty() {
            strands.insert(id_3prime, strand_3prime);
        }

        Ok(new_id)
    }

    /// Split a cyclic strand at nucl.
    ///
    /// If `force_end` is `Some(true)`, nucl will be the new 5' end of the strand.
    /// If `force_end` is `Some(false)` nucl will be the new 3' end of the strand.
    /// If `force_end` is `None`, nucl will be the new 3' end of the strand unless nucl is the 3'
    /// prime extremity of a crossover, in which case nucl will be the new 5' end of the strand.
    fn break_cycle(mut strand: Strand, nucl: Nucl, force_end: Option<bool>) -> Strand {
        let mut last_dom = None;
        let mut replace_last_dom = None;
        let mut prev_helix = None;

        let mut junctions: Vec<DomainJunction> = Vec::with_capacity(strand.domains.len());

        for (i, domain) in strand.domains.iter().enumerate() {
            if domain.prime5_end() == Some(nucl)
                && prev_helix != domain.helix()
                && force_end != Some(false)
            {
                last_dom = if i != 0 {
                    Some(i - 1)
                } else {
                    Some(strand.domains.len() - 1)
                };

                break;
            } else if domain.prime3_end() == Some(nucl) && force_end != Some(true) {
                last_dom = Some(i);
                break;
            } else if let Some(n) = domain.has_nucl(&nucl) {
                let n = if force_end == Some(true) { n - 1 } else { n };
                last_dom = Some(i);
                replace_last_dom = domain.split(n);
            }
            prev_helix = domain.helix();
        }
        let last_dom = last_dom.expect("Could not find nucl in strand");
        let mut new_domains = Vec::new();
        if let Some((_, d2)) = &replace_last_dom {
            new_domains.push(d2.clone());
            junctions.push(strand.junctions[last_dom].clone());
        }
        for (i, d) in strand.domains.iter().enumerate().skip(last_dom + 1) {
            new_domains.push(d.clone());
            junctions.push(strand.junctions[i].clone());
        }
        for (i, d) in strand.domains.iter().enumerate().take(last_dom) {
            new_domains.push(d.clone());
            junctions.push(strand.junctions[i].clone());
        }

        if let Some((d1, _)) = &replace_last_dom {
            new_domains.push(d1.clone());
        } else {
            new_domains.push(strand.domains[last_dom].clone());
        }
        junctions.push(DomainJunction::Prime3);

        strand.domains = new_domains;
        strand.is_cyclic = false;
        strand.junctions = junctions;
        strand
    }

    pub fn add_grid_helix(
        state: &mut AppState,
        position: HelixGridPosition,
        start: isize,
        length: usize,
    ) -> Result<(), OperationError> {
        let grid_manager = state.design_mut().get_updated_grid_data();
        if grid_manager.pos_to_object(position.light()).is_some() {
            return Err(OperationError::GridPositionAlreadyUsed);
        }
        let helix = if let GridId::BezierPathGrid(BezierVertexId { path_id, .. }) = position.grid {
            Helix::new_on_bezier_path(grid_manager, position, path_id)
        } else {
            let grid = grid_manager
                .grids
                .get(&position.grid)
                .ok_or(OperationError::GridDoesNotExist(position.grid))?;
            Ok(Helix::new_on_grid(
                grid,
                position.x,
                position.y,
                position.grid,
            ))
        }?;
        let helix_id = state.design_mut().helices.make_mut().push_helix(helix);

        if length > 0 {
            for b in &[false, true] {
                let new_key = Self::add_strand(state, helix_id, start, *b);
                if let Domain::HelixDomain(ref mut dom) = state
                    .design_mut()
                    .strands
                    .get_mut(&new_key)
                    .unwrap()
                    .domains[0]
                {
                    dom.end = dom.start + length as isize;
                }
            }
        }
        Ok(())
    }

    pub fn add_two_points_bezier(
        state: &mut AppState,
        start: HelixGridPosition,
        end: HelixGridPosition,
    ) -> Result<(), OperationError> {
        log::info!("Add {start:?} {end:?}");
        let grid_manager = state.design_mut().get_updated_grid_data();
        if let Some(obj) = grid_manager.pos_to_object(start.light()) {
            if grid_manager.pos_to_object(end.light()).is_some() {
                return Err(OperationError::GridPositionAlreadyUsed);
            }
            let (_, tangent) = grid_manager
                .get_tangents_between_two_points(start.light(), end.light())
                .ok_or(OperationError::GridDoesNotExist(end.grid))?;
            return Self::add_bezier_point(state, obj, end.light(), tangent, true);
        } else if let Some(obj) = grid_manager.pos_to_object(end.light()) {
            let (tangent, _) = grid_manager
                .get_tangents_between_two_points(start.light(), end.light())
                .ok_or(OperationError::GridDoesNotExist(end.grid))?;
            return Self::add_bezier_point(state, obj, start.light(), tangent, false);
        }
        let helix = Helix::new_bezier_two_points(grid_manager, start, end)?;

        let length = helix.nb_bezier_nucls();

        let helix_id = state.design_mut().helices.make_mut().push_helix(helix);

        if length > 0 {
            for b in &[false, true] {
                let new_key = Self::add_strand(state, helix_id, 0, *b);
                if let Domain::HelixDomain(ref mut dom) = state
                    .design_mut()
                    .strands
                    .get_mut(&new_key)
                    .unwrap()
                    .domains[0]
                {
                    dom.end = dom.start + length as isize;
                }
            }
        }
        Ok(())
    }

    fn add_bezier_point(
        state: &mut AppState,
        object: GridObject,
        point: GridPosition,
        _tangent: Vec3,
        append: bool,
    ) -> Result<(), OperationError> {
        match object {
            GridObject::BezierPoint { helix_id, n } => {
                let mut helices_mut = state.design_mut().helices.make_mut();
                let helix_ref = helices_mut
                    .get_mut(&helix_id)
                    .ok_or(OperationError::HelixDoesNotExists(helix_id))?;
                let desc: Option<&mut CurveDescriptor> =
                    if let Some(desc) = helix_ref.curve.as_mut() {
                        Some(Arc::make_mut(desc))
                    } else {
                        None
                    };
                if let Some(CurveDescriptor::PiecewiseBezier { points, .. }) = desc {
                    let insertion_point = if append { n + 1 } else { n };
                    points.insert(
                        insertion_point,
                        BezierEnd {
                            position: point,
                            inward_coeff: 1.,
                            outward_coeff: 1.,
                        },
                    );
                    Ok(())
                } else {
                    Err(OperationError::NotPiecewiseBezier(helix_id))
                }
            }
            GridObject::Helix(_) => Err(OperationError::GridPositionAlreadyUsed),
        }
    }

    /// Merge two strands with identifier prime5 and prime3. The resulting strand will have
    /// identifier prime5.
    fn merge_strands(
        strands: &mut Strands,
        prime5: usize,
        prime3: usize,
    ) -> Result<(), OperationError> {
        // We panic, if we can't find the strand, because this means that the program has a bug
        if prime5 != prime3 {
            let strand5prime = strands
                .remove(&prime5)
                .ok_or(OperationError::StrandDoesNotExist(prime5))?;
            let strand3prime = strands
                .remove(&prime3)
                .ok_or(OperationError::StrandDoesNotExist(prime3))?;
            let name = strand5prime.name.or(strand3prime.name);
            let len = strand5prime.domains.len() + strand3prime.domains.len();
            let mut domains = Vec::with_capacity(len);
            let mut junctions = Vec::with_capacity(len);
            for (i, domain) in strand5prime.domains.iter().enumerate() {
                domains.push(domain.clone());
                junctions.push(strand5prime.junctions[i].clone());
            }
            let junction_between_merge = {
                let last_interval_prime5 = strand5prime
                    .domains
                    .iter()
                    .rev()
                    .find(|d| matches!(d, Domain::HelixDomain(_)))
                    .ok_or(OperationError::EmptyOrigin)?;
                let first_interval_prime3 = strand5prime
                    .domains
                    .iter()
                    .rev()
                    .find(|d| matches!(d, Domain::HelixDomain(_)))
                    .ok_or(OperationError::EmptyOrigin)?;
                if last_interval_prime5.can_merge(first_interval_prime3) {
                    DomainJunction::Adjacent
                } else {
                    DomainJunction::UnidentifiedXover
                }
            };
            let skip_domain;
            let skip_junction;
            let last_helix = domains.last().and_then(Domain::half_helix);
            let next_helix = strand3prime.domains.first().and_then(Domain::half_helix);
            if last_helix == next_helix
                && domains
                    .last()
                    .unwrap()
                    .can_merge(strand3prime.domains.first().unwrap())
            {
                skip_domain = 1;
                skip_junction = 0;
                domains
                    .last_mut()
                    .as_mut()
                    .unwrap()
                    .merge(strand3prime.domains.first().unwrap());
                junctions.pop();
            } else {
                if let Some(j) = junctions.iter_mut().last() {
                    *j = junction_between_merge;
                }
                if let Some(Domain::Insertion { .. }) = strand3prime.domains.first() {
                    skip_domain = 1;
                    skip_junction = 1;
                    // the last domain is not an insertion in this case
                    domains.push(strand3prime.domains.first().unwrap().clone());
                    let insertion_idx = junctions.len() - 1;
                    junctions.insert(insertion_idx, DomainJunction::Adjacent);
                } else {
                    skip_domain = 0;
                    skip_junction = 0;
                }
            }
            for domain in strand3prime.domains.iter().skip(skip_domain) {
                domains.push(domain.clone());
            }
            for junction in strand3prime.junctions.iter().skip(skip_junction) {
                junctions.push(junction.clone());
            }
            let sequence = if let Some((seq5, seq3)) = strand5prime
                .sequence
                .clone()
                .zip(strand3prime.sequence.clone())
            {
                let new_seq = seq5.into_owned() + &seq3;
                Some(Cow::Owned(new_seq))
            } else if let Some(seq5) = &strand5prime.sequence {
                Some(seq5.clone())
            } else {
                strand3prime.sequence.clone()
            };
            let mut new_strand = Strand {
                domains,
                color: strand5prime.color,
                sequence,
                junctions,
                is_cyclic: false,
                name,
            };
            new_strand.merge_consecutive_domains();
            strands.insert(prime5, new_strand);
            Ok(())
        } else {
            // To make a cyclic strand use `make_cyclic_strand` instead
            Err(OperationError::MergingSameStrand)
        }
    }

    /// Make a strand cyclic by linking the 3' and the 5' end, or undo this operation.
    #[expect(clippy::panic_in_result_fn)] // FIXME
    fn make_cycle(
        strands: &mut Strands,
        strand_id: usize,
        cyclic: bool,
    ) -> Result<(), OperationError> {
        strands
            .get_mut(&strand_id)
            .ok_or(OperationError::StrandDoesNotExist(strand_id))?
            .is_cyclic = cyclic;

        let strand = strands
            .get_mut(&strand_id)
            .ok_or(OperationError::StrandDoesNotExist(strand_id))?;
        if cyclic {
            let (merge_insertions, replace) = match (strand.domains.first(), strand.domains.last())
            {
                (
                    Some(Domain::Insertion { nb_nucl: n1, .. }),
                    Some(Domain::Insertion { nb_nucl: n2, .. }),
                ) => (Some(n1 + n2), true),
                (Some(Domain::Insertion { nb_nucl, .. }), _) => (Some(*nb_nucl), false),
                _ => (None, false),
            };

            if let Some(n) = merge_insertions {
                // If the strand starts and finishes by an Insertion, merge the insertions.
                // TODO UNITTEST for this specific case
                if replace {
                    *strand.domains.last_mut().unwrap() = Domain::new_insertion(n);
                } else {
                    strand.domains.push(Domain::new_insertion(n));
                    strand
                        .junctions
                        .insert(strand.junctions.len() - 1, DomainJunction::Adjacent);
                }
                // remove the first insertions
                strand.domains.remove(0);
                strand.junctions.remove(0);
            }

            let skip_first =
                matches!(strand.domains.first(), Some(Domain::Insertion { .. })) as usize;
            let skip_last =
                matches!(strand.domains.last(), Some(Domain::Insertion { .. })) as usize;
            let last_first_intervals = (
                strand.domains.iter().rev().nth(skip_last),
                strand.domains.get(skip_first),
            );

            if let (Some(Domain::HelixDomain(i1)), Some(Domain::HelixDomain(i2))) =
                last_first_intervals
            {
                let junction = junction(i1, i2);
                *strand.junctions.last_mut().unwrap() = junction;
            } else {
                panic!("Invariant Violated: SaneDomains")
            }
        } else {
            *strand.junctions.last_mut().unwrap() = DomainJunction::Prime3;
        }
        strand.merge_consecutive_domains();
        Ok(())
    }

    pub fn apply_cross_cut(
        state: &mut AppState,
        source_strand: usize,
        target_strand: usize,
        nucl: Nucl,
        target_3prime: bool,
    ) -> Result<(), OperationError> {
        let (design, controller) = state.design_controller_mut();
        Self::cross_cut(
            &mut design.strands,
            source_strand,
            target_strand,
            nucl,
            target_3prime,
            &mut controller.color_idx,
        )?;
        controller.state = ControllerState::Normal;
        Ok(())
    }

    pub fn apply_merge(
        state: &mut AppState,
        prime5_id: usize,
        prime3_id: usize,
    ) -> Result<(), OperationError> {
        if prime5_id != prime3_id {
            Self::merge_strands(&mut state.design_mut().strands, prime5_id, prime3_id)?;
        } else {
            Self::make_cycle(&mut state.design_mut().strands, prime5_id, true)?;
        }
        state.controller_mut().state = ControllerState::Normal;
        Ok(())
    }

    /// Cut the target strand at nucl and the make a cross over from the source strand to the part
    /// that contains nucl.
    fn cross_cut(
        strands: &mut Strands,
        source_strand: usize,
        target_strand: usize,
        nucl: Nucl,
        target_3prime: bool,
        color_idx: &mut usize,
    ) -> Result<(), OperationError> {
        let new_id = strands.keys().max().map_or(0, |n| n + 1);
        let was_cyclic = strands
            .get(&target_strand)
            .ok_or(OperationError::StrandDoesNotExist(target_strand))?
            .is_cyclic;
        Self::split_strand(strands, &nucl, Some(target_3prime), color_idx)?;

        if !was_cyclic && source_strand != target_strand {
            if target_3prime {
                // swap the position of the two half of the target strands so that the merged part is the
                // new id
                let half0 = strands
                    .remove(&target_strand)
                    .ok_or(OperationError::StrandDoesNotExist(target_strand))?;
                let half1 = strands
                    .remove(&new_id)
                    .ok_or(OperationError::StrandDoesNotExist(new_id))?;
                strands.insert(new_id, half0);
                strands.insert(target_strand, half1);
                Self::merge_strands(strands, source_strand, new_id)
            } else {
                // if the target strand is the 5' end of the merge, we give the new id to the source
                // strand because it is the one that is lost in the merge.
                let half0 = strands
                    .remove(&source_strand)
                    .ok_or(OperationError::StrandDoesNotExist(source_strand))?;
                let half1 = strands
                    .remove(&new_id)
                    .ok_or(OperationError::StrandDoesNotExist(new_id))?;
                strands.insert(new_id, half0);
                strands.insert(source_strand, half1);
                Self::merge_strands(strands, target_strand, new_id)
            }
        } else if source_strand == target_strand {
            Self::make_cycle(strands, source_strand, true)
        } else if target_3prime {
            Self::merge_strands(strands, source_strand, target_strand)
        } else {
            Self::merge_strands(strands, target_strand, source_strand)
        }
    }

    pub fn apply_general_cross_over(
        state: &mut AppState,
        source_nucl: Nucl,
        target_nucl: Nucl,
    ) -> Result<(), OperationError> {
        let (design, controller) = state.design_controller_mut();
        controller.general_cross_over(&mut design.strands, source_nucl, target_nucl)?;
        Ok(())
    }

    pub fn check_xovers(design: &mut Design, xovers: Vec<usize>) -> Result<(), OperationError> {
        let xovers_set = &mut design.checked_xovers;
        for x in xovers {
            if !xovers_set.insert(x) {
                xovers_set.remove(&x);
            }
        }
        Ok(())
    }

    fn twisted_pair(mut a1: Nucl, mut b1: Nucl, mut a2: Nucl, mut b2: Nucl) -> bool {
        if a1 > b1 {
            std::mem::swap(&mut a1, &mut b1);
        }
        if a2 > b2 {
            std::mem::swap(&mut a2, &mut b2);
        }

        (a1.prime3() == a2 && b1.prime3() == b2) || (a1.prime5() == a2 && b1.prime5() == b2)
    }

    pub fn apply_several_xovers(
        state: &mut AppState,
        mut pairs: Vec<(Nucl, Nucl)>,
        doubled: bool,
    ) -> Result<(), OperationError> {
        let (design, controller) = state.design_controller_mut();
        pairs.sort();

        for i in 0..pairs.len() {
            for j in i..pairs.len() {
                if Self::twisted_pair(pairs[i].0, pairs[i].1, pairs[j].0, pairs[j].1) {
                    let (l, r) = pairs.split_at_mut(j);
                    std::mem::swap(&mut l[i].1, &mut r[0].1);
                }
            }
        }

        pairs = if doubled {
            let mut ret = pairs.clone();
            for (a, b) in pairs {
                if !ret.iter().any(|(x, y)| {
                    *x == a.prime5() || *x == b.prime5() || *y == a.prime5() || *y == b.prime5()
                }) {
                    ret.push((a.prime3(), b.prime5()));
                }
            }
            ret.dedup();
            ret
        } else {
            pairs
        };
        for (source_nucl, target_nucl) in pairs {
            if let Err(e) =
                controller.general_cross_over(&mut design.strands, source_nucl, target_nucl)
            {
                log::error!("when making xover {source_nucl:?} {target_nucl:?} : {e:?}",);
            }
        }
        Ok(())
    }

    fn general_cross_over(
        &mut self,
        strands: &mut Strands,
        source_nucl: Nucl,
        target_nucl: Nucl,
    ) -> Result<(), OperationError> {
        log::info!("cross over between {source_nucl:?} and {target_nucl:?}");
        let source_id = strands
            .get_strand_nucl(&source_nucl)
            .ok_or(OperationError::NuclDoesNotExist(source_nucl))?;
        let target_id = strands
            .get_strand_nucl(&target_nucl)
            .ok_or(OperationError::NuclDoesNotExist(target_nucl))?;

        let source = strands
            .get(&source_id)
            .cloned()
            .ok_or(OperationError::StrandDoesNotExist(source_id))?;
        let _ = strands
            .get(&target_id)
            .cloned()
            .ok_or(OperationError::StrandDoesNotExist(target_id))?;

        let source_strand_end = strands.is_strand_end(&source_nucl);
        let target_strand_end = strands.is_strand_end(&target_nucl);
        log::info!("source strand {source_id:?}, target strand {target_id:?}",);
        log::info!(
            "source end {:?}, target end {:?}",
            source_strand_end.to_opt(),
            target_strand_end.to_opt()
        );
        match (source_strand_end.to_opt(), target_strand_end.to_opt()) {
            (Some(true), Some(true)) => return Err(OperationError::XoverBetweenTwoPrime3),
            (Some(false), Some(false)) => return Err(OperationError::XoverBetweenTwoPrime5),
            (Some(true), Some(false)) => {
                // We can xover directly
                if source_id == target_id {
                    Self::make_cycle(strands, source_id, true)?;
                } else {
                    Self::merge_strands(strands, source_id, target_id)?;
                }
            }
            (Some(false), Some(true)) => {
                // We can xover directly but we must reverse the xover
                if source_id == target_id {
                    Self::make_cycle(strands, target_id, true)?;
                } else {
                    Self::merge_strands(strands, target_id, source_id)?;
                }
            }
            (Some(b), None) => {
                let target_3prime = b;
                // NS: allow xover within the same helix
                if ALLOW_SAME_HELIX_CROSSOVERS || source_nucl.helix != target_nucl.helix {
                    Self::cross_cut(
                        strands,
                        source_id,
                        target_id,
                        target_nucl,
                        target_3prime,
                        &mut self.color_idx,
                    )?;
                }
            }
            (None, Some(b)) => {
                let target_3prime = b;
                // NS: allow xover within the same helix
                if ALLOW_SAME_HELIX_CROSSOVERS || source_nucl.helix != target_nucl.helix {
                    Self::cross_cut(
                        strands,
                        target_id,
                        source_id,
                        source_nucl,
                        target_3prime,
                        &mut self.color_idx,
                    )?;
                }
            }
            (None, None) => {
                // NS: allow xover within the same helix
                if ALLOW_SAME_HELIX_CROSSOVERS || source_nucl.helix != target_nucl.helix {
                    if source_id != target_id {
                        Self::split_strand(strands, &source_nucl, None, &mut self.color_idx)?;
                        Self::cross_cut(
                            strands,
                            source_id,
                            target_id,
                            target_nucl,
                            true,
                            &mut self.color_idx,
                        )?;
                    } else if source.is_cyclic {
                        Self::split_strand(
                            strands,
                            &source_nucl,
                            Some(false),
                            &mut self.color_idx,
                        )?;
                        Self::cross_cut(
                            strands,
                            source_id,
                            target_id,
                            target_nucl,
                            true,
                            &mut self.color_idx,
                        )?;
                    } else {
                        // if the two nucleotides are on the same strand care must be taken
                        // because one of them might be on the newly crated strand after the
                        // split
                        let pos1 = source
                            .find_nucl(&source_nucl)
                            .ok_or(OperationError::NuclDoesNotExist(source_nucl))?;
                        let pos2 = source
                            .find_nucl(&target_nucl)
                            .ok_or(OperationError::NuclDoesNotExist(target_nucl))?;
                        if pos1 > pos2 {
                            // the source nucl will be on the 5' end of the split and the
                            // target nucl as well
                            Self::split_strand(
                                strands,
                                &source_nucl,
                                Some(false),
                                &mut self.color_idx,
                            )?;
                            Self::cross_cut(
                                strands,
                                source_id,
                                target_id,
                                target_nucl,
                                true,
                                &mut self.color_idx,
                            )?;
                        } else {
                            let new_id = Self::split_strand(
                                strands,
                                &source_nucl,
                                Some(false),
                                &mut self.color_idx,
                            )?;
                            Self::cross_cut(
                                strands,
                                source_id,
                                new_id,
                                target_nucl,
                                true,
                                &mut self.color_idx,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn delete_strands(
        state: &mut AppState,
        strand_ids: Vec<usize>,
    ) -> Result<(), OperationError> {
        for s_id in &strand_ids {
            state.design_mut().strands.remove(s_id);
        }
        Ok(())
    }

    pub fn delete_helices(
        state: &mut AppState,
        helices_id: Vec<usize>,
    ) -> Result<(), OperationError> {
        for h_id in &helices_id {
            if state.design().strands.uses_helix(*h_id) {
                return Err(OperationError::HelixNotEmpty(*h_id));
            }
            state.design_mut().helices.make_mut().remove(h_id);
        }
        Ok(())
    }

    pub fn delete_free_grids(
        design: &mut Design,
        grid_ids: Vec<usize>,
    ) -> Result<(), OperationError> {
        let data = design.get_updated_grid_data();
        let empty_grids = data.get_empty_grids_id();

        let mut free_grids_mut = design.free_grids.make_mut();
        for id in grid_ids {
            let g_id = GridId::FreeGrid(id);
            if !empty_grids.contains(&g_id) {
                return Err(OperationError::GridIsNotEmpty(g_id));
            }
            free_grids_mut
                .remove(&g_id)
                .ok_or(OperationError::GridDoesNotExist(g_id))?;
        }

        Ok(())
    }

    pub fn set_grid_position(
        design: &mut Design,
        grid_id: GridId,
        position: Vec3,
    ) -> Result<(), OperationError> {
        if let GridId::FreeGrid(id) = grid_id {
            let mut new_grids = design.free_grids.make_mut();
            let grid = new_grids
                .get_mut(&FreeGridId(id))
                .ok_or(OperationError::GridDoesNotExist(grid_id))?;
            grid.position = position;
            drop(new_grids);
            Ok(())
        } else {
            log::error!("Setting position of bezier path grids is not yet implemented");
            Err(OperationError::NotImplemented)
        }
    }

    pub fn set_grid_orientation(
        design: &mut Design,
        grid_id: GridId,
        orientation: Rotor3,
    ) -> Result<(), OperationError> {
        if let GridId::FreeGrid(id) = grid_id {
            let mut new_grids = design.free_grids.make_mut();
            let grid = new_grids
                .get_mut(&FreeGridId(id))
                .ok_or(OperationError::GridDoesNotExist(grid_id))?;
            grid.orientation = orientation;
            drop(new_grids);
            Ok(())
        } else {
            log::error!("Setting orientation of bezier path grids is not yet implemented");
            Err(OperationError::NotImplemented)
        }
    }

    pub fn set_grid_nb_turn(
        design: &mut Design,
        grid_id: GridId,
        x: f64,
    ) -> Result<(), OperationError> {
        if let GridId::FreeGrid(id) = grid_id {
            let mut new_grids = design.free_grids.make_mut();
            let grid = new_grids
                .get_mut(&FreeGridId(id))
                .ok_or(OperationError::GridDoesNotExist(grid_id))?;
            if let GridTypeDescr::Hyperboloid {
                nb_turn_per_100_nt, ..
            } = &mut grid.grid_type
            {
                *nb_turn_per_100_nt = x;
            } else {
                return Err(OperationError::GridIsNotHyperboloid(grid_id));
            }
            drop(new_grids);
            Ok(())
        } else {
            log::error!("Setting nb turn of bezier path grids is not yet implemented");
            Err(OperationError::NotImplemented)
        }
    }

    pub fn add_3d_object(
        design: &mut Design,
        object_path: PathBuf,
        design_path: PathBuf,
    ) -> Result<(), OperationError> {
        let object = External3DObject::new(External3DObjectDescriptor {
            object_path,
            design_path,
        })
        .ok_or(OperationError::CouldNotMake3DObject)?;

        design.external_3d_objects.add_object(object);

        Ok(())
    }

    pub fn import_svg_path(design: &mut Design, path: PathBuf) -> Result<(), OperationError> {
        // The imported bezier path will be attached to plane 0 so we need to ensure that it exists
        if design.bezier_planes.get(&BezierPlaneId(0)).is_none() {
            Self::add_bezier_plane(design, Default::default());
        }

        let mut paths = design.bezier_paths.make_mut();
        let path = read_first_svg_path(&path)?;
        paths.push(path);

        drop(paths);

        Ok(())
    }
}

// Some values are only used for logging the error, which Rust considers to be unused.
#[derive(Debug)]
pub enum OperationError {
    GroupHasNoPivot(GroupId),
    NotImplemented,
    /// The operation cannot be applied on the current selection.
    BadSelection,
    /// The controller is in a state incompatible with applying the operation.
    IncompatibleState(String),
    CannotBuildOn(Nucl),
    CutNonExistentStrand,
    GridDoesNotExist(GridId),
    GridPositionAlreadyUsed,
    StrandDoesNotExist(usize),
    HelixDoesNotExists(usize),
    HelixHasNoGridPosition(usize),
    CouldNotMakeEdge(HelixGridPosition, HelixGridPosition),
    MergingSameStrand,
    NuclDoesNotExist(Nucl),
    XoverBetweenTwoPrime5,
    XoverBetweenTwoPrime3,
    CouldNotCreateEdges,
    EmptyOrigin,
    EmptyClipboard,
    WrongClipboard,
    CannotPasteHere,
    HelixNotEmpty(usize),
    EmptyScaffoldSequence,
    NoScaffoldSet,
    NoGrids,
    FinishFirst,
    CameraDoesNotExist(CameraId),
    GridIsNotHyperboloid(GridId),
    DesignOperationError(DesignOperationError),
    NotPiecewiseBezier(usize),
    GridCopyError(GridCopyError),
    CouldNotGetPrime3of(usize),
    PathDoesNotExist(BezierPathId),
    VertexDoesNotExist(BezierPathId, usize),
    GridIsNotEmpty(GridId),
    CouldNotMake3DObject,
    SvgImportError(SvgImportError),
}

impl From<DesignOperationError> for OperationError {
    fn from(e: DesignOperationError) -> Self {
        Self::DesignOperationError(e)
    }
}

impl From<SvgImportError> for OperationError {
    fn from(e: SvgImportError) -> Self {
        Self::SvgImportError(e)
    }
}

fn nucl_pos_2d(helices: &Helices, nucl: &Nucl, segment: usize) -> Option<Vec2> {
    let isometry = helices.get(&nucl.helix).and_then(|h| {
        if segment > 0 {
            h.additional_isometries
                .get(segment - 1)
                .and_then(|i| i.additional_isometry.or(h.isometry2d))
        } else {
            h.isometry2d
        }
    });
    let local_position = nucl.position as f32 * Vec2::unit_x()
        + if nucl.forward {
            Vec2::zero()
        } else {
            Vec2::unit_y()
        };

    isometry.map(|i| i.into_homogeneous_matrix().transform_point2(local_position))
}

#[derive(Clone, Default)]
// TODO : remove all design pointers from this enum
enum ControllerState {
    #[default]
    Normal,
    MakingHyperboloid {
        initial_design: AddressPointer<Design>,
        position: Vec3,
        orientation: Rotor3,
    },
    BuildingStrand {
        builders: Vec<StrandBuilder>,
        initial_design: AddressPointer<Design>,
        initializing: bool,
        ignored_domains: Vec<DomainIdentifier>,
    },
    ChangingColor,
    SettingRollHelices,
    WithPendingOp {
        operation: Arc<dyn SimpleOperation>,
    },
    ApplyingOperation {
        operation: Option<Arc<dyn SimpleOperation>>,
    },
    PositioningStrandPastingPoint {
        pasting_point: Option<Nucl>,
        pasted_strands: Vec<PastedStrand>,
    },
    PositioningStrandDuplicationPoint {
        pasting_point: Option<Nucl>,
        pasted_strands: Vec<PastedStrand>,
        duplication_edge: Option<DuplicationEdge>,
        clipboard: StrandClipboard,
    },
    PositioningHelicesPastingPoint {
        pasting_point: Option<GridPosition>,
        initial_design: AddressPointer<Design>,
    },
    PositioningHelicesDuplicationPoint {
        pasting_point: Option<GridPosition>,
        initial_design: AddressPointer<Design>,
        duplication_edge: Option<Edge>,
        helices: Vec<usize>,
    },
    WithPendingHelicesDuplication {
        last_pasting_point: GridPosition,
        duplication_edge: Edge,
        helices: Vec<usize>,
    },
    WithPendingStrandDuplication {
        last_pasting_point: Nucl,
        duplication_edge: DuplicationEdge,
        clipboard: StrandClipboard,
    },
    WithPendingXoverDuplication {
        last_pasting_point: Nucl,
        duplication_edge: DuplicationEdge,
        xovers: Vec<(Nucl, Nucl)>,
    },
    PastingXovers {
        initial_design: AddressPointer<Design>,
        pasting_point: Option<Nucl>,
    },
    DoingFirstXoversDuplication {
        initial_design: AddressPointer<Design>,
        duplication_edge: Option<DuplicationEdge>,
        xovers: Vec<(Nucl, Nucl)>,
        pasting_point: Option<Nucl>,
    },
    OptimizingScaffoldPosition,
    Simulating {
        interface: Arc<Mutex<HelixSystemInterface>>,
        initial_design: AddressPointer<Design>,
    },
    RapierSimulating {
        interface: Arc<Mutex<RapierInterface>>,
        initial_design: AddressPointer<Design>,
    },
    SimulatingGrids {
        interface: Arc<Mutex<GridSystemInterface>>,
        _initial_design: AddressPointer<Design>,
    },
    Relaxing {
        interface: Arc<Mutex<RevolutionSystemInterface>>,
        _initial_design: AddressPointer<Design>,
    },
    WithPausedSimulation {
        initial_design: AddressPointer<Design>,
    },
    Rolling {
        _interface: Arc<Mutex<RollInterface>>,
        _initial_design: AddressPointer<Design>,
    },
    Twisting {
        _interface: Arc<Mutex<TwistInterface>>,
        _initial_design: AddressPointer<Design>,
        grid_id: GridId,
    },
    ChangingStrandName {
        strand_id: usize,
    },
}

impl std::fmt::Debug for ControllerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::MakingHyperboloid { .. } => write!(f, "MakingHyperboloid"),
            Self::BuildingStrand { .. } => write!(f, "BuildingStrand"),
            Self::ChangingColor => write!(f, "ChangingColor"),
            Self::SettingRollHelices => write!(f, "SettingRollHelices"),
            Self::WithPendingOp { .. } => write!(f, "WithPendingOp"),
            Self::ApplyingOperation { .. } => write!(f, "ApplyingOperation"),
            Self::PositioningStrandPastingPoint { .. } => {
                write!(f, "PositioningStrandPastingPoint")
            }
            Self::PositioningStrandDuplicationPoint { .. } => {
                write!(f, "PositioningStrandDuplicationPoint")
            }
            Self::PositioningHelicesPastingPoint { .. } => {
                write!(f, "PositioningHelicesPastingPoint")
            }
            Self::PositioningHelicesDuplicationPoint { .. } => {
                write!(f, "PositioningHelicesDuplicationPoint")
            }
            Self::WithPendingHelicesDuplication { .. } => {
                write!(f, "WithPendingHelicesDuplication")
            }
            Self::WithPendingStrandDuplication { .. } => {
                write!(f, "WithPendingStrandDuplication")
            }
            Self::WithPendingXoverDuplication { .. } => {
                write!(f, "WithPendingXoverDuplication")
            }
            Self::PastingXovers { .. } => write!(f, "PastingXovers"),
            Self::DoingFirstXoversDuplication { .. } => {
                write!(f, "DoingFirstXoversDuplication")
            }
            Self::OptimizingScaffoldPosition => write!(f, "OptimizingScaffoldPosition"),
            Self::Simulating { .. } => write!(f, "Simulating"),
            Self::RapierSimulating { .. } => write!(f, "RapierSimulating"),
            Self::SimulatingGrids { .. } => write!(f, "SimulatingGrids"),
            Self::Relaxing { .. } => write!(f, "Relaxing"),
            Self::WithPausedSimulation { .. } => write!(f, "WithPausedSimulation"),
            Self::Rolling { .. } => write!(f, "Rolling"),
            Self::Twisting { .. } => write!(f, "Twisting"),
            Self::ChangingStrandName { .. } => write!(f, "ChangingStrandName"),
        }
    }
}

impl ControllerState {
    fn state_name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::MakingHyperboloid { .. } => "MakingHyperboloid",
            Self::BuildingStrand { .. } => "BuildingStrand",
            Self::ChangingColor => "ChangingColor",
            Self::WithPendingOp { .. } => "WithPendingOp",
            Self::ApplyingOperation { .. } => "ApplyingOperation",
            Self::PositioningStrandPastingPoint { .. } => "PositioningStrandPastingPoint",
            Self::PositioningStrandDuplicationPoint { .. } => "PositioningStrandDuplicationPoint",
            Self::WithPendingStrandDuplication { .. } => "WithPendingDuplication",
            Self::WithPendingXoverDuplication { .. } => "WithPendingXoverDuplication",
            Self::PastingXovers { .. } => "PastingXovers",
            Self::DoingFirstXoversDuplication { .. } => "DoingFirstXoversDuplication",
            Self::OptimizingScaffoldPosition => "OptimizingScaffoldPosition",
            Self::Simulating { .. } => "Simulation",
            Self::RapierSimulating { .. } => "Rapier Simulation",
            Self::SimulatingGrids { .. } => "Simulating Grids",
            Self::WithPausedSimulation { .. } => "WithPausedSimulation",
            Self::Rolling { .. } => "Rolling",
            Self::SettingRollHelices => "SettingRollHelices",
            Self::ChangingStrandName { .. } => "ChangingStrandName",
            Self::Twisting { .. } => "Twisting",
            Self::PositioningHelicesPastingPoint { .. } => "Positioning strand pasting point",
            Self::WithPendingHelicesDuplication { .. } => "With pending helices duplication",
            Self::PositioningHelicesDuplicationPoint { .. } => {
                "Positioning helices duplication point"
            }
            Self::Relaxing { .. } => "Relaxing revolution surface",
        }
    }
    fn update_pasting_position(
        &mut self,
        point: Option<PastePosition>,
        strands: Vec<PastedStrand>,
        duplication_edge: Option<DuplicationEdge>,
    ) -> Result<(), OperationError> {
        match self {
            Self::PositioningStrandPastingPoint { .. }
            | Self::Normal
            | Self::WithPendingHelicesDuplication { .. }
            | Self::WithPendingXoverDuplication { .. }
            | Self::WithPendingOp { .. } => {
                *self = Self::PositioningStrandPastingPoint {
                    pasting_point: point.and_then(PastePosition::to_nucl),
                    pasted_strands: strands,
                };
                Ok(())
            }
            Self::PositioningStrandDuplicationPoint { clipboard, .. } => {
                *self = Self::PositioningStrandDuplicationPoint {
                    pasting_point: point.and_then(PastePosition::to_nucl),
                    pasted_strands: strands,
                    duplication_edge,
                    clipboard: clipboard.clone(),
                };
                Ok(())
            }
            _ => Err(OperationError::IncompatibleState(
                self.state_name().to_owned(),
            )),
        }
    }

    fn update_helices_pasting_position(
        &mut self,
        position: Option<PastePosition>,
        edge: Option<Edge>,
        design: &Design,
    ) -> Result<(), OperationError> {
        match self {
            Self::PositioningHelicesPastingPoint { pasting_point, .. } => {
                *pasting_point = position.and_then(PastePosition::to_grid_position);
                Ok(())
            }
            Self::PositioningHelicesDuplicationPoint {
                pasting_point,
                duplication_edge,
                ..
            } => {
                *pasting_point = position.and_then(PastePosition::to_grid_position);
                *duplication_edge = edge;
                Ok(())
            }
            Self::Normal
            | Self::WithPendingOp { .. }
            | Self::WithPendingStrandDuplication { .. }
            | Self::WithPendingXoverDuplication { .. }
            | Self::WithPendingHelicesDuplication { .. } => {
                *self = Self::PositioningHelicesPastingPoint {
                    pasting_point: position.and_then(PastePosition::to_grid_position),
                    initial_design: AddressPointer::new(design.clone()),
                };
                Ok(())
            }
            _ => Err(OperationError::IncompatibleState(
                self.state_name().to_owned(),
            )),
        }
    }

    fn update_xover_pasting_position(
        &mut self,
        point: Option<Nucl>,
        edge: Option<DuplicationEdge>,
        design: &Design,
    ) -> Result<(), OperationError> {
        match self {
            Self::PastingXovers { pasting_point, .. } => {
                *pasting_point = point;
                Ok(())
            }
            Self::DoingFirstXoversDuplication {
                pasting_point,
                duplication_edge,
                ..
            } => {
                *pasting_point = point;
                *duplication_edge = edge;
                Ok(())
            }
            Self::Normal
            | Self::WithPendingOp { .. }
            | Self::WithPendingHelicesDuplication { .. }
            | Self::WithPendingStrandDuplication { .. } => {
                *self = Self::PastingXovers {
                    pasting_point: point,
                    initial_design: AddressPointer::new(design.clone()),
                };
                Ok(())
            }
            _ => Err(OperationError::IncompatibleState(
                self.state_name().to_owned(),
            )),
        }
    }

    fn update_operation(&mut self, op: Arc<dyn SimpleOperation>) {
        match self {
            Self::ApplyingOperation { operation, .. } => *operation = Some(op),
            Self::WithPendingOp { operation, .. } => *operation = op,
            _ => (),
        }
    }

    fn finish(&mut self) {
        let value = self.clone();
        match value {
            Self::ApplyingOperation {
                operation: Some(op),
                ..
            } => {
                *self = Self::WithPendingOp { operation: op };
            }
            Self::MakingHyperboloid { .. }
            | Self::WithPendingOp { .. }
            | Self::PositioningStrandPastingPoint { .. }
            | Self::PositioningStrandDuplicationPoint { .. }
            | Self::WithPendingStrandDuplication { .. }
            | Self::WithPendingXoverDuplication { .. }
            | Self::PastingXovers { .. }
            | Self::DoingFirstXoversDuplication { .. }
            | Self::OptimizingScaffoldPosition
            | Self::Simulating { .. }
            | Self::RapierSimulating { .. }
            | Self::SimulatingGrids { .. }
            | Self::Relaxing { .. }
            | Self::PositioningHelicesPastingPoint { .. }
            | Self::PositioningHelicesDuplicationPoint { .. }
            | Self::WithPendingHelicesDuplication { .. } => (),
            Self::Normal
            | Self::BuildingStrand { .. }
            | Self::ChangingColor
            | Self::ApplyingOperation { .. }
            | Self::WithPausedSimulation { .. }
            | Self::Rolling { .. }
            | Self::SettingRollHelices
            | Self::Twisting { .. }
            | Self::ChangingStrandName { .. } => *self = Self::Normal,
        }
    }

    fn acknowledge_new_selection(&mut self) {
        let value = self.clone();
        if matches!(
            value,
            Self::WithPendingStrandDuplication { .. }
                | Self::WithPendingXoverDuplication { .. }
                | Self::WithPendingHelicesDuplication { .. }
        ) {
            *self = Self::Normal;
        }
    }

    // /// Return true if the operation is undoable only when going from this state to normal
    // fn is_undoable_once(&self) -> bool {
    //     matches!(
    //         self,
    //         Self::PositioningStrandDuplicationPoint { .. }
    //             | Self::PositioningStrandPastingPoint { .. }
    //     )
    // }
}

#[derive(Copy, Clone)]
pub enum InteractorNotification {
    FinishOperation,
    NewSelection,
}

/// Return the appropriate junction between two HelixInterval.
pub fn junction(prime5: &HelixInterval, prime3: &HelixInterval) -> DomainJunction {
    let prime5_nucl = prime5.prime3();
    let prime3_nucl = prime3.prime5();

    if prime3_nucl == prime5_nucl.prime3() {
        DomainJunction::Adjacent
    } else {
        DomainJunction::UnidentifiedXover
    }
}

enum OperationCompatibility {
    Compatible,
    Incompatible,
    FinishFirst,
}

pub enum StatePersistence {
    Persistent,
    NeedFinish,
    Transitory,
}

impl StatePersistence {
    pub fn is_persistent(&self) -> bool {
        matches!(self, Self::Persistent)
    }

    pub fn is_transitory(&self) -> bool {
        matches!(self, Self::Transitory)
    }
}
