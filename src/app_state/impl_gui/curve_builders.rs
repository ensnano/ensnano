use crate::app_state::AppState;
use crate::ensnano_design::{bezier_plane::BezierPathId, curves::torus::CurveDescriptor2D};
use crate::ensnano_gui::left_panel::tabs::revolution_tab::{
    CurveDescriptorBuilder, CurveDescriptorParameter, InstantiatedParameter,
};
use ultraviolet::{Rotor3, Vec3};

pub(super) const ELLIPSE_BUILDER: CurveDescriptorBuilder<AppState> = CurveDescriptorBuilder {
    curve_name: "Ellipse",
    parameters: &[
        CurveDescriptorParameter {
            name: "Semi major axis",
            default_value: InstantiatedParameter::Float(20.0),
        },
        CurveDescriptorParameter {
            name: "Semi minor axis",
            default_value: InstantiatedParameter::Float(10.0),
        },
    ],
    build: &build_ellipse,
    bezier_path_id: &no_bezier_path_id,
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

pub(super) const TWO_SPHERES_BUILDER: CurveDescriptorBuilder<AppState> = CurveDescriptorBuilder {
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
    frame: &default_frame,
};

fn build_two_spheres(
    parameters: &[InstantiatedParameter],
    _: &AppState,
) -> Option<CurveDescriptor2D> {
    let radius_extern = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_float)?
        .into();
    let radius_intern = parameters
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
        radius_extern,
        radius_intern,
        radius_tube,
        smooth_ceil,
    })
}

pub(super) const BEZIER_CURVE_BUILDER: CurveDescriptorBuilder<AppState> = CurveDescriptorBuilder {
    curve_name: "Bezier",
    parameters: &[CurveDescriptorParameter {
        name: "Path n°",
        default_value: InstantiatedParameter::Uint(0),
    }],
    build: &build_bezier,
    bezier_path_id: &get_bezier_path_id,
    frame: &get_bezier_frame,
};

fn build_bezier(parameters: &[InstantiatedParameter], app: &AppState) -> Option<CurveDescriptor2D> {
    let curve_id = parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_uint)?;

    app.0
        .design
        .clone_inner()
        .get_bezier_path_2d(BezierPathId(curve_id as u32))
        .map(CurveDescriptor2D::Bezier)
}

fn no_bezier_path_id(_: &[InstantiatedParameter]) -> Option<usize> {
    None
}

fn get_bezier_path_id(parameters: &[InstantiatedParameter]) -> Option<usize> {
    parameters
        .first()
        .copied()
        .and_then(InstantiatedParameter::get_uint)
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
