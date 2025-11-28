use crate::{
    AppState, SimulationState,
    left_panel::{Message, tabs::GuiTab},
};
use ensnano_design::curves::torus::CurveDescriptor2D;
use ensnano_iced::{
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{extra_jump, jump_by, section, subsection, text_button},
    theme,
    ui_size::UiSize,
    widgets::keyboard_priority::keyboard_priority,
};
use ensnano_interactor::surfaces::{
    EquadiffSolvingMethod, RevolutionSimulationParameters, RevolutionSurfaceRadius,
    RevolutionSurfaceSystemDescriptor, RootingParameters, ShiftGenerator,
    UnrootedRevolutionSurfaceDescriptor,
};
use iced::{
    Alignment, Command, Length,
    widget::{
        Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
    },
};
use iced_aw::TabLabel;
use std::fmt;
use ultraviolet::{Rotor3, Vec3};

#[derive(Debug, Clone, Copy)]
pub enum ParameterKind {
    Float,
    Int,
    Uint,
}

#[derive(Debug, Clone, Copy)]
pub enum InstantiatedParameter {
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

impl InstantiatedParameter {
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
    pub default_value: InstantiatedParameter,
}

pub type Frame = (Vec3, Rotor3);

#[derive(Clone)]
pub struct CurveDescriptorBuilder<S: AppState> {
    pub curve_name: &'static str,
    pub parameters: &'static [CurveDescriptorParameter],
    pub bezier_path_id: &'static (dyn Fn(&[InstantiatedParameter]) -> Option<usize> + Send + Sync),
    pub build:
        &'static (dyn Fn(&[InstantiatedParameter], &S) -> Option<CurveDescriptor2D> + Send + Sync),
    pub frame: &'static (dyn Fn(&[InstantiatedParameter], &S) -> Option<Frame> + Send + Sync),
}

impl<S: AppState> fmt::Debug for CurveDescriptorBuilder<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CurveDescriptorBuilder")
            .field("curve_name", &self.curve_name)
            .finish()
    }
}

impl<S: AppState> fmt::Display for CurveDescriptorBuilder<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.curve_name)
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
    parameter_kind: ParameterKind,
}

impl ParameterWidget {
    fn new(initial_value: InstantiatedParameter) -> Self {
        let (current_text, parameter_kind) = match initial_value {
            InstantiatedParameter::Float(x) => (format!("{x:.3}"), ParameterKind::Float),
            InstantiatedParameter::Int(x) => (x.to_string(), ParameterKind::Int),
            InstantiatedParameter::Uint(x) => (x.to_string(), ParameterKind::Uint),
        };
        Self {
            current_text,
            parameter_kind,
        }
    }

    fn input_view<State: AppState>(
        &self,
        id: RevolutionParameterId,
    ) -> iced::Element<'_, Message<State>> {
        keyboard_priority(
            text_input("", &self.current_text)
                .on_input(move |s| Message::RevolutionParameterUpdate {
                    parameter_id: id,
                    text: s,
                })
                .width(50)
                .style(theme::BadValue(self.contains_valid_input())),
        )
        .on_priority(Message::SetKeyboardPriority(true))
        .on_unpriority(Message::SetKeyboardPriority(false))
        .into()
    }

    fn set_text(&mut self, text: String) {
        self.current_text = text;
    }

    fn contains_valid_input(&self) -> bool {
        self.get_value().is_some()
    }

    fn get_value(&self) -> Option<InstantiatedParameter> {
        match self.parameter_kind {
            ParameterKind::Float => self
                .current_text
                .parse::<f64>()
                .ok()
                .map(InstantiatedParameter::Float),
            ParameterKind::Int => self
                .current_text
                .parse::<isize>()
                .ok()
                .map(InstantiatedParameter::Int),
            ParameterKind::Uint => self
                .current_text
                .parse::<usize>()
                .ok()
                .map(InstantiatedParameter::Uint),
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

    fn view(&self, ui_size: UiSize) -> iced::Element<'_, Message<S>> {
        container(column(self.parameters.iter().enumerate().map(
            |(param_id, param)| {
                row![
                    Space::with_width(ui_size.checkbox_spacing()),
                    text(param.0),
                    Space::with_width(ui_size.checkbox_spacing()),
                    param
                        .1
                        .input_view(RevolutionParameterId::SectionParameter(param_id))
                ]
                .align_items(Alignment::Center)
                .into()
            },
        )))
        .into()
    }

    fn update_builder_parameter(&mut self, param_id: usize, text: String) {
        if let Some(p) = self.parameters.get_mut(param_id) {
            p.1.set_text(text);
        }
    }

    fn instantiated_parameters(&self) -> Vec<InstantiatedParameter> {
        self.parameters
            .iter()
            .filter_map(|p| p.1.get_value())
            .collect()
    }

    fn build_curve(&self, app_state: &S) -> Option<CurveDescriptor2D> {
        (self.builder.build)(&self.instantiated_parameters(), app_state)
    }

    fn get_bezier_path_id(&self) -> Option<usize> {
        (self.builder.bezier_path_id)(&self.instantiated_parameters())
    }

    fn get_frame(&self, app_state: &S) -> Option<Frame> {
        (self.builder.frame)(&self.instantiated_parameters(), app_state)
    }
}

