use crate::app_state::design_interactor::{
    controller::simulations::SimulationInterface,
    presenter::{Presenter, SimulationUpdate},
};
use ahash::RandomState;
use ensnano_design::{Design, helices::NuclCollection, parameters::HelixParameters};
use ensnano_physics::{parameters::RapierParameters, simulation::RapierPhysicsSystem};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

#[derive(Default)]
pub(crate) struct RapierPhysicalSystem {
    system: RapierPhysicsSystem,
    interface: Weak<Mutex<RapierInterface>>,
}

impl RapierPhysicalSystem {
    pub(crate) fn start_new(
        presenter: &Presenter,
        parameters: RapierParameters,
    ) -> Arc<Mutex<RapierInterface>> {
        let system = RapierPhysicsSystem::full_simulation(
            presenter
                .get_design()
                .helix_parameters
                .unwrap_or(HelixParameters::GEARY_2014_DNA_P_STICK),
            parameters,
            presenter.content.nucl_collection.as_ref(),
            &presenter.content.elements,
            &presenter.content.nucleotide,
            &presenter.content.space_position,
            &presenter.get_design().helices,
            &presenter.content.is_clone_map,
        );

        let interface = Arc::new(Mutex::new(RapierInterface {
            space_position: system.get_positions(),
            force_stop: false,
            parameters,
        }));

        let result = Self {
            system,
            interface: Arc::downgrade(&interface),
        };

        result.run();
        interface
    }

    pub(crate) fn run(mut self) {
        std::thread::spawn(move || {
            while let Some(interface) = self.interface.upgrade() {
                let Ok(parameters) = interface.try_lock().map(|i| i.parameters) else {
                    continue;
                };
                // we update the physics
                self.system.step(&parameters);
                // we get the positions
                if let Ok(mut guard) = interface.try_lock() {
                    guard.space_position = self.system.get_positions();
                }
            }
        });
    }
}

#[derive(Default, Clone)]
pub(crate) struct RapierInterface {
    pub parameters: RapierParameters,
    space_position: Vec<(u32, [f32; 3])>,
    pub force_stop: bool,
}

impl RapierInterface {
    pub(crate) fn kill(&mut self) {
        self.force_stop = true;
    }
}

impl SimulationInterface for RapierInterface {
    fn get_simulation_state(&mut self) -> Option<Box<dyn SimulationUpdate>> {
        Some(Box::new(self.clone()))
    }

    fn still_valid(&self) -> bool {
        !self.force_stop
    }
}

impl SimulationUpdate for RapierInterface {
    fn update_design(&self, _: &mut Design) {
        // No operations are done here
    }

    fn update_positions(
        &self,
        _identifier_nucl: &NuclCollection,
        space_position: &mut HashMap<u32, [f32; 3], RandomState>,
    ) {
        // we extract the physical positions here
        space_position.extend(self.space_position.iter().map(|(a, b)| (a, b)));
    }
}
