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

use crate::helpers::*;
use crate::left_panel::Message;
use crate::theme;
use crate::{AppState, SimulationState, UiSize};
use ensnano_design::{
    ultraviolet::{self, Rotor3, Vec3},
    CurveDescriptor2D,
};
use ensnano_interactor::{
    EquadiffSolvingMethod, RevolutionSimulationParameters, RevolutionSurfaceRadius,
    RevolutionSurfaceSystemDescriptor, RootingParameters, ShiftGenerator,
    UnrootedRevolutionSurfaceDescriptor,
};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, Copy)]
pub enum ParameterKind {
    Float,
    Int,
    Uint,
}

#[derive(Debug, Clone, Copy)]
pub enum InstanciatedParameter {
    Float(f64),
    Int(isize),
    Uint(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum RevolutionParameterId {
    SectionParameter(usize),
    HalfTurnCount,
    RevolutionRadius,
    NbSpiral,
    NbSectionPerSegment,
    ScaffoldLenTarget,
    SpringStiffness,
    TorsionStiffness,
    FluidFriction,
    BallMass,
    TimeSpan,
    SimulationStep,
}

impl InstanciatedParameter {
    pub fn get_float(self) -> Option<f64> {
        if let Self::Float(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn get_int(self) -> Option<isize> {
        if let Self::Int(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn get_uint(self) -> Option<usize> {
        if let Self::Uint(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CurveDescriptorParameter {
    pub name: &'static str,
    pub kind: ParameterKind,
    pub default_value: InstanciatedParameter,
}

pub type Frame = (ultraviolet::Vec3, ultraviolet::Rotor3);
#[derive(Clone)]
pub struct CurveDescriptorBuilder<S: AppState> {
    pub nb_parameters: usize,
    pub curve_name: &'static str,
    pub parameters: &'static [CurveDescriptorParameter],
    pub bezier_path_id: &'static (dyn Fn(&[InstanciatedParameter]) -> Option<usize> + Send + Sync),
    pub build:
        &'static (dyn Fn(&[InstanciatedParameter], &S) -> Option<CurveDescriptor2D> + Send + Sync),
    pub frame: &'static (dyn Fn(&[InstanciatedParameter], &S) -> Option<Frame> + Send + Sync),
}

use std::fmt;
impl<S: AppState> fmt::Debug for CurveDescriptorBuilder<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CurveDecriptorBuilder")
            .field("curve_name", &self.curve_name)
            .finish()
    }
}

impl<S: AppState> ToString for CurveDescriptorBuilder<S> {
    fn to_string(&self) -> String {
        self.curve_name.to_string()
    }
}

impl<S: AppState> PartialEq for CurveDescriptorBuilder<S> {
    fn eq(&self, other: &Self) -> bool {
        self.curve_name == other.curve_name
    }
}

impl<S: AppState> Eq for CurveDescriptorBuilder<S> {}

struct ParameterWidget {
    current_text: String,
    state: text_input::State<iced_graphics::text::Paragraph>,
    parameter_kind: ParameterKind,
}

impl ParameterWidget {
    fn new(initial_value: InstanciatedParameter) -> Self {
        let (current_text, parameter_kind) = match initial_value {
            InstanciatedParameter::Float(x) => (format!("{:.3}", x), ParameterKind::Float),
            InstanciatedParameter::Int(x) => (x.to_string(), ParameterKind::Int),
            InstanciatedParameter::Uint(x) => (x.to_string(), ParameterKind::Uint),
        };
        Self {
            current_text,
            state: Default::default(),
            parameter_kind,
        }
    }

    fn input_view<S: AppState>(&self, id: RevolutionParameterId) -> Element<Message<S>> {
        let style = theme::BadValue(self.contains_valid_input());
        text_input("", &self.current_text)
            .on_input(move |s| Message::RevolutionParameterUpdate {
                parameter_id: id,
                text: s,
            })
            .width(50)
            .style(style)
            .into()
    }

    fn set_text(&mut self, text: String) {
        self.current_text = text;
    }

    fn has_keyboard_priority(&self) -> bool {
        self.state.is_focused()
    }

    fn contains_valid_input(&self) -> bool {
        self.get_value().is_some()
    }

    fn get_value(&self) -> Option<InstanciatedParameter> {
        match self.parameter_kind {
            ParameterKind::Float => self
                .current_text
                .parse::<f64>()
                .ok()
                .map(InstanciatedParameter::Float),
            ParameterKind::Int => self
                .current_text
                .parse::<isize>()
                .ok()
                .map(InstanciatedParameter::Int),
            ParameterKind::Uint => self
                .current_text
                .parse::<usize>()
                .ok()
                .map(InstanciatedParameter::Uint),
        }
    }
}

struct CurveDescriptorWidget<S: AppState> {
    parameters: Vec<(&'static str, ParameterWidget)>,
    curve_name: &'static str,
    builder: CurveDescriptorBuilder<S>,
}

impl<S: AppState> CurveDescriptorWidget<S> {
    fn new(builder: CurveDescriptorBuilder<S>) -> Self {
        let parameters = builder
            .parameters
            .iter()
            .map(|builder| (builder.name, ParameterWidget::new(builder.default_value)))
            .collect();

        Self {
            parameters,
            curve_name: builder.curve_name,
            builder,
        }
    }

    fn view(&self, ui_size: UiSize) -> Element<Message<S>, crate::Theme, crate::Renderer> {
        container(column(
            self.parameters
                .iter()
                .enumerate()
                .map(|(param_id, param)| {
                    row![
                        Space::with_width(ui_size.checkbox_spacing()),
                        text(param.0),
                        Space::with_width(ui_size.checkbox_spacing()),
                        //param
                        //    .1
                        //    .input_view(RevolutionParameterId::SectionParameter(param_id))
                        //TODO: REACTIVATE ME!
                    ]
                    .align_items(Alignment::Center)
                    .into()
                })
                .collect::<Vec<_>>(),
        ))
        .into()
    }

    fn update_builder_parameter(&mut self, param_id: usize, text: String) {
        if let Some(p) = self.parameters.get_mut(param_id) {
            p.1.set_text(text)
        }
    }

    fn has_keyboard_priority(&self) -> bool {
        self.parameters
            .iter()
            .any(|(_, p)| p.has_keyboard_priority())
    }

    fn instanciated_parameters(&self) -> Vec<InstanciatedParameter> {
        self.parameters
            .iter()
            .filter_map(|p| p.1.get_value())
            .collect()
    }

    fn build_curve(&self, app_state: &S) -> Option<CurveDescriptor2D> {
        (self.builder.build)(&self.instanciated_parameters(), app_state)
    }

    fn get_bezier_path_id(&self) -> Option<usize> {
        (self.builder.bezier_path_id)(&self.instanciated_parameters())
    }

    fn get_frame(&self, app_state: &S) -> Option<Frame> {
        (self.builder.frame)(&self.instanciated_parameters(), app_state)
    }
}

pub(crate) struct RevolutionTab<S: AppState> {
    curve_descriptor_widget: Option<CurveDescriptorWidget<S>>,
    half_turn_count: ParameterWidget,
    radius_input: ParameterWidget,
    scaling: Option<RevolutionScaling>,
    nb_sprial_state_input: ParameterWidget,
    shift_generator: Option<ShiftGenerator>,
    pub shift_idx: isize,
    scaffold_len_target: ParameterWidget,

    nb_section_per_segment_input: ParameterWidget,
    spring_stiffness: ParameterWidget,
    torsion_stiffness: ParameterWidget,
    fluid_friction: ParameterWidget,
    ball_mass: ParameterWidget,
    time_span: ParameterWidget,
    simulation_step: ParameterWidget,
    equadiff_method: EquadiffSolvingMethod,
}

impl<S: AppState> Default for RevolutionTab<S> {
    fn default() -> Self {
        let init_parameter = RevolutionSimulationParameters::default();
        Self {
            curve_descriptor_widget: None,
            half_turn_count: ParameterWidget::new(InstanciatedParameter::Int(0)),
            radius_input: ParameterWidget::new(InstanciatedParameter::Float(0.)),
            scaling: None,
            nb_sprial_state_input: ParameterWidget::new(InstanciatedParameter::Uint(2)),
            shift_generator: None,
            shift_idx: 0,
            nb_section_per_segment_input: ParameterWidget::new(InstanciatedParameter::Uint(
                init_parameter.nb_section_per_segment,
            )),
            spring_stiffness: ParameterWidget::new(InstanciatedParameter::Float(
                init_parameter.spring_stiffness,
            )),
            torsion_stiffness: ParameterWidget::new(InstanciatedParameter::Float(
                init_parameter.torsion_stiffness,
            )),
            fluid_friction: ParameterWidget::new(InstanciatedParameter::Float(
                init_parameter.fluid_friction,
            )),
            ball_mass: ParameterWidget::new(InstanciatedParameter::Float(init_parameter.ball_mass)),
            time_span: ParameterWidget::new(InstanciatedParameter::Float(init_parameter.time_span)),
            simulation_step: ParameterWidget::new(InstanciatedParameter::Float(
                init_parameter.simulation_step,
            )),
            equadiff_method: init_parameter.method,
            scaffold_len_target: ParameterWidget::new(InstanciatedParameter::Uint(7249)),
        }
    }
}

impl<S: AppState> RevolutionTab<S> {
    pub fn set_builder(&mut self, builder: CurveDescriptorBuilder<S>) {
        if self.curve_descriptor_widget.as_ref().map(|w| w.curve_name) != Some(builder.curve_name) {
            self.curve_descriptor_widget = Some(CurveDescriptorWidget::new(builder))
        }
    }

    pub fn set_method(&mut self, method: EquadiffSolvingMethod) {
        self.equadiff_method = method;
    }

    pub fn get_current_bezier_path_id(&self) -> Option<usize> {
        self.curve_descriptor_widget
            .as_ref()
            .and_then(|w| w.get_bezier_path_id())
    }

    pub fn update_builder_parameter(&mut self, param_id: RevolutionParameterId, text: String) {
        match param_id {
            RevolutionParameterId::SectionParameter(id) => {
                if let Some(widget) = self.curve_descriptor_widget.as_mut() {
                    widget.update_builder_parameter(id, text)
                }
            }
            param => {
                use RevolutionParameterId::*;
                let widget = match param {
                    SectionParameter(_) => unreachable!(),
                    HalfTurnCount => &mut self.half_turn_count,
                    NbSpiral => &mut self.nb_sprial_state_input,
                    RevolutionRadius => &mut self.radius_input,
                    ScaffoldLenTarget => &mut self.scaffold_len_target,
                    NbSectionPerSegment => &mut self.nb_section_per_segment_input,
                    SpringStiffness => &mut self.spring_stiffness,
                    TorsionStiffness => &mut self.torsion_stiffness,
                    FluidFriction => &mut self.fluid_friction,
                    BallMass => &mut self.ball_mass,
                    TimeSpan => &mut self.time_span,
                    SimulationStep => &mut self.simulation_step,
                };
                widget.set_text(text);
            }
        }
    }

    pub fn get_current_unrooted_surface(
        &self,
        app_state: &S,
    ) -> Option<UnrootedRevolutionSurfaceDescriptor> {
        let curve = self
            .curve_descriptor_widget
            .as_ref()
            .and_then(|w| w.build_curve(app_state))?;
        let revolution_radius = self
            .radius_input
            .get_value()
            .and_then(InstanciatedParameter::get_float)
            .map(RevolutionSurfaceRadius::from_signed_f64)?;
        let half_turn_count = self
            .half_turn_count
            .get_value()
            .and_then(InstanciatedParameter::get_int)?;

        let (curve_plane_position, curve_plane_orientation) = self
            .curve_descriptor_widget
            .as_ref()
            .and_then(|w| w.get_frame(app_state))
            .unwrap_or_else(|| (Vec3::zero(), Rotor3::identity()));

        Some(UnrootedRevolutionSurfaceDescriptor {
            curve,
            revolution_radius,
            half_turn_count,
            curve_plane_orientation,
            curve_plane_position,
        })
    }

    pub fn view(
        &self,
        ui_size: UiSize,
        app_state: &S,
    ) -> iced::Element<Message<S>, crate::Theme, crate::Renderer> {
        let desc = self.get_revolution_system(app_state, false);

        let shift_buttons = {
            let buttons = (button(text("-")), button(text("+")));
            if let Some(shift) = self.get_shift_per_turn(app_state) {
                row![
                    buttons.0.on_press(Message::DecrRevolutionShift),
                    buttons.1.on_press(Message::DecrRevolutionShift),
                    Space::with_width(ui_size.checkbox_spacing()),
                    text(format!("Nb shift: {shift}")),
                ]
            } else {
                row![
                    buttons.0,
                    buttons.1,
                    Space::with_width(ui_size.checkbox_spacing()),
                    text("Nb shift: ###"),
                ]
            }
        };

        let simulation_buttons = if let SimulationState::Relaxing = app_state.get_simulation_state()
        {
            self::column![
                button(text("Abort")).on_press(Message::StopSimulation),
                jump_by(2),
                text(
                    app_state
                        .get_reader()
                        .get_current_length_of_relaxed_shape()
                        .map_or("".into(), |l| format!("Current total length: {l}"))
                ),
                button(text("Finish")).on_press(Message::FinishRelaxation),
            ]
        } else {
            let mut button = button(text("Start"));
            if let SimulationState::None = app_state.get_simulation_state() {
                if desc.is_some() {
                    button = button.on_press(Message::InitRevolutionRelaxation);
                }
            }
            self::column![button]
        };

        let content = self::column![
            section("Revolution Surfaces", ui_size),
            checkbox("Show bezier paths", app_state.get_show_bezier_paths())
                .on_toggle(Message::SetShowBezierPaths),
            self::column![
                extra_jump(),
                subsection("Section parameters", ui_size),
                row![
                    text("Curve type"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    pick_list(
                        S::POSSIBLE_CURVES,
                        self.curve_descriptor_widget
                            .as_ref()
                            .map(|w| w.builder.clone()),
                        Message::CurveBuilderPicked,
                    )
                    .placeholder("Pick.."),
                ]
                .align_items(Alignment::Center),
                if let Some(widget) = &self.curve_descriptor_widget {
                    widget.view(ui_size)
                } else {
                    self::column![].into()
                },
            ],
            self::column![
                extra_jump(),
                subsection("Revolution parameters", ui_size),
                row![
                    text("Nb Half Turns"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.half_turn_count
                    //    .input_view(RevolutionParameterId::HalfTurnCount),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                text(self.scaling.map_or(
                    "Nb helix: ###".into(),
                    |RevolutionScaling { nb_helix, .. }| format!("Nb helix: {nb_helix}")
                )),
                row![
                    text("Nb spiral"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.nb_sprial_state_input
                    //    .input_view(RevolutionParameterId::NbSpiral),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                shift_buttons,
                row![
                    text("Revolution Radius"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.radius_input
                    //    .input_view(RevolutionParameterId::RevolutionRadius),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
            ]
            .spacing(2),
            self::column![
                extra_jump(),
                subsection("Discretization parameters", ui_size),
                row![
                    text("Nb section per segments"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.nb_section_per_segment_input
                    //    .input_view(RevolutionParameterId::NbSectionPerSegment),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                row![
                    text("Target length"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.scaffold_len_target
                    //    .input_view(RevolutionParameterId::ScaffoldLenTarget),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
            ]
            .spacing(2),
            self::column![
                extra_jump(),
                subsection("Simulation parameters", ui_size),
                row![
                    text("Spring Stiffness"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.spring_stiffness
                    //    .input_view(RevolutionParameterId::SpringStiffness),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                row![
                    text("Torsion Stiffness"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.torsion_stiffness
                    //    .input_view(RevolutionParameterId::TorsionStiffness),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                row![
                    text("Fluid Friction"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.fluid_friction
                    //    .input_view(RevolutionParameterId::FluidFriction),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                row![
                    text("Ball Mass"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.ball_mass.input_view(RevolutionParameterId::BallMass),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                row![
                    text("Solving Method"),
                    pick_list(
                        EquadiffSolvingMethod::ALL_METHODS,
                        Some(self.equadiff_method),
                        Message::RevolutionEquadiffSolvingMethodPicked,
                    ),
                ]
                .align_items(Alignment::Center),
                row![
                    text("Tie Span"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.time_span.input_view(RevolutionParameterId::TimeSpan),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
                row![
                    text("Simulation Step"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    //self.simulation_step
                    //    .input_view(RevolutionParameterId::SimulationStep),
                    //TODO: REACTIVATE ME!
                ]
                .align_items(Alignment::Center),
            ]
            .spacing(2),
            self::column![
                extra_jump(),
                section("Relaxation computation", ui_size),
                simulation_buttons,
            ],
        ]
        .spacing(5);

        scrollable(content).width(Length::Fill).into()

        //let mut ret = widget::scrollable::Scrollable::new(&mut self.scroll_state);

        //let shift_txt = if let Some(shift) = nb_shift {
        //    format!("Nb shift: {shift}")
        //} else {
        //    "Nb shift: ###".into()
        //};
        //let mut button_incr = Button::new(Text::new("+"));
        //let mut button_decr = Button::new(Text::new("-"));
        //if nb_shift.is_some() {
        //    button_decr = button_decr.on_press(Message::DecrRevolutionShift);
        //    button_incr = button_incr.on_press(Message::IncrRevolutionShift);
        //}
        //ret = ret.push(
        //    Row::new()
        //        .push(button_decr)
        //        .push(button_incr)
        //        .push(Text::new(shift_txt)),
        //);
        //if let SimulationState::Relaxing = app_state.get_simulation_state() {
        //    let button_abbort = Button::new(Text::new("Abort")).on_press(Message::StopSimulation);
        //    ret = ret.push(button_abbort);
        //    extra_jump!(2, ret);
        //    if let Some(len) = app_state.get_reader().get_current_length_of_relaxed_shape() {
        //        ret = ret.push(Text::new(format!("Current total length: {len}")));
        //    }
        //    let button_relaxation =
        //        Button::new(Text::new("Finish")).on_press(Message::FinishRelaxation);
        //    ret = ret.push(button_relaxation);
        //} else {
        //    let mut button = Button::new(Text::new("Start"));
        //    if let SimulationState::None = app_state.get_simulation_state() {
        //        if desc.is_some() {
        //            button = button.on_press(Message::InitRevolutionRelaxation);
        //        }
        //    }
        //    ret = ret.push(button);
        //}
        //ret.into()
    }

    pub fn has_keyboard_priority(&self) -> bool {
        self.curve_descriptor_widget
            .as_ref()
            .map(CurveDescriptorWidget::has_keyboard_priority)
            .unwrap_or(false)
            || self.radius_input.has_keyboard_priority()
            || self.nb_section_per_segment_input.has_keyboard_priority()
            || self.half_turn_count.has_keyboard_priority()
            || self.scaffold_len_target.has_keyboard_priority()
            || self.nb_sprial_state_input.has_keyboard_priority()
            || self.spring_stiffness.has_keyboard_priority()
            || self.torsion_stiffness.has_keyboard_priority()
            || self.fluid_friction.has_keyboard_priority()
            || self.ball_mass.has_keyboard_priority()
            || self.time_span.has_keyboard_priority()
            || self.simulation_step.has_keyboard_priority()
    }

    pub fn get_revolution_system(
        &self,
        app_state: &S,
        compute_area: bool,
    ) -> Option<RevolutionSurfaceSystemDescriptor> {
        let unrooted_surface = self.get_current_unrooted_surface(app_state)?;

        let rooting_parameters = RootingParameters {
            dna_parameters: app_state.get_dna_parameters(),
            nb_helix_per_half_section: self.scaling.as_ref()?.nb_helix / 2,
            shift_per_turn: self.try_get_shift_per_turn(app_state)?,
            junction_smoothening: 0.,
        };

        let surface_descriptor = unrooted_surface.rooted(rooting_parameters, compute_area);

        let simulation_parameters = self.get_simulation_parameters()?;

        let system = RevolutionSurfaceSystemDescriptor {
            target: surface_descriptor,
            scaffold_len_target: self
                .scaffold_len_target
                .get_value()
                .and_then(InstanciatedParameter::get_uint)?,
            helix_parameters: app_state.get_dna_parameters(),
            simulation_parameters,
        };

        Some(system)
    }

    /// Get the number of shift per turn, updating `self.shift_generator` if needed.
    fn get_shift_per_turn(&self, app_state: &S) -> Option<isize> {
        self.try_get_shift_per_turn(app_state).or_else(|| {
            // TODO: This update must be done elsewhere.
            //let unrooted_surface = self.get_current_unrooted_surface(app_state)?;
            //let nb_spiral = self
            //    .nb_sprial_state_input
            //    .get_value()
            //    .and_then(InstanciatedParameter::get_uint)?;
            //let half_nb_helix = self.scaling.as_ref()?.nb_helix / 2;
            //self.shift_generator =
            //    unrooted_surface.shifts_to_get_n_spirals(half_nb_helix, nb_spiral);
            self.try_get_shift_per_turn(app_state)
        })
    }

    /// Return the number of shift per turn if `self.shift_generator` if up-to-date, and `None`
    /// otherwise.
    fn try_get_shift_per_turn(&self, app_state: &S) -> Option<isize> {
        let unrooted_surface = self.get_current_unrooted_surface(app_state)?;
        let nb_spiral = self
            .nb_sprial_state_input
            .get_value()
            .and_then(InstanciatedParameter::get_uint)?;
        let half_nb_helix = self.scaling.as_ref()?.nb_helix / 2;
        self.shift_generator
            .as_ref()
            .and_then(|g| g.ith_value(self.shift_idx, nb_spiral, &unrooted_surface, half_nb_helix))
    }

    fn get_simulation_parameters(&self) -> Option<RevolutionSimulationParameters> {
        let nb_section_per_segment = self
            .nb_section_per_segment_input
            .get_value()
            .and_then(InstanciatedParameter::get_uint)?;
        let spring_stiffness = self
            .spring_stiffness
            .get_value()
            .and_then(InstanciatedParameter::get_float)?;
        let torsion_stiffness = self
            .torsion_stiffness
            .get_value()
            .and_then(InstanciatedParameter::get_float)?;
        let fluid_friction = self
            .fluid_friction
            .get_value()
            .and_then(InstanciatedParameter::get_float)?;
        let ball_mass = self
            .ball_mass
            .get_value()
            .and_then(InstanciatedParameter::get_float)?;
        let time_span = self
            .time_span
            .get_value()
            .and_then(InstanciatedParameter::get_float)?;
        let simulation_step = self
            .simulation_step
            .get_value()
            .and_then(InstanciatedParameter::get_float)?;
        let method = self.equadiff_method;

        let rescaling = self.scaling.as_ref()?.scale;

        Some(RevolutionSimulationParameters {
            nb_section_per_segment,
            spring_stiffness,
            torsion_stiffness,
            fluid_friction,
            ball_mass,
            simulation_step,
            time_span,
            method,
            rescaling,
        })
    }

    pub fn modifying_radius(&self) -> bool {
        self.radius_input.state.is_focused()
    }

    pub fn update(&mut self, app_state: &S) {
        if let Some(r) = app_state.get_current_revoultion_radius() {
            if !self.modifying_radius() {
                self.update_builder_parameter(
                    RevolutionParameterId::RevolutionRadius,
                    format!("{:.3}", r),
                )
            }
        }

        self.scaling = self
            .scaffold_len_target
            .get_value()
            .and_then(InstanciatedParameter::get_uint)
            .and_then(|len_scaffold| {
                app_state.get_recommended_scaling_revolution_surface(len_scaffold)
            });
    }
}

#[derive(Clone, Copy)]
pub struct RevolutionScaling {
    pub nb_helix: usize,
    pub scale: f64,
}
