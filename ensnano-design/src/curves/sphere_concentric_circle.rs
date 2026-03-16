// TODO: check all unused fields (starting with _)

#[cfg(feature = "ensnano_upcoming")]
use crate::curves::CurveConstructor;
use crate::{
    curves::{CurveBounds, Curved, time_nucl_map::AbscissaConverter},
    parameters::HelixParameters,
};
use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};
use ultraviolet::DVec3;

#[cfg(feature = "ensnano_upcoming")]
use ensnano_upcoming::{
    PillConcentricStadium, PillConcentricStadiumDescriptor, PillTennisBallSeam,
    SphereTennisBallSeam,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SphereConcentricCircleDescriptor {
    pub radius: f64,
    pub theta_0: f64,
    pub helix_index: i32, // 0 is the equator, negative for below the equator, positive above
    pub helix_index_shift: Option<f64>, // -0.5 if you want to center the equator between the helices
    pub inter_helix_center_gap: Option<f64>, // in nm, by default 2.65nm
    pub is_closed: Option<bool>,
    pub target_nb_nt: Option<usize>,
    pub abscissa_converter_factor: Option<f64>,
}

impl SphereConcentricCircleDescriptor {
    pub(super) fn with_helix_parameters(
        self,
        helix_parameters: HelixParameters,
    ) -> SphereConcentricCircle {
        let helix_index = self.helix_index as f64 + self.helix_index_shift.unwrap_or(0.);
        let inter_helix_center_gap = self
            .inter_helix_center_gap
            .unwrap_or(HelixParameters::INTER_CENTER_GAP as f64);
        let phi = PI / 2.0 - helix_index * inter_helix_center_gap / self.radius;
        let z_radius = self.radius * phi.sin();
        let z = self.radius * phi.cos();
        let perimeter = TAU * z_radius;

        SphereConcentricCircle {
            _parameters: helix_parameters,
            _radius: self.radius,
            theta_0: self.theta_0,
            _helix_index: helix_index,
            _inter_helix_center_gap: inter_helix_center_gap,
            _perimeter: perimeter,
            _phi: phi,
            z_radius,
            z,
            t_min: 0.,
            t_max: 1.,
            is_closed: self.is_closed,
            target_nb_nt: self.target_nb_nt,
            abscissa_converter_factor: Some(
                self.abscissa_converter_factor.unwrap_or(1.)
                    * HelixParameters::GEARY_2014_DNA.rise as f64
                    / helix_parameters.rise as f64,
            ),
        }
    }
}

pub(super) struct SphereConcentricCircle {
    pub _parameters: HelixParameters,
    pub _radius: f64,
    pub theta_0: f64,
    pub _helix_index: f64,
    pub _inter_helix_center_gap: f64,
    pub _perimeter: f64,
    pub _phi: f64,
    pub z_radius: f64,
    pub z: f64,
    pub t_min: f64,
    pub t_max: f64,
    pub is_closed: Option<bool>,
    pub target_nb_nt: Option<usize>,
    pub abscissa_converter_factor: Option<f64>,
}

impl SphereConcentricCircle {
    fn theta(&self, t: f64) -> f64 {
        t * TAU + self.theta_0
    }

    pub(super) fn t_max(&self) -> f64 {
        1.
    }
}

impl Curved for SphereConcentricCircle {
    fn position(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);
        DVec3 {
            x: self.z_radius * theta.cos(),
            y: self.z_radius * theta.sin(),
            z: self.z,
        }
    }

    fn speed(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);

        let x = -self.z_radius * TAU * theta.sin();

        let y = self.z_radius * TAU * theta.cos();

        let z = 0.0;

        DVec3 { x, y, z }
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);

        let x = -self.z_radius * TAU * TAU * theta.cos();

        let y = -self.z_radius * TAU * TAU * theta.sin();

        let z = 0.;

        DVec3 { x, y, z }
    }

    fn curvilinear_abscissa(&self, t: f64) -> Option<f64> {
        Some(self.z_radius * TAU * t)
    }

    fn inverse_curvilinear_abscissa(&self, x: f64) -> Option<f64> {
        Some(x / TAU / self.z_radius)
    }

    fn bounds(&self) -> CurveBounds {
        CurveBounds::Finite
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        match self.is_closed {
            Some(false) => None,
            _ => Some(self.t_max()),
        }
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        self.target_nb_nt
    }

    fn t_max(&self) -> f64 {
        self.t_max
    }

    fn t_min(&self) -> f64 {
        self.t_min
    }

    fn abscissa_converter(&self) -> Option<AbscissaConverter> {
        Some(AbscissaConverter::linear(
            self.abscissa_converter_factor.unwrap_or(1.),
        ))
    }
}

