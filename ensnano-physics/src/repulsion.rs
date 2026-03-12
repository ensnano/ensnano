//! This module defines the repulsion force which is an important part of the simulation.

use crate::{parameters::RapierParameters, simulation::RapierPhysicsSystem};
use rapier3d::{
    parry::query::DefaultQueryDispatcher,
    prelude::*,
    rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _},
};

impl RapierPhysicsSystem {
    pub fn repulsion_step(&mut self, parameters: &RapierParameters) {
        repulsion_step(self, parameters);
    }
}

fn simple_kernel_2(r: f32, h: f32) -> f32 {
    let v = 1.0 / (r * r) - 1.0 / (h * h);
    v.max(0.0)
}

/// Operates a repulsion between all rigid bodies
/// based on colliders at proximity.
fn repulsion_step(system: &mut RapierPhysicsSystem, parameters: &RapierParameters) {
    let handles = system.nucleotide_body_map.values().collect::<Vec<_>>();

    let constant_factor = 1.0 / 24.0;

    let dt0 = RapierParameters::default().dt;
    let virtual_seconds = parameters.dt / dt0;

    let forces = handles
        .clone()
        .par_iter()
        .map(|handle| {
            let collider = system.collider_set.get(**handle).unwrap();
            let helix_id = collider.user_data;
            let position = *collider.position();

            let force_range = parameters.repulsion_range;
            let force_strength = parameters.repulsion_strength;

            let query_pipeline = system.broad_phase.as_query_pipeline(
                &DefaultQueryDispatcher,
                &system.rigid_body_set,
                &system.collider_set,
                QueryFilter::new(),
            );
            // we query for all colliders within a volume
            query_pipeline
                .intersect_shape(
                    position,
                    &Ball {
                        radius: force_range,
                    },
                )
                // we only keep the objects registered in group 2, which should be the nucleotides
                .filter(|(_, collider)| {
                    collider.collision_groups().test(InteractionGroups::new(
                        Group::GROUP_2,
                        Group::GROUP_2,
                        InteractionTestMode::And,
                    ))
                })
                // we only keep the objects that are from different helixes,
                // filtering out our own helix
                .filter(|(_, collider)| collider.user_data != helix_id)
                // from that we get a list of relative position.translation.vector - collider.position().translation.vector
                .map(|(_, collider)| {
                    position.translation.vector - collider.position().translation.vector
                })
                // which we then filter to only keep valid ranges
                .filter(|v| {
                    v.norm_squared() > f32::EPSILON && v.norm_squared() <= force_range * force_range
                })
                // which we then normalize while keeping its length
                .map(|v| (v.normalize(), v.norm()))
                // which we then multiply by that square, and some other constants
                .map(|(v, d)| {
                    v * simple_kernel_2(d, force_range) * force_strength * constant_factor
                        / virtual_seconds
                })
                // and we then sum all these forces
                .sum()
        })
        .collect::<Vec<Vector<Real>>>();

    for (force, handle) in forces.into_iter().zip(handles.into_iter()) {
        let Some(collider) = system.collider_set.get(*handle) else {
            continue;
        };

        let Some(parent) = collider.parent() else {
            continue;
        };

        let Some(body) = system.rigid_body_set.get_mut(parent) else {
            continue;
        };

        let collider_isometry = collider.position();

        let point = collider_isometry.translation.vector;

        body.add_force_at_point(force, point.into(), true);
    }
}
