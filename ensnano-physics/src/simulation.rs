use crate::{
    full_simulation::{
        CutHelicesSetup, FullSimulationSetup, KCutHelicesSetup, RigidHelicesSetup, build_simulation,
    },
    helices::build_helices,
    parameters::RapierParameters,
};
use ahash::HashMap;
use ensnano_design::{
    Nucl,
    elements::DesignElement,
    helices::{Helices, HelixCollection, NuclCollection},
    parameters::HelixParameters,
};
use rapier3d::{na::Vector3, prelude::*};

#[derive(Default)]
pub struct RapierPhysicsSystem {
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_join_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,

    pub rapier_parameters: RapierParameters,

    pub crossovers: Vec<(ColliderHandle, ColliderHandle)>,
    pub nucleotide_body_map: HashMap<u32, ColliderHandle>,
}

impl RapierPhysicsSystem {
    pub fn full_simulation(
        parameters: HelixParameters,
        rapier_parameters: RapierParameters,
        nucl_collection: &NuclCollection,
        elements: &Vec<DesignElement>,
        nucleotide: &HashMap<u32, Nucl>,
        space_position: &HashMap<u32, [f32; 3]>,
        helices: &Helices,
    ) -> Self {
        let intermediary = build_helices(elements, nucleotide);

        match rapier_parameters.simulation_type {
            crate::parameters::RapierSimulationType::Full => build_simulation(
                FullSimulationSetup,
                &intermediary,
                nucl_collection,
                elements,
                space_position,
                helices,
                &parameters,
                &rapier_parameters,
            ),
            crate::parameters::RapierSimulationType::Rigid => build_simulation(
                RigidHelicesSetup,
                &intermediary,
                nucl_collection,
                elements,
                space_position,
                helices,
                &parameters,
                &rapier_parameters,
            ),
            crate::parameters::RapierSimulationType::Cut => build_simulation(
                CutHelicesSetup,
                &intermediary,
                nucl_collection,
                elements,
                space_position,
                helices,
                &parameters,
                &rapier_parameters,
            ),
            crate::parameters::RapierSimulationType::KCut => build_simulation(
                KCutHelicesSetup,
                &intermediary,
                nucl_collection,
                elements,
                space_position,
                helices,
                &parameters,
                &rapier_parameters,
            ),
        }
    }

    pub fn step(&mut self) {
        self.repulsion_step(1.0 / 24.0);

        self.physics_pipeline.step(
            &Vector3::new(0.0, 0.0, 0.0),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_join_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );

        // let mut sum = 0.0;
        // let mut count = 0;
        // for (a, b) in &self.crossovers {
        //     let Some(a) = self.collider_set.get(*a) else {
        //         continue;
        //     };
        //     let Some(b) = self.collider_set.get(*b) else {
        //         continue;
        //     };

        //     sum += a.translation().metric_distance(&b.translation());

        //     count += 1;
        // }

        // let average = sum / count as f32;

        // println!("Average : {average}");
    }

    pub fn get_positions(&self) -> Vec<(u32, [f32; 3])> {
        let mut result = vec![];

        for (index, handle) in &self.nucleotide_body_map {
            let position = self
                .collider_set
                .get(*handle)
                .expect("Couldn't get the collider of a nucleotide")
                .position()
                .translation;

            result.push((*index, [position.x, position.y, position.z]));
        }

        result
    }
}