pub(crate) struct RevolutionTab<State: AppState> {
    curve_descriptor_widget: Option<CurveDescriptorWidget<State>>,
    half_turn_count: ParameterWidget,
    radius_input: ParameterWidget,
    scaling: Option<RevolutionScaling>,
    nb_spiral_state_input: ParameterWidget,
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

impl<State: AppState> Default for RevolutionTab<State> {
    fn default() -> Self {
        let init_parameter = RevolutionSimulationParameters::default();
        Self {
            curve_descriptor_widget: None,
            half_turn_count: ParameterWidget::new(InstantiatedParameter::Int(0)),
            radius_input: ParameterWidget::new(InstantiatedParameter::Float(0.)),
            scaling: None,
            nb_spiral_state_input: ParameterWidget::new(InstantiatedParameter::Uint(2)),
            shift_generator: None,
            shift_idx: 0,
            nb_section_per_segment_input: ParameterWidget::new(InstantiatedParameter::Uint(
                init_parameter.nb_section_per_segment,
            )),
            spring_stiffness: ParameterWidget::new(InstantiatedParameter::Float(
                init_parameter.spring_stiffness,
            )),
            torsion_stiffness: ParameterWidget::new(InstantiatedParameter::Float(
                init_parameter.torsion_stiffness,
            )),
            fluid_friction: ParameterWidget::new(InstantiatedParameter::Float(
                init_parameter.fluid_friction,
            )),
            ball_mass: ParameterWidget::new(InstantiatedParameter::Float(init_parameter.ball_mass)),
            time_span: ParameterWidget::new(InstantiatedParameter::Float(init_parameter.time_span)),
            simulation_step: ParameterWidget::new(InstantiatedParameter::Float(
                init_parameter.simulation_step,
            )),
            equadiff_method: init_parameter.method,
            scaffold_len_target: ParameterWidget::new(InstantiatedParameter::Uint(7249)),
        }
    }
}

impl<State: AppState> RevolutionTab<State> {
    pub(crate) fn set_builder(&mut self, builder: CurveDescriptorBuilder<State>) {
        if self.curve_descriptor_widget.as_ref().map(|w| w.curve_name) != Some(builder.curve_name) {
            self.curve_descriptor_widget = Some(CurveDescriptorWidget::new(builder));
        }
    }

    pub(crate) fn set_method(&mut self, method: EquadiffSolvingMethod) {
        self.equadiff_method = method;
    }

    pub(crate) fn get_current_bezier_path_id(&self) -> Option<usize> {
        self.curve_descriptor_widget
            .as_ref()
            .and_then(CurveDescriptorWidget::get_bezier_path_id)
    }

    pub(crate) fn update_builder_parameter(
        &mut self,
        param_id: RevolutionParameterId,
        text: String,
    ) {
        match param_id {
            RevolutionParameterId::SectionParameter(id) => {
                if let Some(widget) = self.curve_descriptor_widget.as_mut() {
                    widget.update_builder_parameter(id, text);
                }
            }
            param => {
                let widget = match param {
                    RevolutionParameterId::SectionParameter(_) => unreachable!(),
                    RevolutionParameterId::HalfTurnCount => &mut self.half_turn_count,
                    RevolutionParameterId::NbSpiral => &mut self.nb_spiral_state_input,
                    RevolutionParameterId::RevolutionRadius => &mut self.radius_input,
                    RevolutionParameterId::ScaffoldLenTarget => &mut self.scaffold_len_target,
                    RevolutionParameterId::NbSectionPerSegment => {
                        &mut self.nb_section_per_segment_input
                    }
                    RevolutionParameterId::SpringStiffness => &mut self.spring_stiffness,
                    RevolutionParameterId::TorsionStiffness => &mut self.torsion_stiffness,
                    RevolutionParameterId::FluidFriction => &mut self.fluid_friction,
                    RevolutionParameterId::BallMass => &mut self.ball_mass,
                    RevolutionParameterId::TimeSpan => &mut self.time_span,
                    RevolutionParameterId::SimulationStep => &mut self.simulation_step,
                };
                widget.set_text(text);
            }
        }
    }

