use crate::{
    AppState,
    consts::{MAX_NB_TURN, MIN_NB_TURN, NB_TURN_SLIDER_SPACING, NB_TURN_STEP},
    left_panel::Message,
};
use ensnano_iced::{ui_size::UiSize, widgets::keyboard_priority::keyboard_priority};
use ensnano_interactor::selection::Selection;
use iced::{
    Alignment, Length,
    widget::{Column, Space, column, row, slider, text, text_input},
};
use paste::paste;
use ultraviolet::{Bivec3, Mat3, Rotor3, Vec2, Vec3};

macro_rules! type_builder {
    ($builder_name:ident, $initializer:tt, $internal:tt, $convert_in:path, $convert_out:path, $($param: ident: $param_type: tt %$formatter:path), *) => {
        paste! {
            pub(crate) struct $builder_name {
                $(
                    #[expect(clippy::allow_attributes)]
                    #[allow(dead_code)]
                    $param: $param_type,
                    [<$param _string>]: String,
                )*
                    value_to_modify: ValueKind,
            }

            impl $builder_name {
                const PARAMETER_NAMES: &'static [&'static str] = &[$(stringify!($param),)*];
                pub(crate) fn new(value_to_modify: ValueKind, initial_value: $initializer) -> Self {
                    let initial: $internal = $convert_in(initial_value);
                    Self {
                        value_to_modify,
                        $(
                            $param: initial.$param,
                            [<$param _string>]: $formatter::fmt(&initial.$param),
                        )*
                    }

                }
                fn update_str_value(&mut self, n: usize, value_str: String) {
                    let mut refs = [$(&mut self.[<$param _string>],)*];
                    if let Some(val) = refs.get_mut(n) {
                        **val = value_str;
                    }
                }

                fn view<State: AppState>(&self) -> iced::Element<'_, Message<State>> {
                    let str_values = [$(& self.[<$param _string>],)*];
                    let mut ret = Column::new().width(Length::Fill).align_items(Alignment::End);
                    let value_to_modify = self.value_to_modify;
                    for i in 0..Self::PARAMETER_NAMES.len() {
                        ret = ret.push(row![
                            text(Self::PARAMETER_NAMES[i]),
                            Space::with_width(5),
                            keyboard_priority(
                                "Contextual value change priority",
                                Message::SetKeyboardPriority,
                                text_input("", str_values[i])
                                    .on_input(move |string| Message::ContextualValueChanged(value_to_modify, i, string))
                                    .on_submit(Message::ContextualValueSubmitted(value_to_modify))
                                    .width(50)
                            )
                        ].width(Length::Fill))
                    }
                    ret.into()
                }

                fn submit_value(&mut self) -> Option<$initializer> {
                    $(
                        let $param = $formatter::parse(&self.[<$param _string>])?;
                    )*
                    let out: $internal = $internal {
                        $(
                            $param,
                        )*
                    };

                    Some($convert_out(out))
                }
            }
        }
    }
}

struct DegreeAngleFormatter;

impl DegreeAngleFormatter {
    fn fmt(angle: &f32) -> String {
        format!("{:.1}°", angle.to_degrees())
    }

    fn parse(angle_str: &str) -> Option<f32> {
        angle_str
            .trim_end_matches('°')
            .parse::<f32>()
            .ok()
            .map(f32::to_radians)
    }
}

struct FloatFormatter;

impl FloatFormatter {
    fn fmt(float: &f32) -> String {
        format!("{float:.2}")
    }

    fn parse(float_str: &str) -> Option<f32> {
        float_str.parse::<f32>().ok()
    }
}

type_builder!(
    Vec3Builder,
    Vec3,
    Vec3,
    std::convert::identity,
    std::convert::identity,
    x: f32 % FloatFormatter,
    y: f32 % FloatFormatter,
    z: f32 % FloatFormatter
);

type_builder!(
    Vec2Builder,
    Vec2,
    Vec2,
    std::convert::identity,
    std::convert::identity,
    x: f32 % FloatFormatter,
    y: f32 % FloatFormatter
);

type_builder!(
    DirectionAngleBuilder,
    Rotor3,
    DirectionAngle,
    DirectionAngle::from_rotor,
    DirectionAngle::to_rotor,
    x: f32 % FloatFormatter,
    y: f32 % FloatFormatter,
    z: f32 % FloatFormatter,
    angle: f32 % DegreeAngleFormatter
);

#[derive(Clone, Copy, Debug)]
pub enum ValueKind {
    HelixGridPosition,
    GridOrientation,
    BezierVertexPosition,
}

