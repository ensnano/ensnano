use crate::{
    app_state::AppState,
    gui::curve::{CurveDescriptorBuilder, CurveDescriptorParameter, InstantiatedParameter},
};
use ensnano_design::{
    bezier_plane::BezierPathId, curves::bezier, curves::torus::CurveDescriptor2D,
};
use ordered_float::OrderedFloat;
use std::f64::consts::PI;
use ultraviolet::{Rotor3, Vec3};

pub(super) const ELLIPSE_BUILDER: CurveDescriptorBuilder = CurveDescriptorBuilder {
    curve_name: "Ellipse",
    parameters: &[
        CurveDescriptorParameter {
            name: "Semi major axis",
            default_value: InstantiatedParameter::Float(10.0),
        },
        CurveDescriptorParameter {
            name: "Semi minor axis",
            default_value: InstantiatedParameter::Float(5.0),
        },
    ],
    build: &build_ellipse,
    bezier_path_id: &no_bezier_path_id,
    rotational_symmetry_order: &rotational_symmetry_order_ellipse,
    frame: &default_frame,
};

fn build_ellipse(parameters: &[InstantiatedParameter], _: &AppState) -> Option<CurveDescriptor2D> {
    let a = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_float)?;
    let b = parameters
        .get(1)
        .copied()
        .and_then(InstantiatedParameter::get_float)?;
    Some(CurveDescriptor2D::Ellipse {
        semi_minor_axis: b.into(),
        semi_major_axis: a.into(),
    })
}

fn rotational_symmetry_order_ellipse(parameters: &[InstantiatedParameter]) -> Option<usize> {
    let a = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_float)?;
    let b = parameters
        .get(1)
        .copied()
        .and_then(InstantiatedParameter::get_float)?;
    if a == b {
        return Some(0);
    }
    return Some(2);
}

pub(super) const TWO_SPHERES_BUILDER: CurveDescriptorBuilder = CurveDescriptorBuilder {
    curve_name: "Two spheres",
    parameters: &[
        CurveDescriptorParameter {
            name: "Radius extern",
            default_value: InstantiatedParameter::Float(25.),
        },
        CurveDescriptorParameter {
            name: "Radius intern",
            default_value: InstantiatedParameter::Float(17.),
        },
        CurveDescriptorParameter {
            name: "Radius tube",
            default_value: InstantiatedParameter::Float(7.6),
        },
        CurveDescriptorParameter {
            name: "Smooth ceil",
            default_value: InstantiatedParameter::Float(0.04),
        },
    ],
    build: &build_two_spheres,
    bezier_path_id: &no_bezier_path_id,
    rotational_symmetry_order: &no_rotational_symmetry_order,
    frame: &default_frame,
};

fn build_two_spheres(
    parameters: &[InstantiatedParameter],
    _: &AppState,
) -> Option<CurveDescriptor2D> {
    let radius_extern: OrderedFloat<f64> = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_float)?
        .into();
    let radius_intern: OrderedFloat<f64> = parameters
        .get(1)
        .copied()
        .and_then(InstantiatedParameter::get_float)?
        .into();
    let radius_tube = parameters
        .get(2)
        .copied()
        .and_then(InstantiatedParameter::get_float)?
        .into();
    let smooth_ceil = parameters
        .get(3)
        .copied()
        .and_then(InstantiatedParameter::get_float)?
        .into();

    Some(CurveDescriptor2D::TwoBalls {
        radius_extern: radius_extern.max(radius_intern),
        radius_intern: radius_intern.min(radius_extern),
        radius_tube,
        smooth_ceil,
    })
}

pub(super) const BEZIER_CURVE_BUILDER: CurveDescriptorBuilder = CurveDescriptorBuilder {
    curve_name: "Bezier",
    parameters: &[CurveDescriptorParameter {
        name: "Path n°",
        default_value: InstantiatedParameter::Uint(0),
    }],
    build: &build_bezier,
    bezier_path_id: &get_bezier_path_id,
    rotational_symmetry_order: &get_rotational_symmetry_order,
    frame: &get_bezier_frame,
};

fn build_bezier(parameters: &[InstantiatedParameter], app: &AppState) -> Option<CurveDescriptor2D> {
    let curve_id = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_uint)?;

    let bezier = app
        .0
        .design
        .clone_inner()
        .get_bezier_path_2d(BezierPathId(curve_id as u32));

    if let Some(bezier) = bezier {
        // let rotational_symmetry_order = Some(parameters
        //     .get(1)
        //     .copied()
        //     .and_then(InstantiatedParameter::get_uint)
        //     .unwrap_or(1)
        //     .max(1)); // 0 is not allowed

        return Some(CurveDescriptor2D::Bezier(bezier));
    } else {
        return None;
    }
}