    pub(crate) fn get_current_unrooted_surface(
        &self,
        app_state: &State,
    ) -> Option<UnrootedRevolutionSurfaceDescriptor> {
        let curve = self
            .curve_descriptor_widget
            .as_ref()
            .and_then(|w| w.build_curve(app_state))?;
        let revolution_radius = self
            .radius_input
            .get_value()
            .and_then(InstantiatedParameter::get_float)
            .map(RevolutionSurfaceRadius::from_signed_f64)?;
        let half_turn_count = self
            .half_turn_count
            .get_value()
            .and_then(InstantiatedParameter::get_int)?;

        let (curve_plane_position, curve_plane_orientation) = self
            .curve_descriptor_widget
            .as_ref()
            .and_then(|w| w.get_frame(app_state))
            .unwrap_or_else(|| (Vec3::zero(), Rotor3::identity()));

        Some(UnrootedRevolutionSurfaceDescriptor {
            curve,
            revolution_radius,
            half_turn_count,
            curve_plane_position,
            curve_plane_orientation,
        })
    }

    pub(crate) fn get_revolution_system(
        &self,
        app_state: &State,
        compute_area: bool,
    ) -> Option<RevolutionSurfaceSystemDescriptor> {
        let unrooted_surface = self.get_current_unrooted_surface(app_state)?;

        let rooting_parameters = RootingParameters {
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
                .and_then(InstantiatedParameter::get_uint)?,
            helix_parameters: app_state.get_dna_parameters(),
            simulation_parameters,
        };

        Some(system)
    }

    /// Get the number of shift per turn, updating `self.shift_generator` if needed.
    fn get_shift_per_turn(&self, app_state: &State) -> Option<isize> {
        self.try_get_shift_per_turn(app_state).or_else(|| {
            // TODO: This update must be done elsewhere.
            //let unrooted_surface = self.get_current_unrooted_surface(app_state)?;
            //let nb_spiral = self
            //    .nb_spiral_state_input
            //    .get_value()
            //    .and_then(InstantiatedParameter::get_uint)?;
            //let half_nb_helix = self.scaling.as_ref()?.nb_helix / 2;
            //self.shift_generator =
            //    unrooted_surface.shifts_to_get_n_spirals(half_nb_helix, nb_spiral);
            self.try_get_shift_per_turn(app_state)
        })
    }

    /// Return the number of shift per turn if `self.shift_generator` is up-to-date, and `None`
    /// otherwise.
    fn try_get_shift_per_turn(&self, app_state: &State) -> Option<isize> {
        let unrooted_surface = self.get_current_unrooted_surface(app_state)?;
        let nb_spiral = self
            .nb_spiral_state_input
            .get_value()
            .and_then(InstantiatedParameter::get_uint)?;
        let half_nb_helix = self.scaling.as_ref()?.nb_helix / 2;
        self.shift_generator
            .as_ref()
            .and_then(|g| g.ith_value(self.shift_idx, nb_spiral, &unrooted_surface, half_nb_helix))
    }

    fn get_simulation_parameters(&self) -> Option<RevolutionSimulationParameters> {
        let nb_section_per_segment = self
            .nb_section_per_segment_input
            .get_value()
            .and_then(InstantiatedParameter::get_uint)?;
        let spring_stiffness = self
            .spring_stiffness
            .get_value()
            .and_then(InstantiatedParameter::get_float)?;
        let torsion_stiffness = self
            .torsion_stiffness
            .get_value()
            .and_then(InstantiatedParameter::get_float)?;
        let fluid_friction = self
            .fluid_friction
            .get_value()
            .and_then(InstantiatedParameter::get_float)?;
        let ball_mass = self
            .ball_mass
            .get_value()
            .and_then(InstantiatedParameter::get_float)?;
        let time_span = self
            .time_span
            .get_value()
            .and_then(InstantiatedParameter::get_float)?;
        let simulation_step = self
            .simulation_step
            .get_value()
            .and_then(InstantiatedParameter::get_float)?;
        let method = self.equadiff_method;

        Some(RevolutionSimulationParameters {
            nb_section_per_segment,
            spring_stiffness,
            torsion_stiffness,
            fluid_friction,
            ball_mass,
            time_span,
            simulation_step,
            method,
        })
    }

