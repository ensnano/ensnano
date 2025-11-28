//! This module handles porting the data from ensnano to a physical simulation
//! done through the rapier3d crate.

mod anchors;
mod full_simulation;
mod helices;
mod import;
pub mod parameters;
mod repulsion;
pub mod simulation;

use rapier3d::{
    na::{Const, OVector, Point3},
    prelude::*,
};
use ultraviolet::Vec3;

/// Conversion method
pub(crate) fn vec_to_vector(v: Vec3) -> OVector<f32, Const<3>> {
    vector![v.x, v.y, v.z]
}

/// Conversion method
pub(crate) fn point_from_parts(parts: &[f32; 3]) -> Point3<f32> {
    Point3::new(parts[0], parts[1], parts[2])
}
