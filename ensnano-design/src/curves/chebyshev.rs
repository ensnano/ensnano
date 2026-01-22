use crate::{
    chebyshev_polynomials::ChebyshevPolynomial,
    curves::{CurveBounds, Curved, revolution::InterpolationDescriptor},
};
use serde::{Deserialize, Serialize};
use ultraviolet::DVec3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolynomialCoordinates {
    pub x: InterpolationDescriptor,
    pub y: InterpolationDescriptor,
    pub z: InterpolationDescriptor,
}

impl PolynomialCoordinates {
    pub(super) fn instantiated(self) -> PolynomialCoordinates_ {
        PolynomialCoordinates_ {
            x: self.x.instantiated(),
            y: self.y.instantiated(),
            z: self.z.instantiated(),
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
