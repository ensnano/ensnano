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

use super::circle_curve::CircleCurve;
use super::Curved;
use std::f64::consts::{PI, TAU};
use ultraviolet::{DRotor3, DVec3, Vec3};

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

fn default_number_of_helices() -> usize {
    6
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
        let abscissa_converter_factor = Some(circle_radius / (self.radius + section_radius)); // better <= 1

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
