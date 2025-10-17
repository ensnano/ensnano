use super::import::*;
use ensnano_design::Nucl;
use crate::ensnano_interactor::ObjectType;
use ahash::HashMap;
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

    pub nucleotide_body_map: HashMap<u32, ColliderHandle>,
}

impl RapierPhysicsSystem {
    pub fn new(
        // these parameters are all part of DesignContent
        object_type: &HashMap<u32, ObjectType>,
        nucleotide: &HashMap<u32, Nucl>,
        space_position: &HashMap<u32, [f32; 3]>,
    ) -> Self {
        let mut rigid_body_set: RigidBodySet = Default::default();
        let mut collider_set: ColliderSet = Default::default();
        let mut impulse_joint_set: ImpulseJointSet = Default::default();

        let mut nucleotide_body_map: HashMap<u32, ColliderHandle> = Default::default();

        let handles = generate_intermediary_representation(nucleotide)
            .into_iter()
            .map(|v| {
                v.into_iter()
                    .map(|b| {
                        b.into_rigid_body(
                            space_position,
                            &mut rigid_body_set,
                            &mut collider_set,
                            &mut nucleotide_body_map,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // TODO for now we only do direct connections.
        // In the future, we want to check for continuous double helix
        // strands portions and link them more tightly.
        for helix in &handles {
            for link_size in [2, 3, 4, 8] {
                for window in helix.windows(link_size) {
                    generate_springs(
                        window[0],
                        window[1],
                        &mut rigid_body_set,
                        &mut collider_set,
                        &mut impulse_joint_set,
                    );
                }
            }
        }

        // we add crossover springs
        add_crossover_springs(
            object_type,
            nucleotide,
            &nucleotide_body_map,
            &collider_set,
            &mut impulse_joint_set,
        );

        Self {
            rigid_body_set,
            collider_set,
            impulse_joint_set,
            nucleotide_body_map,
            ..Default::default()
        }
    }

    pub fn step(&mut self) {
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
