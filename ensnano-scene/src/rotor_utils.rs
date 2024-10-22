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

use ensnano_design::ultraviolet::*;
use std::f32::consts::PI;

pub trait SafeRotor {
    fn safe_from_rotation_from_unit_x_to(u: Vec3) -> Rotor3;
    fn safe_from_rotation_to_unit_x_from(u: Vec3) -> Rotor3;
}

impl SafeRotor for Rotor3 {
    fn safe_from_rotation_from_unit_x_to(u: Vec3) -> Rotor3 {
        // u must be normalized
        let _ε: f32 = 1e-5;
        let ux = Vec3::unit_x();
        let ux_dot_u = u.x; //ux.dot(u);
        if ux_dot_u > 1. - _ε {
            return Rotor3::identity();
        } else if ux_dot_u < -1. + _ε {
            return Rotor3::from_rotation_xy(PI);
        } else {
            return Rotor3::from_rotation_between(ux, u);
        };
    }

    fn safe_from_rotation_to_unit_x_from(u: Vec3) -> Rotor3 {
        // u must be normalized
        let _ε: f32 = 1e-5;
        let ux = Vec3::unit_x();
        let ux_dot_u = u.x; //ux.dot(u);
        if ux_dot_u > 1. - _ε {
            return Rotor3::identity();
        } else if ux_dot_u < -1. + _ε {
            return Rotor3::from_rotation_xy(PI);
        } else {
            return Rotor3::from_rotation_between(u, ux);
        };
    }
}
