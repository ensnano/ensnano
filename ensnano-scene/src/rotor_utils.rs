use std::f32::consts::PI;
use ultraviolet::{Rotor3, Vec3};

pub(crate) trait SafeRotor {
    fn safe_from_rotation_from_unit_x_to(u: Vec3) -> Rotor3;
    fn safe_from_rotation_to_unit_x_from(u: Vec3) -> Rotor3;
}

impl SafeRotor for Rotor3 {
    fn safe_from_rotation_from_unit_x_to(u: Vec3) -> Rotor3 {
        // u must be normalized
        let eps: f32 = 1e-5;
        let ux = Vec3::unit_x();
        let ux_dot_u = u.x; //ux.dot(u);
        if ux_dot_u > 1. - eps {
            Self::identity()
        } else if ux_dot_u < -1. + eps {
            Self::from_rotation_xy(PI)
        } else {
            Self::from_rotation_between(ux, u)
        }
    }

    fn safe_from_rotation_to_unit_x_from(u: Vec3) -> Rotor3 {
        // u must be normalized
        let eps: f32 = 1e-5;
        let ux = Vec3::unit_x();
        let ux_dot_u = u.x; //ux.dot(u);
        if ux_dot_u > 1. - eps {
            Self::identity()
        } else if ux_dot_u < -1. + eps {
            Self::from_rotation_xy(PI)
        } else {
            Self::from_rotation_between(u, ux)
        }
    }
}
