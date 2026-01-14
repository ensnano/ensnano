//! This module handles porting the data from ensnano to a physical simulation
//! done through the rapier3d crate.

mod anchors;
mod brown;
mod helices;
pub mod parameters;
mod repulsion;
mod setup;
pub mod simulation;

use rapier3d::{
    na::{Const, OVector},
    prelude::*,
};
use ultraviolet::Vec3;

/// Conversion method used inside this crate.
pub(crate) fn vec_to_vector(v: Vec3) -> OVector<f32, Const<3>> {
    vector![v.x, v.y, v.z]
}
