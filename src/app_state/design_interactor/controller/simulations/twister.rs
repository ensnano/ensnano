use crate::app_state::design_interactor::controller::simulations::SimulationInterface;
use crate::app_state::design_interactor::controller::simulations::roller::{
    DesignData, RollSystem,
};
use crate::ensnano_design::Design;
use crate::ensnano_design::curves::CurveDescriptor;
use crate::ensnano_design::curves::twist::{Twist, nb_turn_per_100_nt_to_omega, twist_to_omega};
use crate::ensnano_design::grid::grid_collection::FreeGridId;
use crate::ensnano_design::grid::{GridDescriptor, GridId, GridTypeDescr};
use crate::ensnano_design::helices::Helix;
use crate::ensnano_design::parameters::HelixParameters;
use crate::ensnano_design::utils::vec_to_dvec;
use crate::{
    app_state::design_interactor::{Presenter, presenter::SimulationUpdate},
    controller::channel_reader::ChannelReader,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

const NB_ROLL_STEP_PER_TWIST: usize = 500;
const MIN_OMEGA: f64 = -0.2;
const MAX_OMEGA: f64 = 0.2;
const NB_STEP_OMEGA: usize = 300;

struct TwistSystem {
    current_omega: f64,
    best_omega: f64,
    best_square_error: f64,
}

pub struct Twister {
    data: DesignData,
    system: TwistSystem,
    interface: Weak<Mutex<TwistInterface>>,
    state: TwistState,
}

impl Twister {
    pub fn start_new(
        presenter: &Presenter,
        target_grid: GridId,
        reader: &mut ChannelReader,
    ) -> Option<Arc<Mutex<TwistInterface>>> {
        let mut helices: Vec<Helix> = Vec::new();
        let mut keys: Vec<usize> = Vec::new();
        for (key, helix) in presenter.get_helices().iter().filter(|(_, h)| {
            h.grid_position
                .filter(|pos| pos.grid == target_grid)
                .is_some()
        }) {
            keys.push(*key);
            helices.push(helix.clone());
        }
        let helix_parameters = presenter.get_design().helix_parameters.unwrap_or_default();
        let mut xovers = presenter.get_xovers_list();
        xovers.retain(|(n1, n2)| keys.contains(&n1.helix) && keys.contains(&n2.helix));
        let mut helix_map = HashMap::new();
        for (n, k) in keys.iter().enumerate() {
            helix_map.insert(*k, n);
        }
        let system = TwistSystem {
            current_omega: MIN_OMEGA,
            best_omega: MIN_OMEGA,
            best_square_error: f64::INFINITY,
        };

        let data = DesignData {
            helices,
            helix_map,
            xovers,
            helix_parameters,
        };

        let interface = Arc::new(Mutex::new(TwistInterface::default()));
        let interface_dyn: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
        reader.attach_state(&interface_dyn);

        let initial_state = if let Some(grid) = FreeGridId::try_from_grid_id(target_grid)
            .and_then(|target_grid| presenter.get_design().free_grids.get(&target_grid))
        {
            TwistState {
                grid_id: target_grid,
                grid: *grid,
                helices: presenter
                    .get_design()
                    .helices
                    .iter()
                    .map(|(k, h)| (*k, h.clone()))
                    .collect(),
            }
        } else {
            log::error!("Could not get grid {target_grid:?}");
            return None;
        };

        let twister = Self {
            data,
            system,
            state: initial_state,
            interface: Arc::downgrade(&interface),
        };

        twister.run();
        Some(interface)
    }

    fn evaluate_twist(&mut self, twist: f64) -> f64 {
        self.data.update_twist(twist);
        let mut roll_system = RollSystem::new(self.data.helices.len(), None, &self.data.helix_map);
        for _ in 0..NB_ROLL_STEP_PER_TWIST {
            roll_system.solve_one_step(&mut self.data, 1e-3);
        }
        self.data.square_xover_constraints()
    }

    fn solve_one_step(&mut self) {
        let err = self.evaluate_twist(self.system.current_omega);
        println!("err = {err}");
        if err < self.system.best_square_error {
            println!("best omega = {}", self.system.current_omega);
            self.system.best_square_error = err;
            self.system.best_omega = self.system.current_omega;
            self.state
                .set_twist(self.system.best_omega, &self.data.helix_parameters);
        }
        self.system.current_omega += (MAX_OMEGA - MIN_OMEGA) / (NB_STEP_OMEGA as f64);
        println!("current_omega = {}", self.system.current_omega);
    }

    pub fn run(mut self) {
        std::thread::spawn(move || {
            while let Some(interface_ptr) = self.interface.upgrade() {
                self.solve_one_step();
                interface_ptr.lock().unwrap().stabilized = self.system.current_omega >= MAX_OMEGA;
                interface_ptr.lock().unwrap().new_state = Some(self.state.clone());
            }
        });
    }
}

#[derive(Clone)]
pub struct TwistState {
    grid_id: GridId,
    helices: HashMap<usize, Helix>,
    grid: GridDescriptor,
}

impl TwistState {
    fn set_twist(&mut self, twist: f64, helix_parameters: &HelixParameters) {
        let omega = match &mut self.grid.grid_type {
            GridTypeDescr::Hyperboloid {
                nb_turn_per_100_nt, ..
            } => {
                *nb_turn_per_100_nt = twist;
                nb_turn_per_100_nt_to_omega(*nb_turn_per_100_nt, helix_parameters)
            }
            GridTypeDescr::Square { twist: grid_twist }
            | GridTypeDescr::Honeycomb { twist: grid_twist } => {
                *grid_twist = Some(twist);
                twist_to_omega(twist, helix_parameters)
            }
        };

        if let Some(new_omega) = omega {
            for h in self.helices.values_mut() {
                if let Some(CurveDescriptor::Twist(Twist { omega, .. })) =
                    h.curve.as_mut().map(Arc::make_mut)
                {
                    *omega = new_omega;
                    // no need to update the curve because the helices here are not used to make
                    // computations
                } else {
                    log::error!("Wrong kind of curve descriptor");
                }
            }
        }
    }
}

impl SimulationUpdate for TwistState {
    fn update_design(&self, design: &mut Design) {
        let mut new_helices = design.helices.make_mut();
        for (i, h) in &self.helices {
            new_helices.insert(*i, h.clone());
        }

        let mut grids_mut = design.free_grids.make_mut();
        if let Some(grid) =
            FreeGridId::try_from_grid_id(self.grid_id).and_then(|g_id| grids_mut.get_mut(&g_id))
        {
            *grid = self.grid;
        } else {
            log::error!("COULD NOT UPDATE GRID {:?}", self.grid_id);
        }
    }
}

#[derive(Default)]
pub struct TwistInterface {
    pub new_state: Option<TwistState>,
    stabilized: bool,
}

impl SimulationInterface for TwistInterface {
    fn get_simulation_state(&mut self) -> Option<Box<dyn SimulationUpdate>> {
        let s = self.new_state.take()?;
        Some(Box::new(s))
    }

    fn still_valid(&self) -> bool {
        !self.stabilized
    }
}

impl DesignData {
    fn square_xover_constraints(&self) -> f64 {
        let mut ret = 0.0;
        let len_0 = super::roller::dist_ac(&self.helix_parameters) as f64;
        for (n1, n2) in &self.xovers {
            let hid_1 = &self.helix_map[&n1.helix];
            let hid_2 = &self.helix_map[&n2.helix];
            let helix_1 = &self.helices[*hid_1];
            let helix_2 = &self.helices[*hid_2];

            if self.support_helix_idx(helix_1).unwrap_or(*hid_1)
                != self.support_helix_idx(helix_2).unwrap_or(*hid_2)
            {
                let pos_1 =
                    vec_to_dvec(helix_1.space_pos(&self.helix_parameters, n1.position, n1.forward));
                let pos_2 =
                    vec_to_dvec(helix_2.space_pos(&self.helix_parameters, n2.position, n2.forward));

                let len = (pos_1 - pos_2).mag();

                ret += (len - len_0) * (len - len_0);
            }
        }
        ret
    }

    fn support_helix_idx(&self, helix: &Helix) -> Option<usize> {
        helix
            .support_helix
            .as_ref()
            .and_then(|h_id| self.helix_map.get(h_id))
            .copied()
    }

    fn update_twist(&mut self, twist: f64) {
        for h in &mut self.helices {
            if let Some(CurveDescriptor::Twist(Twist { omega, .. })) =
                h.curve.as_mut().map(Arc::make_mut)
            {
                *omega =
                    nb_turn_per_100_nt_to_omega(twist, &self.helix_parameters).unwrap_or(*omega);
                h.try_update_curve(&self.helix_parameters);
            } else {
                log::error!("Update twist: Wrong kind of curve descriptor");
            }
        }
    }
}
