use crate::simulation::RapierPhysicsSystem;
use rapier3d::{
    parry::query::DefaultQueryDispatcher,
    prelude::*,
    rayon::iter::{IntoParallelIterator as _, ParallelIterator as _},
};

// const FORCE_RANGE: f32 = 5.0;
// const FORCE_STRENGTH: f32 = 0.1;

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
        .clone()
        .into_par_iter()
        .map(|handle| {
            let collider = system.collider_set.get(handle).unwrap();
            let position = *collider.position();

            let force_range = system.rapier_parameters.repulsion_range;
            let force_strength = system.rapier_parameters.repulsion_strength;

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
                        InteractionTestMode::Or,
                    ))
                })
                // from that we get a list of relative vectorsosition.translation.vector - collider.position().translation.vector
                .map(|(_, collider)| {
                    position.translation.vector - collider.position().translation.vector
                })
                // which we then filter to only keep valid ranges
                .filter(|v| v.norm_squared() > 0.0 && v.norm_squared() <= force_range * force_range)
                // which we then normalize while keeping its length
                .map(|v| (v.normalize(), v.norm()))
                // which we then multiply by that square, and some other constants
                .map(|(v, d)| v * simple_kernel_1(d, force_range) * delta * force_strength)
                // and we then sum all these forces
                .sum()
        })
        .collect::<Vec<Vector<Real>>>();

    for (force, handle) in forces.into_iter().zip(handles.into_iter()) {
        let Some(collider) = system.collider_set.get(handle) else {
            continue;
        };

        let Some(parent) = collider.parent() else {
            continue;
        };

        let Some(body) = system.rigid_body_set.get_mut(parent) else {
            continue;
        };

        // let Some(isometry) = collider.position() else {
        //     continue;
        // };

        let isometry = collider.position();

        let force: Vector<Real> = isometry.rotation.inverse() * force;

        body.apply_impulse_at_point(force, isometry.translation.vector.into(), true);
    }
}
