/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! This module handles porting the data from ensnano to a physical simulation
//! done through the rapier3d crate.
//!

mod anchors;
mod full_simulation;
mod helices;
mod import;
mod repulsion;
mod simulation;

use rapier3d::{
    na::{Const, OVector, Point3},
    prelude::*,
};
pub use simulation::RapierPhysicsSystem;
use ultraviolet::Vec3;

/// Conversion method
pub(crate) fn vec_to_vector(v: Vec3) -> OVector<f32, Const<3>> {
    vector![v.x, v.y, v.z]
}

/// Conversion method
pub(crate) fn point_from_parts(parts: &[f32; 3]) -> Point3<f32> {
    Point3::new(parts[0], parts[1], parts[2])
}
