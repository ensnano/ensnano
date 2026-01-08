use std::f32::consts::PI;

use rand::Rng;
use rapier3d::{
    prelude::*,
    rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _},
};

use crate::simulation::RapierPhysicsSystem;

impl RapierPhysicsSystem {
    pub fn brownian_motion_step(&mut self, delta: f32) {
        brownian_motion_step(self, delta);
    }
}

fn random_unit_vector(rng: &mut impl Rng) -> Vector<f32> {
    let phi: f32 = rng.random_range(0.0..PI * 2.0);
    let costheta: f32 = rng.random_range(-1.0..1.0);

    let theta = costheta.acos();
    let x = theta.sin() * phi.cos();
    let y = theta.sin() * phi.sin();
    let z = theta.cos();

    Vector::new(x, y, z)
}

fn random_force(max_magnitude: f32, rng: &mut impl Rng) -> Vector<f32> {
    let dir = random_unit_vector(rng);
    let magnitude = rng.random_range(0.0..max_magnitude);

    magnitude * dir
}

/// Applies a random force to each nucleotide
fn brownian_motion_step(system: &mut RapierPhysicsSystem, delta: f32) {
    let handles = system.nucleotide_body_map.values().collect::<Vec<_>>();

    let max_magnitude = system.rapier_parameters.brownian_motion_strength;

    if max_magnitude <= 0.0 {
        return;
    }

    let displacements = handles
        .clone()
        .par_iter()
        .map(|handle| {
            let mut rng = rand::rng();
            (*handle, random_force(max_magnitude * delta, &mut rng))
        })
        .collect::<Vec<(&ColliderHandle, Vector<f32>)>>();

    let sum = displacements
        .iter()
        .fold(vector![0.0, 0.0, 0.0], |a, (_, b)| a + *b);

    let average = sum / displacements.len() as f32;

    for (handle, displacement) in displacements {
        let Some(collider) = system.collider_set.get(*handle) else {
            continue;
        };

        let Some(parent) = collider.parent() else {
            continue;
        };

        let Some(body) = system.rigid_body_set.get_mut(parent) else {
            continue;
        };

        let translation = body.translation();

        body.set_translation(translation + displacement - average, true);

        //body.add_force_at_point(force - average, point.into(), true);
    }
}
