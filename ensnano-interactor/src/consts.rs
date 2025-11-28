use ensnano_design::curves::bezier::{BezierControlPoint, CubicBezierControlPoint};
use crate::{RevolutionSimulationParameters, surfaces::EquadiffSolvingMethod};
use ensnano_consts::{
    BEZIER_CONTROL1_COLOR, BEZIER_CONTROL2_COLOR, BEZIER_END_COLOR, BEZIER_END_WIDGET_ID,
    BEZIER_START_COLOR, BEZIER_START_WIDGET_ID, PIECEWISE_BEZIER_COLOR,
};

pub fn bezier_widget_id(helix_id: u32, control_point: BezierControlPoint) -> u32 {
    let bezier_id = bezier_control_id(control_point);
    (helix_id << 8) | bezier_id
}

pub fn widget_id_to_bezier(id: u32) -> Option<(usize, BezierControlPoint)> {
    let control = match id & 0xFF {
        n if n > BEZIER_END_WIDGET_ID => Some(BezierControlPoint::PiecewiseBezier(
            (n - 1 - BEZIER_END_WIDGET_ID) as usize,
        )),
        n => {
            let control = ((n - BEZIER_START_WIDGET_ID) as usize).try_into().ok();
            control.map(BezierControlPoint::CubicBezier)
        }
    };
    Some((id >> 8) as usize).zip(control)
}

pub const fn bezier_control_color(control_point: BezierControlPoint) -> u32 {
    match control_point {
        BezierControlPoint::CubicBezier(CubicBezierControlPoint::Start) => BEZIER_START_COLOR,
        BezierControlPoint::CubicBezier(CubicBezierControlPoint::Control1) => BEZIER_CONTROL1_COLOR,
        BezierControlPoint::CubicBezier(CubicBezierControlPoint::Control2) => BEZIER_CONTROL2_COLOR,
        BezierControlPoint::CubicBezier(CubicBezierControlPoint::End) => BEZIER_END_COLOR,
        BezierControlPoint::PiecewiseBezier(_) => PIECEWISE_BEZIER_COLOR,
    }
}

pub fn bezier_control_id(control_point: BezierControlPoint) -> u32 {
    match control_point {
        BezierControlPoint::CubicBezier(c) => {
            let control_id: usize = c.into();
            BEZIER_START_WIDGET_ID + control_id as u32
        }
        BezierControlPoint::PiecewiseBezier(n) => n as u32 + BEZIER_END_WIDGET_ID + 1,
    }
}

pub const BASE_SCROLL_SENSITIVITY: f32 = 0.12;

pub fn scroll_sensitivity_conversion(sensitivity: f32) -> f32 {
    10f32.powf(sensitivity / 10.) * BASE_SCROLL_SENSITIVITY
}

pub const DEFAULT_REVOLUTION_SIMULATION_PARAMETERS: RevolutionSimulationParameters =
    RevolutionSimulationParameters {
        nb_section_per_segment: 100,
        spring_stiffness: 8.0,
        torsion_stiffness: 30.0,
        fluid_friction: 1.0,
        ball_mass: 10.0,
        time_span: 5.0e-2,
        simulation_step: 1e-3,
        method: EquadiffSolvingMethod::Ralston,
    };
