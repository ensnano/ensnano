use super::Curved;
use serde::{Deserialize, Serialize};
use std::f64::consts::{PI, TAU};
use ultraviolet::{DVec2, DVec3};

const INTER_HELIX_GAP: f64 = 2.65;

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct SuperTwist {
    r: f64,
    delta: f64,
    omega: f64,
    nb_helices: usize,
    helix_idx: usize,
    t_max: Option<f64>,
    t_min: Option<f64>,
}

impl Curved for SuperTwist {
    fn bounds(&self) -> super::CurveBounds {
        super::CurveBounds::BiInfinite
    }

    fn t_max(&self) -> f64 {
        if let Some(t_max) = self.t_max {
            t_max.max(1.0)
        } else {
            1.0
        }
    }

    fn t_min(&self) -> f64 {
        if let Some(t_min) = self.t_min {
            t_min.min(0.0)
        } else {
            0.0
        }
    }

    fn position(&self, t: f64) -> DVec3 {
        let ct = (t * self.omega).cos();
        let st = (t * self.omega).sin();

        #[expect(non_snake_case)]
        let M = DVec3 {
            x: self.r * ct,
            y: self.r * st,
            z: self.delta * t,
        };

        let dm_dt = DVec3 {
            x: -self.r * self.delta * st,
            y: self.r * self.delta * ct,
            z: self.delta,
        };

        let ds = DVec2::new(self.r * self.omega, self.delta).mag();

        #[expect(non_snake_case)]
        let Z = dm_dt / ds;

        #[expect(non_snake_case)]
        let X = DVec3 {
            x: -ct,
            y: -st,
            z: 0.0,
        };

        #[expect(non_snake_case)]
        let Y = Z.cross(X);

        let omega_ = TAU * ds / (self.nb_helices as f64 * INTER_HELIX_GAP);

        let angle_per_helix = TAU / self.nb_helices as f64;
        let r = INTER_HELIX_GAP / 2. / (PI / self.nb_helices as f64).sin();
        let angle = omega_ * t + self.helix_idx as f64 * angle_per_helix;
        let ct = r * angle.cos();
        let st = r * angle.sin();

        M + ct * X + st * Y
    }
}