#[derive(Debug, Clone)]
pub enum InstantiatedValue {
    HelixGridPosition(Vec3),
    GridOrientation(Rotor3),
    GridNbTurn(f32),
    BezierVertexPosition(Vec2),
}

pub(crate) enum GridPositionBuilder {
    Cartesian(Vec3Builder),
}

impl GridPositionBuilder {
    pub(crate) fn new_cartesian(position: Vec3) -> Self {
        Self::Cartesian(Vec3Builder::new(ValueKind::HelixGridPosition, position))
    }

    fn view<State: AppState>(&self) -> iced::Element<'_, Message<State>> {
        match self {
            Self::Cartesian(builder) => builder.view(),
        }
    }

    fn update_str_value(&mut self, n: usize, value_str: String) {
        match self {
            Self::Cartesian(builder) => builder.update_str_value(n, value_str),
        }
    }

    fn submit_value(&mut self) -> Option<InstantiatedValue> {
        match self {
            Self::Cartesian(builder) => builder
                .submit_value()
                .map(InstantiatedValue::HelixGridPosition),
        }
    }
}

pub(crate) enum GridOrientationBuilder {
    DirectionAngle(DirectionAngleBuilder),
}

impl GridOrientationBuilder {
    pub(crate) fn new_direction_angle(orientation: Rotor3) -> Self {
        Self::DirectionAngle(DirectionAngleBuilder::new(
            ValueKind::GridOrientation,
            orientation,
        ))
    }

    fn view<State: AppState>(&self) -> iced::Element<'_, Message<State>> {
        match self {
            Self::DirectionAngle(builder) => builder.view(),
        }
    }

    fn update_str_value(&mut self, n: usize, value_str: String) {
        match self {
            Self::DirectionAngle(builder) => builder.update_str_value(n, value_str),
        }
    }

    fn submit_value(&mut self) -> Option<InstantiatedValue> {
        match self {
            Self::DirectionAngle(builder) => builder
                .submit_value()
                .map(InstantiatedValue::GridOrientation),
        }
    }
}

pub(crate) struct BezierVertexBuilder {
    position_builder: Vec2Builder,
}

impl BezierVertexBuilder {
    pub(crate) fn new(position: Vec2) -> Self {
        Self {
            position_builder: Vec2Builder::new(ValueKind::BezierVertexPosition, position),
        }
    }
}

impl<State> Builder<State> for BezierVertexBuilder
where
    State: AppState,
{
    fn view(
        &self,
        ui_size: UiSize,
        _selection: &Selection,
        _app_state: &State,
    ) -> iced::Element<'_, Message<State>> {
        self::column![
            text("Position").size(ui_size.intermediate_text()),
            self.position_builder.view(),
        ]
        .width(Length::Fill)
        .into()
    }

    fn update_str_value(&mut self, value_kind: ValueKind, n: usize, value_str: String) {
        if matches!(value_kind, ValueKind::BezierVertexPosition) {
            self.position_builder.update_str_value(n, value_str);
        } else {
            log::error!("Unexpected value kind {value_kind:?} for BezierVertexBuilder",);
        }
    }

    fn submit_value(&mut self, value_kind: ValueKind) -> Option<InstantiatedValue> {
        if matches!(value_kind, ValueKind::BezierVertexPosition) {
            self.position_builder
                .submit_value()
                .map(InstantiatedValue::BezierVertexPosition)
        } else {
            log::error!("Unexpected value kind {value_kind:?} for BezierVertexBuilder",);
            None
        }
    }
}

pub(crate) struct GridBuilder {
    position_builder: GridPositionBuilder,
    orientation_builder: GridOrientationBuilder,
}

impl GridBuilder {
    pub(crate) fn new(position: Vec3, orientation: Rotor3) -> Self {
        Self {
            position_builder: GridPositionBuilder::new_cartesian(position),
            orientation_builder: GridOrientationBuilder::new_direction_angle(orientation),
        }
    }