#[cfg(feature = "ensnano_upcoming")]
impl Curved for SphereTennisBallSeam {
    fn position(&self, t: f64) -> DVec3 {
        self.position(t)
    }

    fn speed(&self, t: f64) -> DVec3 {
        self.speed(t)
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        self.acceleration(t)
    }

    fn curvilinear_abscissa(&self, t: f64) -> Option<f64> {
        Some(t)
    }

    fn inverse_curvilinear_abscissa(&self, x: f64) -> Option<f64> {
        Some(x)
    }

    fn bounds(&self) -> CurveBounds {
        CurveBounds::Finite
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        Some(self.t_max())
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        self.target_nb_nt
    }

    fn t_max(&self) -> f64 {
        self.perimeter
    }

    fn t_min(&self) -> f64 {
        0.
    }
}

#[cfg(feature = "ensnano_upcoming")]
impl Curved for PillTennisBallSeam {
    fn position(&self, t: f64) -> DVec3 {
        self.position(t)
    }

    fn speed(&self, t: f64) -> DVec3 {
        self.speed(t)
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        self.acceleration(t)
    }

    fn curvilinear_abscissa(&self, t: f64) -> Option<f64> {
        self.curvilinear_abscissa(t)
    }

    fn inverse_curvilinear_abscissa(&self, x: f64) -> Option<f64> {
        self.inverse_curvilinear_abscissa(x)
    }

    fn bounds(&self) -> CurveBounds {
        CurveBounds::Finite
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        Some(self.t_max())
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        self.target_nb_nt
    }

    fn t_max(&self) -> f64 {
        1.
    }

    fn t_min(&self) -> f64 {
        0.
    }
}

#[cfg(feature = "ensnano_upcoming")]
impl CurveConstructor for PillConcentricStadiumDescriptor {
    type Curve = PillConcentricStadium;

    fn instantiate_with_parameters(&self, parameters: HelixParameters) -> PillConcentricStadium {
        self.instantiate(
            HelixParameters::INTER_CENTER_GAP as f64,
            parameters.rise as f64,
            HelixParameters::GEARY_2014_DNA.rise as f64,
        )
    }
}

#[cfg(feature = "ensnano_upcoming")]
impl Curved for PillConcentricStadium {
    fn position(&self, t: f64) -> DVec3 {
        self.position(t)
    }

    fn speed(&self, t: f64) -> DVec3 {
        self.speed(t)
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        self.acceleration(t)
    }

    fn curvilinear_abscissa(&self, t: f64) -> Option<f64> {
        Some(t * self.perimeter)
    }

    fn inverse_curvilinear_abscissa(&self, x: f64) -> Option<f64> {
        Some(x / self.perimeter)
    }

    fn bounds(&self) -> CurveBounds {
        CurveBounds::Finite
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        match self.is_closed {
            Some(false) => None,
            _ => Some(self.t_max()),
        }
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        self.target_nb_nt
    }

    fn t_max(&self) -> f64 {
        self.t_max
    }

    fn t_min(&self) -> f64 {
        self.t_min
    }

    fn abscissa_converter(&self) -> Option<AbscissaConverter> {
        Some(AbscissaConverter::linear(
            self.abscissa_converter_factor.unwrap_or(1.),
        ))
    }
}
