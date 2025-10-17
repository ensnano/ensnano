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

use super::Curved;
use crate::HelixParameters;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use ultraviolet::DVec3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircleCurve {
    pub _parameters: HelixParameters,
    pub radius: f64,
    pub z: f64,
    pub perimeter: f64,
    pub abscissa_converter_factor: Option<f64>,
    pub target_nb_nt: Option<usize>, // desired length for the total circle in nt
    pub is_closed: Option<bool>,     // closed unless this is false
}

impl CircleCurve {
    fn theta(&self, t: f64) -> f64 {
        t * TAU
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

    fn objective_nb_nt(&self) -> Option<usize> {
        return self.target_nb_nt;
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        match self.is_closed {
            Some(false) => None,
            _ => Some(self.t_max()),
        }
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
