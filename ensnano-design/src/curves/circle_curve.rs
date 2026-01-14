use crate::{
    curves::{CurveBounds, Curved, time_nucl_map::AbscissaConverter},
    parameters::HelixParameters,
};
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use ultraviolet::DVec3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct CircleCurve {
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

    fn curvilinear_abscissa(&self, t: f64) -> Option<f64> {
        Some(self.radius * TAU * t)
    }

    fn inverse_curvilinear_abscissa(&self, x: f64) -> Option<f64> {
        Some(x / TAU / self.radius)
    }

    fn bounds(&self) -> CurveBounds {
        CurveBounds::Finite
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        self.target_nb_nt
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

    fn abscissa_converter(&self) -> Option<AbscissaConverter> {
        Some(AbscissaConverter::linear(
            self.abscissa_converter_factor.unwrap_or(1.),
        ))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CircleDescriptor {
    pub radius: f64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub abscissa_converter_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_closed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub target_nb_nt: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub helix_parameters: Option<HelixParameters>,
}

impl CircleDescriptor {
    pub(super) fn with_helix_parameters(self, helix_parameters: &HelixParameters) -> CircleCurve {
        let circle_helix_parameters = self.helix_parameters.unwrap_or(*helix_parameters);
        let perimeter = TAU * self.radius;
        CircleCurve {
            _parameters: circle_helix_parameters,
            radius: self.radius,
            z: 0.,
            perimeter,
            abscissa_converter_factor: self.abscissa_converter_factor,
            is_closed: self.is_closed,
            target_nb_nt: self.target_nb_nt,
        }
    }
}
