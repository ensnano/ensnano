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

use super::*;

use chebyshev_polynomials::ChebyshevPolynomial;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolynomialCoordinates {
    pub x: InterpolationDescriptor,
    pub y: InterpolationDescriptor,
    pub z: InterpolationDescriptor,
}

impl PolynomialCoordinates {
    pub(super) fn instanciated(self) -> PolynomialCoordinates_ {
        PolynomialCoordinates_ {
            x: self.x.instanciated(),
            y: self.y.instanciated(),
            z: self.z.instanciated(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct PolynomialCoordinates_ {
    x: ChebyshevPolynomial,
    y: ChebyshevPolynomial,
    z: ChebyshevPolynomial,
}

fn eval_poly_01(t: f64, poly: &ChebyshevPolynomial) -> f64 {
    let interval = poly.definition_interval();
    let t_min = interval[0];
    let t_max = interval[1];

    let arg = t_min * (1. - t) + t_max * t;
    poly.evaluate(arg) / 10.
}

impl Curved for PolynomialCoordinates_ {
    fn position(&self, t: f64) -> DVec3 {
        let x = eval_poly_01(t, &self.x);
        let y = eval_poly_01(t, &self.y);
        let z = eval_poly_01(t, &self.z);
        DVec3 { x, y, z }
    }

    fn bounds(&self) -> CurveBounds {
        CurveBounds::Finite
    }
}
