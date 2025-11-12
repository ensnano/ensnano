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

// TODO: check all unused fields (starting with _)

use super::Curved;
use crate::ensnano_design::HelixParameters;
use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};
use ultraviolet::DVec3;

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
        let phi = PI / 2.0 - helix_index * inter_helix_center_gap as f64 / self.radius;
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

    fn curvilinear_abscissa(&self, _t: f64) -> Option<f64> {
        Some(self.z_radius * TAU * _t)
    }

    fn inverse_curvilinear_abscissa(&self, _x: f64) -> Option<f64> {
        Some(_x / TAU / self.z_radius)
    }

    fn bounds(&self) -> super::CurveBounds {
        super::CurveBounds::Finite
    }

    fn full_turn_at_t(&self) -> Option<f64> {
        match self.is_closed {
            Some(false) => None,
            _ => Some(self.t_max()),
        }
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        return self.target_nb_nt;
    }

    fn t_max(&self) -> f64 {
        self.t_max
    }

    fn t_min(&self) -> f64 {
        self.t_min
    }

    fn abscissa_converter(&self) -> Option<crate::ensnano_design::AbscissaConverter> {
        return Some(crate::ensnano_design::AbscissaConverter::linear(
            self.abscissa_converter_factor.unwrap_or(1.),
        ));
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SphereTennisBallSeamDescriptor {
    pub radius: f64,
    pub theta_0_deg: f64,
    pub phi_deg: f64, // in radian 0 is the equator, negative for below the equator, positive above
    pub target_nb_nt: Option<usize>,
}

impl SphereTennisBallSeamDescriptor {
    pub(super) fn with_helix_parameters(
        self,
        helix_parameters: HelixParameters,
    ) -> SphereTennisBallSeam {
        let theta_0 = self.theta_0_deg * PI / 180.;
        let phi = self.phi_deg * PI / 180.;
        let z_radius = self.radius * phi.cos();
        let z = self.radius * phi.sin();
        let t1 = PI * z_radius;
        let t2 = t1 + PI * z;
        let t3 = t2 + PI * z_radius;
        let perimeter = t3 + PI * z;
        SphereTennisBallSeam {
            _parameters: helix_parameters,
            _theta_0: theta_0,
            t1,
            t2,
            t3,
            perimeter,
            _phi: phi,
            z_radius,
            z,
            target_nb_nt: self.target_nb_nt,
        }
    }
}

pub(super) struct SphereTennisBallSeam {
    pub _parameters: HelixParameters,
    pub _theta_0: f64,
    pub z_radius: f64,
    pub z: f64,
    pub _phi: f64,
    pub t1: f64,
    pub t2: f64,
    pub t3: f64,
    pub perimeter: f64,
    pub target_nb_nt: Option<usize>,
}

impl SphereTennisBallSeam {
    pub(super) fn t_max(&self) -> f64 {
        self.perimeter
    }
}

impl Curved for SphereTennisBallSeam {
    fn position(&self, t: f64) -> DVec3 {
        let t = t.rem_euclid(self.perimeter);
        if t < self.t1 {
            let t = t / self.z_radius;
            return DVec3 {
                x: self.z_radius * t.cos(),
                y: self.z_radius * t.sin(),
                z: self.z,
            };
        }
        if t < self.t2 {
            let t = (t - self.t1) / self.z;
            return DVec3 {
                x: -self.z_radius,
                y: -self.z * t.sin(),
                z: self.z * t.cos(),
            };
        }
        if t < self.t3 {
            let t = (t - self.t2) / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.cos(),
                y: self.z_radius * t.sin(),
                z: -self.z,
            };
        }
        let t = (t - self.t3) / self.z;
        return DVec3 {
            x: self.z_radius,
            y: -self.z * t.sin(),
            z: -self.z * t.cos(),
        };
    }

    fn speed(&self, t: f64) -> DVec3 {
        let t = t.rem_euclid(self.perimeter);
        if t < self.t1 {
            let t = t / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.sin(),
                y: self.z_radius * t.cos(),
                z: 0.,
            };
        }
        if t < self.t2 {
            let t = (t - self.t1) / self.z;
            return DVec3 {
                x: 0.,
                y: -self.z * t.cos(),
                z: -self.z * t.sin(),
            };
        }
        if t < self.t3 {
            let t = (t - self.t2) / self.z_radius;
            return DVec3 {
                x: self.z_radius * t.sin(),
                y: self.z_radius * t.cos(),
                z: 0.,
            };
        }
        let t = (t - self.t3) / self.z;
        return DVec3 {
            x: 0.,
            y: -self.z * t.cos(),
            z: self.z * t.sin(),
        };
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        let t = t.rem_euclid(self.perimeter);
        if t < self.t1 {
            let t = t / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.cos(),
                y: -self.z_radius * t.sin(),
                z: 0.,
            };
        }
        if t < self.t2 {
            let t = (t - self.t1) / self.z;
            return DVec3 {
                x: 0.,
                y: self.z * t.sin(),
                z: -self.z * t.cos(),
            };
        }
        if t < self.t3 {
            let t = (t - self.t2) / self.z_radius;
            return DVec3 {
                x: self.z_radius * t.cos(),
                y: -self.z_radius * t.sin(),
                z: 0.,
            };
        }
        let t = (t - self.t3) / self.z;
        return DVec3 {
            x: 0.,
            y: self.z * t.sin(),
            z: self.z * t.cos(),
        };
    }

    fn curvilinear_abscissa(&self, _t: f64) -> Option<f64> {
        Some(_t)
    }

    fn inverse_curvilinear_abscissa(&self, _x: f64) -> Option<f64> {
        Some(_x)
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

    fn full_turn_at_t(&self) -> Option<f64> {
        Some(self.t_max())
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        return self.target_nb_nt;
    }

    fn t_max(&self) -> f64 {
        self.perimeter
    }

    fn t_min(&self) -> f64 {
        0.
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PillTennisBallSeamDescriptor {
    pub radius: f64,
    pub length: f64,
    // pub theta_0_deg: f64,
    pub phi_deg: f64, // in radian 0 is the equator, negative for below the equator, positive above
    pub target_nb_nt: Option<usize>,
}

impl PillTennisBallSeamDescriptor {
    pub(super) fn with_helix_parameters(
        self,
        helix_parameters: HelixParameters,
    ) -> PillTennisBallSeam {
        // let theta_0 = self.theta_0_deg * PI / 180.;
        let phi = self.phi_deg * PI / 180.;
        let z_radius = self.radius * phi.cos();
        let z = self.radius * phi.sin();
        let t1a = self.length;
        let t1 = t1a + PI * z_radius;
        let t1b = t1 + self.length;
        let t2 = t1b + PI * z;
        let t2b = t2 + self.length;
        let t3 = t2b + PI * z_radius;
        let t3b = t3 + self.length;
        let perimeter = t3b + PI * z;
        PillTennisBallSeam {
            _parameters: helix_parameters,
            length: self.length,
            // theta_0,
            t1a,
            t1,
            t1b,
            t2,
            t2b,
            t3,
            t3b,
            perimeter,
            _phi: phi,
            z_radius,
            z,
            target_nb_nt: self.target_nb_nt,
        }
    }
}

pub(super) struct PillTennisBallSeam {
    pub _parameters: HelixParameters,
    pub length: f64,
    pub z_radius: f64,
    pub z: f64,
    pub _phi: f64,
    pub t1a: f64,
    pub t1: f64,
    pub t1b: f64,
    pub t2: f64,
    pub t2b: f64,
    pub t3: f64,
    pub t3b: f64,
    pub perimeter: f64,
    pub target_nb_nt: Option<usize>,
}

impl PillTennisBallSeam {
    pub(super) fn t_max(&self) -> f64 {
        1.0
    }
}

impl Curved for PillTennisBallSeam {
    fn position(&self, t: f64) -> DVec3 {
        let t = (t * self.perimeter).rem_euclid(self.perimeter);
        if t < self.t1a {
            return DVec3 {
                x: self.z_radius,
                y: -self.length / 2.0 + t,
                z: self.z,
            };
        }
        if t < self.t1 {
            let t = (t - self.t1a) / self.z_radius;
            return DVec3 {
                x: self.z_radius * t.cos(),
                y: self.length / 2.0 + self.z_radius * t.sin(),
                z: self.z,
            };
        }
        if t < self.t1b {
            return DVec3 {
                x: -self.z_radius,
                y: self.length / 2.0 - (t - self.t1),
                z: self.z,
            };
        }
        if t < self.t2 {
            let t = (t - self.t1b) / self.z;
            return DVec3 {
                x: -self.z_radius,
                y: -self.length / 2.0 - self.z * t.sin(),
                z: self.z * t.cos(),
            };
        }
        if t < self.t2b {
            return DVec3 {
                x: -self.z_radius,
                y: -self.length / 2.0 + t - self.t2,
                z: -self.z,
            };
        }
        if t < self.t3 {
            let t = (t - self.t2b) / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.cos(),
                y: self.length / 2.0 + self.z_radius * t.sin(),
                z: -self.z,
            };
        }
        if t < self.t3b {
            return DVec3 {
                x: self.z_radius,
                y: self.length / 2.0 - (t - self.t3),
                z: -self.z,
            };
        }
        let t = (t - self.t3b) / self.z;
        return DVec3 {
            x: self.z_radius,
            y: -self.length / 2.0 - self.z * t.sin(),
            z: -self.z * t.cos(),
        };
    }

    fn speed(&self, t: f64) -> DVec3 {
        let t = (t * self.perimeter).rem_euclid(self.perimeter);
        if t < self.t1a {
            return DVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            };
        }
        if t < self.t1 {
            let t = (t - self.t1a) / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.sin(),
                y: self.z_radius * t.cos(),
                z: 0.0,
            };
        }
        if t < self.t1b {
            return DVec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            };
        }
        if t < self.t2 {
            let t = (t - self.t1b) / self.z;
            return DVec3 {
                x: 0.0,
                y: -self.z * t.cos(),
                z: -self.z * t.sin(),
            };
        }
        if t < self.t2b {
            return DVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            };
        }
        if t < self.t3 {
            let t = (t - self.t2b) / self.z_radius;
            return DVec3 {
                x: self.z_radius * t.sin(),
                y: self.z_radius * t.cos(),
                z: 0.0,
            };
        }
        if t < self.t3b {
            return DVec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            };
        }
        let t = (t - self.t3b) / self.z;
        return DVec3 {
            x: 0.0,
            y: -self.z * t.cos(),
            z: self.z * t.sin(),
        };
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        let t = (t * self.perimeter).rem_euclid(self.perimeter);
        if t < self.t1a {
            return DVec3::zero();
        }
        if t < self.t1 {
            let t = (t - self.t1a) / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.cos(),
                y: -self.z_radius * t.sin(),
                z: 0.0,
            };
        }
        if t < self.t1b {
            return DVec3::zero();
        }
        if t < self.t2 {
            let t = (t - self.t1b) / self.z;
            return DVec3 {
                x: 0.0,
                y: self.z * t.sin(),
                z: -self.z * t.cos(),
            };
        }
        if t < self.t2b {
            return DVec3::zero();
        }
        if t < self.t3 {
            let t = (t - self.t2b) / self.z_radius;
            return DVec3 {
                x: self.z_radius * t.cos(),
                y: -self.z_radius * t.sin(),
                z: 0.0,
            };
        }
        if t < self.t3b {
            return DVec3::zero();
        }
        let t = (t - self.t3b) / self.z;
        return DVec3 {
            x: 0.0,
            y: self.z * t.sin(),
            z: self.z * t.cos(),
        };
    }

    fn curvilinear_abscissa(&self, _t: f64) -> Option<f64> {
        Some(_t * self.perimeter)
    }

    fn inverse_curvilinear_abscissa(&self, _x: f64) -> Option<f64> {
        Some(_x / self.perimeter)
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

    fn full_turn_at_t(&self) -> Option<f64> {
        Some(self.t_max())
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        return self.target_nb_nt;
    }

    fn t_max(&self) -> f64 {
        1.
    }

    fn t_min(&self) -> f64 {
        0.
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PillConcentricStadiumDescriptor {
    pub radius: f64,
    pub length: f64,
    pub helix_index: i32, // 0 is the equator, negative for below the equator, positive above
    pub helix_index_shift: Option<f64>, // -0.5 if you want to center the equator between the helices
    pub inter_helix_center_gap: Option<f64>, // in nm, by default 2.65nm
    pub is_closed: Option<bool>,
    pub target_nb_nt: Option<usize>,
    pub abscissa_converter_factor: Option<f64>,
}

impl PillConcentricStadiumDescriptor {
    pub(super) fn with_helix_parameters(
        self,
        helix_parameters: HelixParameters,
    ) -> PillConcentricStadium {
        let helix_index = self.helix_index as f64 + self.helix_index_shift.unwrap_or(0.);
        let inter_helix_center_gap = self
            .inter_helix_center_gap
            .unwrap_or(HelixParameters::INTER_CENTER_GAP as f64);
        let phi = PI / 2.0 - helix_index * inter_helix_center_gap as f64 / self.radius;
        let z_radius = self.radius * phi.sin();
        let z = self.radius * phi.cos();
        let t1 = self.length;
        let t2 = t1 + PI * z_radius;
        let t3 = t2 + self.length;
        let perimeter = TAU * z_radius + 2.0 * self.length;

        PillConcentricStadium {
            _parameters: helix_parameters,
            _radius: self.radius,
            length: self.length,
            _helix_index: helix_index,
            _inter_helix_center_gap: inter_helix_center_gap,
            t1,
            t2,
            t3,
            perimeter,
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

pub(super) struct PillConcentricStadium {
    pub _parameters: HelixParameters,
    pub _radius: f64,
    pub length: f64,
    pub _helix_index: f64,
    pub _inter_helix_center_gap: f64,
    pub t1: f64,
    pub t2: f64,
    pub t3: f64,
    pub perimeter: f64,
    pub _phi: f64,
    pub z_radius: f64,
    pub z: f64,
    pub t_min: f64,
    pub t_max: f64,
    pub is_closed: Option<bool>,
    pub target_nb_nt: Option<usize>,
    pub abscissa_converter_factor: Option<f64>,
}

impl PillConcentricStadium {
    pub(super) fn t_max(&self) -> f64 {
        self.perimeter
    }
}

impl Curved for PillConcentricStadium {
    fn position(&self, t: f64) -> DVec3 {
        let t = (t * self.perimeter).rem_euclid(self.perimeter);
        // println!("{}, {}, {}, {}", t, self.t1, self.t2, self.t3);
        if t < self.t1 {
            return DVec3 {
                x: self.z_radius,
                y: -self.length / 2.0 + t,
                z: self.z,
            };
        }
        if t < self.t2 {
            let t = (t - self.t1) / self.z_radius;
            return DVec3 {
                x: self.z_radius * t.cos(),
                y: self.length / 2.0 + self.z_radius * t.sin(),
                z: self.z,
            };
        }
        if t < self.t3 {
            return DVec3 {
                x: -self.z_radius,
                y: self.length / 2.0 - (t - self.t2),
                z: self.z,
            };
        }
        let t = (t - self.t3) / self.z_radius;
        return DVec3 {
            x: -self.z_radius * t.cos(),
            y: -self.length / 2.0 - self.z_radius * t.sin(),
            z: self.z,
        };
    }

    fn speed(&self, t: f64) -> DVec3 {
        let t = (t * self.perimeter).rem_euclid(self.perimeter);
        if t < self.t1 {
            return DVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            };
        }
        if t < self.t2 {
            let t = (t - self.t1) / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.sin(),
                y: self.z_radius * t.cos(),
                z: 0.0,
            };
        }
        if t < self.t3 {
            return DVec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            };
        }
        let t = (t - self.t3) / self.z_radius;
        return DVec3 {
            x: self.z_radius * t.sin(),
            y: -self.z_radius * t.cos(),
            z: self.z,
        };
    }

    fn acceleration(&self, t: f64) -> DVec3 {
        let t = (t * self.perimeter).rem_euclid(self.perimeter);
        if t < self.t1 {
            return DVec3::zero();
        }
        if t < self.t2 {
            let t = (t - self.t1) / self.z_radius;
            return DVec3 {
                x: -self.z_radius * t.cos(),
                y: -self.z_radius * t.sin(),
                z: 0.0,
            };
        }
        if t < self.t3 {
            return DVec3::zero();
        }
        let t = (t - self.t3) / self.z_radius;
        return DVec3 {
            x: self.z_radius * t.cos(),
            y: self.z_radius * t.sin(),
            z: self.z,
        };
    }

    fn curvilinear_abscissa(&self, _t: f64) -> Option<f64> {
        Some(_t * self.perimeter)
    }

    fn inverse_curvilinear_abscissa(&self, _x: f64) -> Option<f64> {
        Some(_x / self.perimeter)
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

    // fn first_theta(&self) -> Option<f64> {
    //     Some(self.theta_0)
    // }

    // fn last_theta(&self) -> Option<f64> {
    //     Some(self.last_theta())
    // }

    fn full_turn_at_t(&self) -> Option<f64> {
        match self.is_closed {
            Some(false) => None,
            _ => Some(self.t_max()),
        }
    }

    fn objective_nb_nt(&self) -> Option<usize> {
        return self.target_nb_nt;
    }

    fn t_max(&self) -> f64 {
        self.t_max
    }

    fn t_min(&self) -> f64 {
        self.t_min
    }

    fn abscissa_converter(&self) -> Option<crate::ensnano_design::AbscissaConverter> {
        return Some(crate::ensnano_design::AbscissaConverter::linear(
            self.abscissa_converter_factor.unwrap_or(1.),
        ));
    }
}
