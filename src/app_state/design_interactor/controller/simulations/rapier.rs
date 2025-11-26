use crate::ensnano_design::Design;
use crate::ensnano_physics::simulation::RapierPhysicsSystem;
use crate::{
    app_state::design_interactor::{
        controller::simulations::SimulationInterface,
        presenter::{Presenter, SimulationUpdate},
    },
    controller::channel_reader::ChannelReader,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

#[derive(Default)]
pub struct RapierPhysicalSystem {
    system: RapierPhysicsSystem,
    interface: Weak<Mutex<RapierInterface>>,
}

impl RapierPhysicalSystem {
    pub fn start_new(
        presenter: &Presenter,
        reader: &mut ChannelReader,
    ) -> Arc<Mutex<RapierInterface>> {
        // first prototype simulation
        // let system = RapierPhysicsSystem::new(
        //     &presenter.content.object_type,
        //     &presenter.content.nucleotide,
        //     &presenter.content.space_position,
        //     &presenter.content.helix_map,
        //     &presenter.get_design().helices,
        // );

        let system = RapierPhysicsSystem::full_simulation(
            presenter
                .get_design()
                .helix_parameters
                .unwrap_or(HelixParameters::GEARY_2014_DNA_P_STICK),
            &presenter.content.object_type,
            &presenter.content.nucleotide,
            &presenter.content.space_position,
            &presenter.get_design().helices,
        );

        let interface = Arc::new(Mutex::new(RapierInterface {
            space_position: system.get_positions(),
            force_stop: false,
        }));

        let result = Self {
            system,
            interface: Arc::downgrade(&interface),
        };

        let interface_dyn: Arc<Mutex<dyn SimulationInterface>> = interface.clone();
        reader.attach_state(&interface_dyn);

        result.run();
        interface
    }

    pub fn run(mut self) {
        std::thread::spawn(move || {
            while let Some(interface) = self.interface.upgrade() {
                // we update the physics
                self.system.step();
                // we get the positions
                interface.lock().unwrap().space_position = self.system.get_positions();

                // DEBUG : slowing down the simulation
                std::thread::sleep(Duration::from_secs_f32(0.1));
            }
        });
    }
}

#[derive(Default, Clone)]
pub struct RapierInterface {
    space_position: Vec<(u32, [f32; 3])>,
    pub force_stop: bool,
}

impl RapierInterface {
    pub fn kill(&mut self) {
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
        space_position: &mut HashMap<u32, [f32; 3], ahash::RandomState>,
    ) {
        // we extract the physical positions here
        space_position.extend(self.space_position.iter().map(|(a, b)| (a, b)));
    }
}