    fn nb_turn_row<'a, S: AppState>(
        app_state: &S,
        selection: &Selection,
    ) -> Option<iced::Element<'a, Message<S>>> {
        if let Selection::Grid(_, g_id) = selection {
            if let Some(nb_turn) = app_state.get_reader().get_grid_nb_turn(*g_id) {
                let row = row![
                    text(format!("{nb_turn:.2}")),
                    slider(MIN_NB_TURN..=MAX_NB_TURN, nb_turn, |x| {
                        Message::InstantiatedValueSubmitted(InstantiatedValue::GridNbTurn(x))
                    })
                    .step(NB_TURN_STEP),
                ]
                .spacing(NB_TURN_SLIDER_SPACING);
                Some(row.into())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<State> Builder<State> for GridBuilder
where
    State: AppState,
{
    fn view(
        &self,
        ui_size: UiSize,
        selection: &Selection,
        app_state: &State,
    ) -> iced::Element<'_, Message<State>> {
        self::column![
            text("Position").size(ui_size.intermediate_text()),
            self.position_builder.view(),
            text("Orientation").size(ui_size.intermediate_text()),
            self.orientation_builder.view(),
            text("Twist").size(ui_size.intermediate_text()),
            if let Some(row) = Self::nb_turn_row(app_state, selection) {
                row
            } else {
                row![].into()
            },
        ]
        .width(Length::Fill)
        .into()
    }

    fn update_str_value(&mut self, value_kind: ValueKind, n: usize, value_str: String) {
        match value_kind {
            ValueKind::HelixGridPosition => self.position_builder.update_str_value(n, value_str),
            ValueKind::GridOrientation => self.orientation_builder.update_str_value(n, value_str),
            vk @ ValueKind::BezierVertexPosition => {
                log::error!("Unexpected value kind for GridBuilder {vk:?}");
            }
        }
    }

    fn submit_value(&mut self, value_kind: ValueKind) -> Option<InstantiatedValue> {
        match value_kind {
            ValueKind::HelixGridPosition => self.position_builder.submit_value(),
            ValueKind::GridOrientation => self.orientation_builder.submit_value(),
            vk @ ValueKind::BezierVertexPosition => {
                log::error!("Unexpected value kind for GridBuilder {vk:?}");
                None
            }
        }
    }
}

pub(crate) trait Builder<State>
where
    State: AppState,
{
    fn view<'a>(
        &'a self,
        ui_size: UiSize,
        selection: &Selection,
        app_state: &State,
    ) -> iced::Element<'a, Message<State>>;
    fn update_str_value(&mut self, value_kind: ValueKind, n: usize, value_str: String);
    fn submit_value(&mut self, value_kind: ValueKind) -> Option<InstantiatedValue>;
}

#[derive(Debug, Clone, Copy)]
struct DirectionAngle {
    x: f32,
    y: f32,
    z: f32,
    angle: f32,
}

impl DirectionAngle {
    const CONVERSION_EPSILON: f32 = 1e-6;

    fn from_rotor(rotor: Rotor3) -> Self {
        let direction = Vec3::unit_x().rotated_by(rotor);
        log::info!("direction {direction:?}");

        let real_z = Self::real_z(direction);
        log::info!("real z {real_z:?}");
        let real_y = real_z.cross(direction);
        log::info!("real y {real_y:?}");

        let cos_angle = Vec3::unit_z().rotated_by(rotor).dot(real_z);
        let sin_angle = -Vec3::unit_z().rotated_by(rotor).dot(real_y);
        log::info!("cos = {cos_angle}, sin = {sin_angle}");
        let angle = sin_angle.atan2(cos_angle);

        Self {
            x: direction.x,
            y: direction.y,
            z: direction.z,
            angle,
        }
    }

    fn to_rotor(self) -> Rotor3 {
        let direction = Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
        .normalized();

        let angle = self.angle;
        let real_z = Self::real_z(direction);
        log::info!("real z {real_z:?}");
        let z = real_z.rotated_by(Rotor3::from_angle_plane(
            angle,
            Bivec3::from_normalized_axis(direction),
        ));
        let y = z.cross(direction);
        log::info!(" x {direction:?}");
        log::info!(" y {y:?}");
        log::info!(" z {real_z:?}");

        Mat3::new(direction, y, z).into_rotor3()
    }

    fn real_z(direction: Vec3) -> Vec3 {
        let z_angle = direction.y.asin();
        log::info!("z angle {}", z_angle.to_degrees());

        if direction.y.abs() < 1. - Self::CONVERSION_EPSILON {
            let radius = z_angle.cos();
            log::info!("radius {radius}");
            log::info!("direction.x / radius {}", direction.x / radius);
            let y_angle = if direction.z > 0. {
                -(direction.x / radius).clamp(-1., 1.).acos()
            } else {
                (direction.x / radius).clamp(-1., 1.).acos()
            };
            log::info!("y angle {}", y_angle.to_degrees());

            Vec3::unit_z().rotated_by(Rotor3::from_angle_plane(
                y_angle,
                Bivec3::from_normalized_axis(Vec3::unit_y()),
            ))
        } else {
            Vec3::unit_z()
        }
    }
}
