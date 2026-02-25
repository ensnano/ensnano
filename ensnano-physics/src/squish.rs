use crate::{parameters::RapierParameters, simulation::RapierPhysicsSystem};
use rapier3d::{
    prelude::*,
    rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _},
};

impl RapierPhysicsSystem {
    pub fn squish_step(&mut self, parameters: &RapierParameters) {
        squish_step(self, parameters);
    }
}

/// Operates a squishing force towards the y = 0 plane.
fn squish_step(system: &mut RapierPhysicsSystem, parameters: &RapierParameters) {
    let handles = system.nucleotide_body_map.values().collect::<Vec<_>>();

    let constant_factor = 1.0 / 24.0;

    if parameters.squish_strength <= f32::EPSILON {
        return;
    }

    let forces = handles
        .clone()
        .par_iter()
        .map(|handle| {
            let collider = system.collider_set.get(**handle).unwrap();
            let position = *collider.position();

            let Some(parent) = collider.parent() else {
                return Vector::default();
            };

            let Some(body) = system.rigid_body_set.get(parent) else {
                return Vector::default();
            };

            let strength = -position.translation.y;
            let strength = strength.clamp(
                -parameters.squish_soft_cutoff,
                parameters.squish_soft_cutoff,
            );
            let strength = constant_factor * parameters.squish_strength * strength
                / parameters.squish_soft_cutoff;

            let damping = -body.linvel()[1];
            let damping = damping * parameters.squish_damping * constant_factor;

            vector![0.0, strength + damping, 0.0]
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
