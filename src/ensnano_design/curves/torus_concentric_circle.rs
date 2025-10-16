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

use super::circle_curve::CircleCurve;
use crate::ensnano_design::HelixParameters;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TorusConcentricCircleDescriptor {
    pub radius: f64,
    pub number_of_helices: u32, // determine the radius together with inter_helix_center_gap
    pub helix_index: i32,       // 0 is the equator, modulo nb_helices, clockwise
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub helix_index_shift: Option<f64>, // -0.5 if you want to center the equator between the helices
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub inter_helix_center_gap: Option<f64>, // in nm, by default 2.65nm
}

impl TorusConcentricCircleDescriptor {
    pub(super) fn with_helix_parameters(self, helix_parameters: &HelixParameters) -> CircleCurve {
        let inter_helix_center_gap = self
            .inter_helix_center_gap
            .unwrap_or(helix_parameters.inter_helix_axis_gap() as f64);
        let inter_helix_angle = TAU / (self.number_of_helices as f64);
        let section_radius = inter_helix_center_gap / 2. / (inter_helix_angle / 2.).sin();
        let φ =
            inter_helix_angle * (self.helix_index as f64 + self.helix_index_shift.unwrap_or(0.));
        let circle_radius = self.radius - section_radius * φ.cos();
        let z = section_radius * φ.sin();
        let perimeter = TAU * circle_radius;
        let abscissa_converter_factor = Some(
            circle_radius / (self.radius + section_radius)
                * HelixParameters::GEARY_2014_DNA.rise as f64
                / helix_parameters.rise as f64,
        ); // better <= 1

        let mut circle_helix_parameters = helix_parameters.clone();
        circle_helix_parameters.inter_helix_gap = inter_helix_center_gap as f32;

        CircleCurve {
            _parameters: circle_helix_parameters,
            radius: circle_radius,
            z,
            perimeter,
            abscissa_converter_factor,
            is_closed: None,
            target_nb_nt: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EllipticTorusConcentricCircleDescriptor {
    pub radius: f64,
    pub horizontal_axis: f64,
    pub vertical_axis: f64,
    // #[serde(skip_serializing_if = "Option::is_none", default)]
    // pub section_angle: Option<f64>, // unused for now
    pub helix_theta: f64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub number_of_helices: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub helix_index: Option<i32>,
}

impl EllipticTorusConcentricCircleDescriptor {
    pub(super) fn with_helix_parameters(self, helix_parameters: &HelixParameters) -> CircleCurve {
        let circle_radius = self.radius - self.horizontal_axis * self.helix_theta.cos();
        let perimeter = TAU * circle_radius;
        let z = self.vertical_axis * self.helix_theta.sin();
        let abscissa_converter_factor = Some(
            circle_radius / (self.radius + self.horizontal_axis)
                * HelixParameters::GEARY_2014_DNA.rise as f64
                / helix_parameters.rise as f64,
        ); // better <= 1

        let circle_helix_parameters = helix_parameters.clone();

        CircleCurve {
            _parameters: circle_helix_parameters,
            radius: circle_radius,
            z,
            perimeter,
            abscissa_converter_factor,
            is_closed: None,
            target_nb_nt: None,
        }
    }
}
