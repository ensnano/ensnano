//! This module defines important structures that hold the simulation for the rest of the program.

use crate::{
    helices::build_helices,
    parameters::RapierParameters,
    setup::{
        CutHelicesSetup, FullSimulationSetup, KCutHelicesSetup, RigidHelicesSetup, build_simulation,
    },
};
use ahash::HashMap;
use ensnano_design::{
    elements::DesignElement,
    helices::{Helices, NuclCollection},
    nucl::Nucl,
    parameters::HelixParameters,
};
use rapier3d::{na::Vector3, prelude::*};

/// This structures holds all the data necessary for the simulation.
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
        is_clone_map: &HashMap<u32, bool>,
    ) -> Self {
        let intermediary = build_helices(elements, nucleotide, is_clone_map);

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

        self.brownian_motion_step(1.0 / 24.0);

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
