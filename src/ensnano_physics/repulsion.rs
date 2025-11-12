use rapier3d::{
    parry::query::DefaultQueryDispatcher,
    prelude::*,
    rayon::iter::{IntoParallelIterator, ParallelIterator},
};

use crate::ensnano_physics::simulation::RapierPhysicsSystem;

const FORCE_RANGE: f32 = 5.0;
const FORCE_STRENGTH: f32 = 0.1;
const FORCE_RANGE_SQUARED: f32 = FORCE_RANGE * FORCE_RANGE;

impl RapierPhysicsSystem {
    pub fn repulsion_step(&mut self, delta: f32) {
        repulsion_step(self, delta);
    }
}

// Following three functions from the "Particle-based Viscoelastic Fluid Simulation"
fn simple_kernel_1(r: f32, h: f32) -> f32 {
    1.0 - r / h
}

/// Operates a repulsion between all rigid bodies
/// based on colliders at proximity.
fn repulsion_step(system: &mut RapierPhysicsSystem, delta: f32) {
    let handles = system.collider_set.iter().map(|p| p.0).collect::<Vec<_>>();

    // let forces = system
    //     .collider_set
    //     .iter()
    let forces = handles
        .into_par_iter()
        .map(|handle| {
            let body = system.collider_set.get(handle).unwrap();
            let position = *body.position();

            let query_pipeline = system.broad_phase.as_query_pipeline(
                &DefaultQueryDispatcher,
                &system.rigid_body_set,
                &system.collider_set,
                QueryFilter::new(),
            );
            // we query for all colliders within a volume
            query_pipeline
                .intersect_shape(
                    *body.position(),
                    &Ball {
                        radius: FORCE_RANGE,
                    },
                )
                // from that we get a list of relative vectors
                .map(|(_, collider)| {
                    position.translation.vector - collider.position().translation.vector
                })
                // which we then filter to only keep valid ranges
                .filter(|v| v.norm_squared() > 0.0 && v.norm_squared() <= FORCE_RANGE_SQUARED)
                // which we then normalize while keeping its length
                .map(|v| (v.normalize(), v.norm()))
                // which we then multiply by that square, and some other constants
                .map(|(v, d)| v * simple_kernel_1(d, FORCE_RANGE) * delta * FORCE_STRENGTH)
                // and we then sum all these forces
                .sum()
        })
        .collect::<Vec<_>>();

    for (force, (_, collider)) in forces.iter().zip(system.collider_set.iter()) {
        let Some(parent) = collider.parent() else {
            continue;
        };
        let Some(body) = system.rigid_body_set.get_mut(parent) else {
            continue;
        };

        body.apply_impulse_at_point(*force, collider.position().translation.vector.into(), true);
    }
}