    pub(crate) fn modifying_radius(&self) -> bool {
        // self.radius_input.state.is_focused()
        // FIXME
        false
    }
}

impl<State: AppState> GuiTab<State> for RevolutionTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::AutoMode)))
    }

    fn update(&mut self, app_state: &mut State) -> Command<Message<State>> {
        if let Some(r) = app_state.get_current_revolution_radius()
            && !self.modifying_radius()
        {
            self.update_builder_parameter(
                RevolutionParameterId::RevolutionRadius,
                format!("{r:.3}"),
            );
        }

        self.scaling = self
            .scaffold_len_target
            .get_value()
            .and_then(InstantiatedParameter::get_uint)
            .and_then(|len_scaffold| {
                app_state.get_recommended_scaling_revolution_surface(len_scaffold)
            });

        if self.try_get_shift_per_turn(app_state).is_none()
            && let Some((unrooted_surface, nb_spiral)) =
                self.get_current_unrooted_surface(app_state).zip(
                    self.nb_spiral_state_input
                        .get_value()
                        .and_then(InstantiatedParameter::get_uint),
                )
        {
            let half_nb_helix = self.scaling.as_ref().unwrap().nb_helix / 2;
            self.shift_generator =
                unrooted_surface.shifts_to_get_n_spirals(half_nb_helix, nb_spiral);
        }
        Command::none()
    }

    fn content(
        &self,
        ui_size: UiSize,
        app_state: &State,
    ) -> iced::Element<'_, Self::Message, iced::Theme, iced::Renderer> {
        let desc = self.get_revolution_system(app_state, false);

        let shift_buttons = {
            let buttons = (button(text("-")), button(text("+")));
            if let Some(shift) = self.get_shift_per_turn(app_state) {
                row![
                    buttons.0.on_press(Message::DecrRevolutionShift),
                    buttons.1.on_press(Message::IncrRevolutionShift),
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

        let simulation_buttons = if app_state.get_simulation_state() == SimulationState::Relaxing {
            self::column![
                text_button("Abort", ui_size).on_press(Message::StopSimulation),
                jump_by(2),
                text(
                    app_state
                        .get_reader()
                        .get_current_length_of_relaxed_shape()
                        .map_or(String::new(), |l| format!("Current total length: {l}"))
                ),
                text_button("Finish", ui_size).on_press(Message::FinishRelaxation),
            ]
        } else {
            let mut button = text_button("Start", ui_size);
            if app_state.get_simulation_state() == SimulationState::None && desc.is_some() {
                button = button.on_press(Message::InitRevolutionRelaxation);
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
                        State::POSSIBLE_CURVES,
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
                    self.half_turn_count
                        .input_view(RevolutionParameterId::HalfTurnCount),
                ]
                .align_items(Alignment::Center),
                text(self.scaling.map_or_else(
                    || "Nb helix: ###".into(),
                    |RevolutionScaling { nb_helix }| format!("Nb helix: {nb_helix}")
                )),
                row![
                    text("Nb spiral"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.nb_spiral_state_input
                        .input_view(RevolutionParameterId::NbSpiral),
                ]
                .align_items(Alignment::Center),
                shift_buttons,
                row![
                    text("Revolution Radius"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.radius_input
                        .input_view(RevolutionParameterId::RevolutionRadius),
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
                    self.nb_section_per_segment_input
                        .input_view(RevolutionParameterId::NbSectionPerSegment),
                ]
                .align_items(Alignment::Center),
                row![
                    text("Target length"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.scaffold_len_target
                        .input_view(RevolutionParameterId::ScaffoldLenTarget),
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
                    self.spring_stiffness
                        .input_view(RevolutionParameterId::SpringStiffness),
                ]
                .align_items(Alignment::Center),
                row![
                    text("Torsion Stiffness"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.torsion_stiffness
                        .input_view(RevolutionParameterId::TorsionStiffness),
                ]
                .align_items(Alignment::Center),
                row![
                    text("Fluid Friction"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.fluid_friction
                        .input_view(RevolutionParameterId::FluidFriction),
                ]
                .align_items(Alignment::Center),
                row![
                    text("Ball Mass"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.ball_mass.input_view(RevolutionParameterId::BallMass),
                ]
                .align_items(Alignment::Center),
                row![
                    text("Solving Method"),
                    Space::with_width(ui_size.checkbox_spacing()),
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
                    self.time_span.input_view(RevolutionParameterId::TimeSpan),
                ]
                .align_items(Alignment::Center),
                row![
                    text("Simulation Step"),
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.simulation_step
                        .input_view(RevolutionParameterId::SimulationStep),
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
    }
}

#[derive(Clone, Copy)]
pub struct RevolutionScaling {
    pub nb_helix: usize,
}