fn no_bezier_path_id(_: &[InstantiatedParameter]) -> Option<usize> {
    None
}

fn no_rotational_symmetry_order(_: &[InstantiatedParameter]) -> Option<usize> {
    Some(1)
}

fn get_bezier_path_id(parameters: &[InstantiatedParameter]) -> Option<usize> {
    parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_uint)
}

fn get_rotational_symmetry_order(parameters: &[InstantiatedParameter]) -> Option<usize> {
    let value = parameters
        .get(1)
        .copied()
        .and_then(InstantiatedParameter::get_uint)?;
    Some(value.max(1))
}

fn get_bezier_frame(
    parameters: &[InstantiatedParameter],
    app: &AppState,
) -> Option<(Vec3, Rotor3)> {
    let path_id = get_bezier_path_id(parameters)?;
    app.0
        .design
        .clone_inner()
        .get_first_bezier_plane(BezierPathId(path_id as u32))
        .map(|plane| (plane.position, plane.orientation))
}

fn default_frame(_: &[InstantiatedParameter], app: &AppState) -> Option<(Vec3, Rotor3)> {
    app.0
        .design
        .clone_inner()
        .get_default_bezier()
        .map(|plane| (plane.position, plane.orientation))
}

const MINIMUM_NB_BRANCHES_STAR: usize = 4;
const STAR_PATH_ID: u64 = 666;
pub(super) const STAR_BUILDER: CurveDescriptorBuilder = CurveDescriptorBuilder {
    curve_name: "Star",
    parameters: &[
        CurveDescriptorParameter {
            name: "Nb of branches",
            default_value: InstantiatedParameter::Uint(5),
        },
        CurveDescriptorParameter {
            name: "External radius",
            default_value: InstantiatedParameter::Float(20.0),
        },
        CurveDescriptorParameter {
            name: "Internal radius",
            default_value: InstantiatedParameter::Float(10.0),
        },
    ],
    build: &build_star,
    bezier_path_id: &no_bezier_path_id,
    rotational_symmetry_order: &rotational_symmetry_order_star,
    frame: &default_frame,
};

fn build_star(parameters: &[InstantiatedParameter], _: &AppState) -> Option<CurveDescriptor2D> {
    let nb_branches = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_uint)?
        .max(MINIMUM_NB_BRANCHES_STAR);
    let r2: f64 = parameters
        .get(1)
        .copied()
        .and_then(InstantiatedParameter::get_float)?
        .into();
    let r1: f64 = parameters
        .get(2)
        .copied()
        .and_then(InstantiatedParameter::get_float)?
        .into();

    let a = PI / nb_branches as f64;
    let mut points = Vec::new();
    for i in 0..nb_branches {
        let b: f64 = 2. * a * i as f64;
        points.push(Vec3::new((r1 * b.cos()) as f32, (r1 * b.sin()) as f32, 0.));
        points.push(Vec3::new(
            (r2 * (b + a).cos()) as f32,
            (r2 * (b + a).sin()) as f32,
            0.,
        ));
    }

    let mut bezier_points = Vec::new();
    for ((u, v), w) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .zip(points.iter().cycle().skip(2))
    {
        bezier_points.push(bezier::BezierEndCoordinates {
            position: v.clone(),
            vector_in: (*w - *u) / 6.,
            vector_out: (*w - *u) / 6.,
        });
    }
    let bezier = bezier::InstantiatedPiecewiseBezier {
        ends: bezier_points,
        is_cyclic: true,
        t_min: Some(0.),
        t_max: Some(2. * nb_branches as f64),
        id: STAR_PATH_ID,
        discretize_quickly: true,
    };

    Some(CurveDescriptor2D::Bezier(bezier))
}

fn rotational_symmetry_order_star(parameters: &[InstantiatedParameter]) -> Option<usize> {
    let value = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_uint)?;
    Some(value.max(MINIMUM_NB_BRANCHES_STAR))
}

pub(super) const NONE_BUILDER: CurveDescriptorBuilder = CurveDescriptorBuilder {
    curve_name: "None",
    parameters: &[],
    build: &build_none,
    bezier_path_id: &no_bezier_path_id,
    rotational_symmetry_order: &rotational_symmetry_order_none,
    frame: &none_frame,
};

fn build_none(parameters: &[InstantiatedParameter], app: &AppState) -> Option<CurveDescriptor2D> {
    None
}

fn none_frame(_: &[InstantiatedParameter], app: &AppState) -> Option<(Vec3, Rotor3)> {
    None
}

fn rotational_symmetry_order_none(parameters: &[InstantiatedParameter]) -> Option<usize> {
    None
}
