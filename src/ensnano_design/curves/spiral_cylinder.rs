use super::Curved;
use crate::ensnano_design::HelixParameters;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use ultraviolet::DVec3;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SpiralCylinderDescriptor {
    pub theta_0: f64,
    pub radius: f64,
    pub number_of_turns: f64,
    #[serde(default = "default_number_of_helices")]
    pub number_of_helices: usize,
    pub helix_index: usize,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub inter_helix_axis_gap: Option<f64>,
}

fn default_number_of_helices() -> usize {
    2
}

impl SpiralCylinderDescriptor {
    pub(super) fn with_helix_parameters(self, helix_parameters: HelixParameters) -> SpiralCylinder {
        let inter_helix_axis_gap = if let Some(ihg) = self.inter_helix_axis_gap {
            ihg
        } else {
            helix_parameters.inter_helix_axis_gap() as f64
        };
        let rise_per_turn = self.rise_per_turn(inter_helix_axis_gap);
        let rt = self.radius * TAU;
        let d_curvilinear_abscissa = f64::hypot(rt, rise_per_turn);
        SpiralCylinder {
            theta_0: self.theta_0,
            radius: self.radius,
            _parameters: helix_parameters,
            number_of_turns: self.number_of_turns,
            number_of_helices: self.number_of_helices,
            helix_index: self.helix_index % self.number_of_helices,
            _inter_helix_axis_gap: inter_helix_axis_gap,
            rise_per_turn,
            d_curvilinear_abscissa,
        }
    }

    fn rise_per_turn(&self, inter_helix_axis_gap: f64) -> f64 {
        let slope = self.number_of_helices as f64 * inter_helix_axis_gap / TAU / self.radius;
        assert!(
            slope < 1.0,
            "Radius for spiral_cylinder is too small wtr inter helix axis gap"
        );
        self.number_of_helices as f64 * inter_helix_axis_gap / (1.0 - slope * slope).sqrt()
    }
}

pub(super) struct SpiralCylinder {
    pub theta_0: f64,
    pub radius: f64,
    pub number_of_turns: f64,
    pub _parameters: HelixParameters,
    pub number_of_helices: usize,
    pub _inter_helix_axis_gap: f64,
    pub helix_index: usize,
    pub rise_per_turn: f64,          // computed by SpiralCylinderDescriptor
    pub d_curvilinear_abscissa: f64, // computed by SpiralCylinderDescriptor: derivative of the curvilinear abscissa by t
}

impl SpiralCylinder {
    fn theta(&self, t: f64) -> f64 {
        t * TAU + self.theta_0 + TAU * self.helix_index as f64 / self.number_of_helices as f64
    }
}

impl Curved for SpiralCylinder {
    fn t_max(&self) -> f64 {
        self.number_of_turns + 1.0
    }

    fn t_min(&self) -> f64 {
        -1.0
    }

    fn position(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);
        DVec3 {
            x: self.radius * theta.cos(),
            y: self.radius * theta.sin(),
            z: self.rise_per_turn * t,
        }
    }

    fn speed(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);
        DVec3 {
            x: -self.radius * TAU * theta.sin(),
            y: self.radius * TAU * theta.cos(),
            z: self.rise_per_turn,
        }
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        let theta = self.theta(t);
        DVec3 {
            x: -self.radius * TAU * TAU * theta.cos(),
            y: -self.radius * TAU * TAU * theta.sin(),
            z: 0.0,
        }
    }

    fn bounds(&self) -> super::CurveBounds {
        super::CurveBounds::BiInfinite
    }

    fn is_time_maps_singleton(&self) -> bool {
        true
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        Some(1.0)
    }

    fn curvilinear_abscissa(&self, t: f64) -> Option<f64> {
        Some(self.d_curvilinear_abscissa * (t - self.t_min()))
    }

    fn inverse_curvilinear_abscissa(&self, x: f64) -> Option<f64> {
        Some(x / self.d_curvilinear_abscissa + self.t_min())
    }
}
