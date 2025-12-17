// TODO: split in 2 files

use ultraviolet::{Bivec3, DBivec3, DRotor3, DVec3, Rotor3, Vec3};

// === GEOMETRIC ALGEBRA CONVERSIONS ===

pub fn vec_to_dvec(v: Vec3) -> DVec3 {
    DVec3 {
        x: v.x as f64,
        y: v.y as f64,
        z: v.z as f64,
    }
}

pub fn bivec_to_dbivec(bv: Bivec3) -> DBivec3 {
    DBivec3 {
        xy: bv.xy as f64,
        xz: bv.xz as f64,
        yz: bv.yz as f64,
    }
}

pub fn rotor_to_drotor(rot: Rotor3) -> DRotor3 {
    DRotor3 {
        s: rot.s as f64,
        bv: bivec_to_dbivec(rot.bv),
    }
}

pub fn dvec_to_vec(dv: DVec3) -> Vec3 {
    Vec3 {
        x: dv.x as f32,
        y: dv.y as f32,
        z: dv.z as f32,
    }
}

// === SERIALIZATION UTILS ===

pub(crate) fn isize_is_zero(x: &isize) -> bool {
    *x == 0
}

pub(crate) fn f32_is_zero(x: &f32) -> bool {
    *x == 0.0
}

pub(crate) fn is_false(x: &bool) -> bool {
    !x
}
