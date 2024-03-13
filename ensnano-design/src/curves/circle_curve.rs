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

use crate::{parameters, HelixParameters};

use super::Curved;
use std::f64::consts::{PI, TAU};
use ultraviolet::{DRotor3, DVec3, Vec3};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircleCurve {
    pub _parameters: HelixParameters,
    pub radius: f64,
    pub z: f64,
    pub perimeter: f64,
    pub abscissa_converter_factor: Option<f64>,
}

impl CircleCurve {
    fn theta(&self, t: f64) -> f64 {
        t * TAU
    }

    pub(super) fn last_theta(&self) -> f64 {
        self.theta(1.)
    }

    pub(super) fn t_min(&self) -> f64 {
        0.
    }

    pub(super) fn t_max(&self) -> f64 {
        1.
    }
}

impl Curved for CircleCurve {
    fn position(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);
        DVec3 {
            x: self.radius * theta.cos(),
            y: self.radius * theta.sin(),
            z: self.z,
        }
    }

    fn speed(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);

        let x = -self.radius * TAU * theta.sin();

        let y = self.radius * TAU * theta.cos();

        let z = 0.0;

        DVec3 { x, y, z }
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);

        let x = -self.radius * TAU * TAU * theta.cos();

        let y = -self.radius * TAU * TAU * theta.sin();

        let z = 0.;

        DVec3 { x, y, z }
    }

    fn curvilinear_abscissa(&self, _t: f64) -> Option<f64> {
        Some(self.radius * TAU * _t)
    }

    fn inverse_curvilinear_abscissa(&self, _x: f64) -> Option<f64> {
        Some(_x / TAU / self.radius)
    }

    fn bounds(&self) -> super::CurveBounds {
        super::CurveBounds::Finite
    }

    // fn subdivision_for_t(&self, t: f64) -> Option<usize> {
    //     None
    // }

    // fn is_time_maps_singleton(&self) -> bool {
    //     true
    // }

    fn first_theta(&self) -> Option<f64> {
        Some(0.)
    }

    fn last_theta(&self) -> Option<f64> {
        Some(self.last_theta())
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        Some(self.t_max())
    }

    fn t_max(&self) -> f64 {
        1.
    }

    fn t_min(&self) -> f64 {
        0.
    }

    fn abscissa_converter(&self) -> Option<crate::AbscissaConverter> {
        return Some(crate::AbscissaConverter::linear(
            self.abscissa_converter_factor.unwrap_or(1.),
        ));
    }
}
