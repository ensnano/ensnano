use crate::{
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{extra_jump, jump_by, section, subsection, text_button},
    left_panel::{LeftPanelMessage, tabs::GuiTab},
    theme,
};
use ensnano_design::curves::torus::CurveDescriptor2D;
use ensnano_state::{
    app_state::AppState,
    gui::{
        curve::{CurveDescriptorBuilder, Frame, InstantiatedParameter, RevolutionScaling},
        state::RevolutionParameterId,
    },
};
use ensnano_utils::{
    SimulationState,
    keyboard_priority::keyboard_priority,
    surfaces::{
        EquadiffSolvingMethod,
        RevolutionSimulationParameters,
        RevolutionSurfaceRadius,
        RevolutionSurfaceSystemDescriptor,
        RootingParameters, //ShiftGenerator,
        UnrootedRevolutionSurfaceDescriptor,
    },
    ui_size::UiSize,
};
use iced::{
    Alignment, Command, Length,
    widget::{
        Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
    },
};
use iced_aw::TabLabel;
use ultraviolet::{Rotor3, Vec3};

use num::integer::{gcd, lcm};

#[derive(Debug, Clone, Copy)]
enum ParameterKind {
    Float,
    Int,
    Uint,
}

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

    fn input_view(&self, id: RevolutionParameterId) -> iced::Element<'_, LeftPanelMessage> {
        keyboard_priority(
            format!("Revolution tab {id:?}"),
            LeftPanelMessage::SetKeyboardPriority,
            text_input("", &self.current_text)
                .on_input(move |s| LeftPanelMessage::RevolutionParameterUpdate {
                    parameter_id: id,
                    text: s,
                })
                .width(50)
                .style(theme::BadValue(self.contains_valid_input())),
        )
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

struct CurveDescriptorWidget {
    parameters: Vec<(&'static str, ParameterWidget)>,
    curve_name: &'static str,
    builder: CurveDescriptorBuilder,
}

impl CurveDescriptorWidget {
    fn new(builder: CurveDescriptorBuilder) -> Self {
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

    fn view(&self, ui_size: UiSize) -> iced::Element<'_, LeftPanelMessage> {
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

    fn build_curve(&self, app_state: &AppState) -> Option<CurveDescriptor2D> {
        (self.builder.build)(&self.instantiated_parameters(), app_state)
    }

    fn get_bezier_path_id(&self) -> Option<usize> {
        (self.builder.bezier_path_id)(&self.instantiated_parameters())
    }

    fn get_rotational_symmetry_order(&self) -> Option<usize> {
        (self.builder.rotational_symmetry_order)(&self.instantiated_parameters())
    }

    fn get_frame(&self, app_state: &AppState) -> Option<Frame> {
        (self.builder.frame)(&self.instantiated_parameters(), app_state)
    }
}

pub(crate) struct RevolutionTab {
    curve_descriptor_widget: Option<CurveDescriptorWidget>,
    nb_helices: usize,
    winding: isize,
    nb_spirals: usize,
    twist: usize,
    twist_input: ParameterWidget, // half_turn_count
    radius_input: ParameterWidget,
    scaling: Option<RevolutionScaling>,
    nb_helices_input: ParameterWidget,
    nb_spirals_state_input: ParameterWidget,
    // shift_generator: Option<ShiftGenerator>, // NS: obsolete
    // pub(crate) shift_idx: isize,
    // pub(crate) nb_spirals_idx: isize,
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

impl Default for RevolutionTab {
    fn default() -> Self {
        let init_parameter = RevolutionSimulationParameters::default();
        Self {
            curve_descriptor_widget: None,
            twist_input: ParameterWidget::new(InstantiatedParameter::Uint(0)),
            radius_input: ParameterWidget::new(InstantiatedParameter::Float(10.)),
            scaling: None,
            nb_helices: 12,
            winding: 0,
            twist: 0,
            nb_spirals: 2,
            nb_helices_input: ParameterWidget::new(InstantiatedParameter::Uint(12)),
            nb_spirals_state_input: ParameterWidget::new(InstantiatedParameter::Uint(2)),
            // shift_generator: None,
            // shift_idx: 0,
            // nb_spirals_idx: 0,
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
            scaffold_len_target: ParameterWidget::new(InstantiatedParameter::Uint(8064)),
        }
    }
}

impl RevolutionTab {
    pub(crate) fn set_builder(&mut self, builder: CurveDescriptorBuilder) {
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
            .and_then(CurveDescriptorWidget::get_rotational_symmetry_order)
    }

    pub(crate) fn get_rotational_symmetry_order(&self) -> Option<usize> {
        self.curve_descriptor_widget
            .as_ref()
            .and_then(CurveDescriptorWidget::get_rotational_symmetry_order)
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
                    RevolutionParameterId::Twist => &mut self.twist_input,
                    RevolutionParameterId::NbHelices => &mut self.nb_helices_input,
                    RevolutionParameterId::NbSpiral => &mut self.nb_spirals_state_input,
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
        app_state: &AppState,
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

        // obsolete
        // let twist = self
        //     .twist_input
        //     .get_value()
        //     .and_then(InstantiatedParameter::get_uint)
        //     .unwrap_or(0) as isize;

        let (simplified_twist, simplified_rotational_symmetry_order) =
            self.get_simplified_twist_and_rotational_symmetry_order();

        // let rotational_symmetry_order = curve.rotational_symmetry_order();

        // // NICOLAS: now half_turn_count only works for Ellipse
        // let half_turn_count = curve.twist();

        let (curve_plane_position, curve_plane_orientation) = self
            .curve_descriptor_widget
            .as_ref()
            .and_then(|w| w.get_frame(app_state))
            .unwrap_or_else(|| (Vec3::zero(), Rotor3::identity()));

        Some(UnrootedRevolutionSurfaceDescriptor {
            curve,
            revolution_radius,
            simplified_twist,
            simplified_rotational_symmetry_order,
            curve_plane_position,
            curve_plane_orientation,
        })
    }

    pub(crate) fn get_revolution_system(
        &self,
        app_state: &AppState,
        compute_area: bool,
    ) -> Option<RevolutionSurfaceSystemDescriptor> {
        // println!("starting");
        let unrooted_surface = self.get_current_unrooted_surface(app_state)?;

        let rooting_parameters = RootingParameters {
            nb_helices: self.nb_helices,
            nb_spirals: self.nb_spirals,
            winding: self.winding,
            junction_smoothening: 0.,
            // obsolete
            // nb_helix_per_half_section: self.get_even_nb_helices_input()? / 2, // self.scaling.as_ref()?.nb_helix / 2,
            // shift_per_turn: self.try_get_shift_per_turn(app_state)?,
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

        // println!("system ok");
        Some(system)
    }

    // obsolete
    // /// Get the number of shift per turn, updating `self.shift_generator` if needed.
    // fn get_shift_per_turn(&self, app_state: &AppState) -> Option<isize> {
    //     self.try_get_shift_per_turn(app_state).or_else(|| {
    //         // TODO: This update must be done elsewhere.
    //         //let unrooted_surface = self.get_current_unrooted_surface(app_state)?;
    //         //let nb_spiral = self
    //         //    .nb_spiral_state_input
    //         //    .get_value()
    //         //    .and_then(InstantiatedParameter::get_uint)?;
    //         //let half_nb_helix = self.scaling.as_ref()?.nb_helix / 2;
    //         //self.shift_generator =
    //         //    unrooted_surface.shifts_to_get_n_spirals(half_nb_helix, nb_spiral);
    //         self.try_get_shift_per_turn(app_state)
    //     })
    // }

    // obsolete
    // fn get_nb_spirals(&self, app_state: &AppState) -> Option<usize> {
    //     let nb_sp = self.nb_spirals_idx.abs() as usize;
    //     // println!("Ping {nb_sp}");
    //     Some(nb_sp)
    // }

    // obsolete
    // #[inline(always)]
    // fn get_even_nb_helices_input(&self) -> Option<usize> {
    //     let nb_helices = (((self
    //         .nb_helices_input
    //         .get_value()
    //         .and_then(InstantiatedParameter::get_uint)?
    //         + 1)
    //         / 2)
    //         * 2)
    //     .max(4);
    //     Some(nb_helices)
    // }

    // obsolete
    // #[inline(always)]
    // fn get_nb_spirals_input(&self) -> Option<usize> {
    //     let nb_spiral = self
    //         .nb_spirals_state_input
    //         .get_value()
    //         .and_then(InstantiatedParameter::get_uint)?;
    //     Some(nb_spiral)
    // }

    // obsolete
    // /// Return the number of shift per turn if `self.shift_generator` is up-to-date, and `None`
    // /// otherwise.
    // fn try_get_shift_per_turn(&self, app_state: &AppState) -> Option<isize> {
    //     let unrooted_surface = self.get_current_unrooted_surface(app_state)?;
    //     let nb_helices = self.get_even_nb_helices_input()?;
    //     let nb_spiral = self.get_nb_spirals_input()?;
    //     let half_nb_helix = nb_helices / 2;
    //     // let half_nb_helix = self.scaling.as_ref()?.nb_helix / 2;
    //     self.shift_generator
    //         .as_ref()
    //         .and_then(|g| g.ith_value(self.shift_idx, nb_spiral, &unrooted_surface, half_nb_helix))
    // }

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

    /// Increase the twist of the revolution surface
    pub(crate) fn inc_twist(&mut self) -> usize {
        self.twist += 1;
        self.check_and_adapt_nb_helices();
        self.twist
    }

    /// Decrease the twist of the revolution surface
    pub(crate) fn dec_twist(&mut self) -> usize {
        if self.twist > 0 {
            self.twist -= 1;
        }
        self.check_and_adapt_nb_helices();
        self.twist
    }

    /// Simplify the fraction twist / rotational_sym_order to allow nb_helices to be a multiple of only normalized_rot_sym_order
    pub(crate) fn get_simplified_twist_and_rotational_symmetry_order(&self) -> (usize, usize) {
        match self.get_rotational_symmetry_order() {
            None => (self.twist, 1),
            Some(0) => (0, 1),
            Some(rso) => {
                let tw = self.twist;
                // simplify the fraction tw/rso
                let g = gcd(tw, rso);
                (tw / g, rso / g)
            }
        }
    }

    /// Increase the number of helices to next multiple of 2 and normalized rotational order symmetry; and adapt the number of spirals if needed
    pub(crate) fn inc_nb_helices(&mut self) -> usize {
        let (tw, rso) = self.get_simplified_twist_and_rotational_symmetry_order();
        let m = lcm(2, rso);
        println!("{tw} {rso} {m}");
        self.nb_helices += m;
        self.check_and_adapt_nb_helices()
    }

    // Decrease the number of helices to previous multiple of 2 and normalized rotational order symmetry; and adapt the number of spirals if needed
    pub(crate) fn dec_nb_helices(&mut self) -> usize {
        let (_, rso) = self.get_simplified_twist_and_rotational_symmetry_order();
        let m = lcm(2, rso);
        if self.nb_helices > m {
            self.nb_helices = self.nb_helices - m;
        }
        if self.nb_helices < m.max(4) {
            self.nb_helices = match m {
                2 => 4,
                3 => 6,
                _ => m,
            }
        }
        self.check_and_adapt_nb_helices()
    }

    /// Makes sure the number of helices is even and at least 4
    pub(crate) fn check_and_adapt_nb_helices(&mut self) -> usize {
        let (_, rso) = self.get_simplified_twist_and_rotational_symmetry_order();
        let m = lcm(2, rso);
        let r = self.nb_helices % m;
        if r != 0 {
            self.nb_helices = self.nb_helices - r;
        }
        if self.nb_helices < m.max(4) {
            self.nb_helices = match m {
                2 => 4,
                3 => 6,
                _ => m,
            }
        }
        self.check_and_adapt_nb_spirals();
        self.nb_helices
    }

    /// Look for the next divisor of number of helices and at most nb_helices
    pub(crate) fn inc_nb_spirals(&mut self) -> usize {
        let nb_sp = self.nb_spirals;
        let nb_hx = self.nb_helices;
        if self.nb_spirals >= nb_hx / 2 {
            self.nb_spirals = nb_hx;
        } else {
            for i in nb_sp + 1..=nb_hx / 2 {
                if nb_hx % i == 0 {
                    self.nb_spirals = i;
                    break;
                }
            }
        }
        self.check_and_adapt_winding();
        self.nb_spirals
    }

    /// Look for the previous divisor of number of helices and at least 1
    pub(crate) fn dec_nb_spirals(&mut self) -> usize {
        let nb_sp = self.nb_spirals;
        let nb_hx = self.nb_helices;
        if self.nb_spirals <= 1 {
            self.nb_spirals = 1;
        } else {
            for i in (1..nb_sp).rev() {
                if nb_hx % i == 0 {
                    self.nb_spirals = i;
                    break;
                }
            }
        }
        self.check_and_adapt_winding();
        self.nb_spirals
    }

    /// Make sure the nb of spirals is ≥ 1 and divide nb_helices
    pub(crate) fn check_and_adapt_nb_spirals(&mut self) -> usize {
        if self.nb_spirals <= 0 {
            self.nb_spirals = 1;
        }
        if self.nb_helices % self.nb_spirals != 0 {
            self.dec_nb_spirals();
        }
        self.check_and_adapt_winding();
        self.nb_spirals
    }

    /// Check and adapt the winding parameter
    /// Winding parameter must be such that:
    /// - given n = number of helices per section
    /// - given s = number of spiraling helices (s divides n)
    /// - given t,r = the twist and rotational symmetry order with t and r coprime and r dividing n; let d = t*n/r
    /// - given a winding parameter w, starting from index 0:
    /// - after a turn the helices reaches the index d+w
    /// - we want that it returns to its initial position after exactly n/s turns, it follows that:
    /// - (n/s)*(d+w) is a multiple of n and thus: d + w is a multiple of s, let's say k * s
    ///     - given that the length of the cycle in term of turns is lcm(d+w, n) / (d+w), it follows that d is a proper winding iff:
    ///          lcm(d+w, n)/(d+w) = n/s
    ///       iff lcm(k*s, n/s * s) = s * lcm(k, n/s) = k*s * n/s
    ///       iff lcm(k, n/s) = k * n/s
    ///       iff k and n/s are coprime
    /// Hence, the winding parameters generating s spirals (of n/s turns) are: -d + s * k where k and n/s are coprime
    pub(crate) fn check_and_adapt_winding(&mut self) -> isize {
        let wd = self.winding;
        let nb_hx = self.nb_helices as isize;
        let nb_sp = self.nb_spirals as isize;
        let (tw, rso) = self.get_simplified_twist_and_rotational_symmetry_order();

        let d = (tw as isize * nb_hx) / rso as isize;
        let w = wd + d;
        if w % nb_sp != 0 || gcd(w / nb_sp, nb_hx / nb_sp) != 1 {
            self.inc_winding();
        }
        self.winding
    }

    /// Increase the winding parameter
    pub(crate) fn inc_winding(&mut self) -> isize {
        let wd = self.winding;
        let nb_hx = self.nb_helices as isize;
        let nb_sp = self.nb_spirals as isize;
        let n_s = nb_hx / nb_sp;
        let (tw, rso) = self.get_simplified_twist_and_rotational_symmetry_order();

        let d = (tw as isize * nb_hx) / rso as isize;
        let w = wd + d;
        let mut k = 1 + w / nb_sp;
        for i in 0..=n_s {
            print!("{},", (i * (d + -d + k * nb_sp)) % nb_hx);
        }
        println!();
        while gcd(k, n_s) != 1 {
            k += 1;
            for i in 0..=n_s {
                print!("{},", (i * (d + -d + k * nb_sp)) % nb_hx);
            }
            println!();
        }
        println!("*");

        self.winding = -d + k * nb_sp;
        self.winding
    }

    /// Decrease the winding parameter
    pub(crate) fn dec_winding(&mut self) -> isize {
        let wd = self.winding;
        let nb_hx = self.nb_helices as isize;
        let nb_sp = self.nb_spirals as isize;
        let n_s = nb_hx / nb_sp;
        let (tw, rso) = self.get_simplified_twist_and_rotational_symmetry_order();

        let d = (tw as isize * nb_hx) / rso as isize;
        let w = wd + d;
        let mut k = -1 + w / nb_sp;
        for i in 0..=n_s {
            print!("{},", (i * (d + -d + k * nb_sp)) % nb_hx);
        }
        println!();
        while gcd(k, n_s) != 1 {
            k -= 1;
            for i in 0..=n_s {
                print!("{},", (i * (d + -d + k * nb_sp)) % nb_hx);
            }
            println!();
        }
        println!("*");

        self.winding = -d + k * nb_sp;
        self.winding
    }
}

macro_rules! button_widget {
    ($label: expr, $value: expr, $dec_msg: tt, $inc_msg: tt, $comment: expr, $ui_size: expr) => {{
        let buttons = (button(" - "), button(" + "));
        let value = if let Some(x) = $value {
            format!("{}", x)
        } else {
            "—".into()
        };
        row![
            text(format!("{}: {:>4}", $label, value)),
            Space::with_width($ui_size.checkbox_spacing()),
            buttons.0.on_press(LeftPanelMessage::$dec_msg),
            buttons.1.on_press(LeftPanelMessage::$inc_msg),
            Space::with_width($ui_size.checkbox_spacing()),
            text(format!("{}", $comment)),
        ]
        .align_items(Alignment::Center)
    }};
}

impl GuiTab for RevolutionTab {
    type Message = LeftPanelMessage;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::AutoMode)))
    }

    fn update(&mut self, app_state: &mut AppState) -> Command<LeftPanelMessage> {
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

        // NS: obsolete
        // if self.try_get_shift_per_turn(app_state).is_none()
        //     && let Some((unrooted_surface, nb_spirals)) =
        //         self.get_current_unrooted_surface(app_state).zip(
        //             self.nb_spirals_state_input
        //                 .get_value()
        //                 .and_then(InstantiatedParameter::get_uint),
        //         )
        // {
        //     // let half_nb_helix = self.scaling.as_ref().map_or(0, |scaling| scaling.nb_helix) / 2;
        //     let nb_helices = self.get_even_nb_helices_input().unwrap_or(0);
        //     let half_nb_helices = nb_helices / 2;
        //     self.shift_generator =
        //         unrooted_surface.shifts_to_get_n_spirals(half_nb_helices, nb_spirals);
        // }
        Command::none()
    }

    fn content(&self, ui_size: UiSize, app_state: &AppState) -> iced::Element<'_, Self::Message> {
        let desc = self.get_revolution_system(app_state, false);

        let nb_helices_buttons = button_widget!(
            "Nb of helices per section",
            Some(self.nb_helices),
            DecrNbHelices,
            IncrNbHelices,
            self.scaling.map_or_else(
                || "No suggested nb helices".into(),
                |RevolutionScaling {
                     suggested_nb_helix: sug_nb_helix,
                 }| format!("Suggested: about {sug_nb_helix}")
            ),
            ui_size
        );

        let winding_buttons = button_widget!(
            "Winding parameter",
            Some(self.winding), // get_shift_per_turn(app_state),
            DecrWinding,
            IncrWinding,
            "",
            ui_size
        );

        let nb_spirals_buttons = button_widget!(
            "Nb of spirals",
            Some(self.nb_spirals), // self.get_nb_spirals(app_state),
            DecrNbSpirals,
            IncrNbSpirals,
            "",
            ui_size
        );

        let twist_buttons = button_widget!(
            {
                match self.get_rotational_symmetry_order() {
                    None | Some(0) | Some(1) => "Twist (Nb of turns)".into(),
                    Some(sym_order) => format!("Twist (Nb of 1/{sym_order}-turns)"),
                }
            },
            Some(self.twist), // self.get_nb_spirals(app_state),
            DecrTwist,
            IncrTwist,
            "",
            ui_size
        );

        let simulation_buttons = if SimulationState::Relaxing == app_state.get_simulation_state() {
            column![
                text_button("Abort", ui_size).on_press(LeftPanelMessage::StopSimulation),
                jump_by(2),
                text(
                    app_state
                        .get_reader()
                        .get_current_length_of_relaxed_shape()
                        .map_or(String::new(), |l| format!("Current total length: {l}"))
                ),
                text_button("Finish", ui_size).on_press(LeftPanelMessage::FinishRelaxation),
            ]
        } else {
            let mut button = text_button("Start", ui_size);
            if SimulationState::None == app_state.get_simulation_state() && desc.is_some() {
                button = button.on_press(LeftPanelMessage::InitRevolutionRelaxation);
            }
            column![button]
        };

        // let string_: &'_ String = &self
        //     .get_rotational_symmetry_order()
        //     .map_or_else(
        //         || "Twist (Nb of turns)".into(),
        //         |sym_order| match sym_order {
        //             0 | 1 => "Twist (Nb of turns)".into(),
        //             _ => format!("Twist (Nb of 1/{sym_order}-turns)"),
        //         },
        //     )
        //     .into();

        // let even_nb_helices_input = self.get_even_nb_helices_input();

        let content = column![
            section("Revolution Surfaces", ui_size),
            checkbox("Show revolution axis", app_state.get_show_bezier_paths())
                .on_toggle(LeftPanelMessage::SetShowBezierPaths),
            column![
                extra_jump(),
                subsection("Revolution surface parameters", ui_size),
                row![
                    "Curve type",
                    Space::with_width(ui_size.checkbox_spacing()),
                    pick_list(
                        AppState::POSSIBLE_CURVES,
                        self.curve_descriptor_widget
                            .as_ref()
                            .map(|w| w.builder.clone()),
                        LeftPanelMessage::CurveBuilderPicked,
                    )
                    .placeholder("Pick.."),
                ]
                .align_items(Alignment::Center),
                if let Some(widget) = &self.curve_descriptor_widget {
                    widget.view(ui_size)
                } else {
                    column![].into()
                },
            ],
            column![
                // row![
                //     text(string_.as_str()),
                //     Space::with_width(ui_size.checkbox_spacing()),
                //     self.twist_input // half_turn_count
                //         .input_view(RevolutionParameterId::Twist), // ::HalfTurnCount
                // ]
                // .align_items(Alignment::Center),
                twist_buttons,
                row![
                    "Revolution Radius",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.radius_input
                        .input_view(RevolutionParameterId::RevolutionRadius),
                ]
                .align_items(Alignment::Center),
                extra_jump(),
                subsection("DNA routing parameters", ui_size),
                // row![
                //     "Nb helices (even)",
                //     Space::with_width(ui_size.checkbox_spacing()),
                //     self.nb_helices_input
                //         .input_view(RevolutionParameterId::NbHelices),
                //     text(self.scaling.map_or_else(
                //         || if let Some(nb_hx) = even_nb_helices_input {
                //             format!(" Used: {nb_hx} — No suggested nb helices")
                //         } else {
                //             "No suggested nb helices".into()
                //         },
                //         |RevolutionScaling {
                //              nb_helix: sug_nb_helix,
                //          }| if let Some(nb_hx) = even_nb_helices_input {
                //             format!(" Used: {nb_hx} — Suggested nb helices: {sug_nb_helix}")
                //         } else {
                //             format!(" Suggested nb helices: {sug_nb_helix}")
                //         }
                //     )),
                // ]
                // .align_items(Alignment::Center),
                nb_helices_buttons,
                // row![
                //     "Nb spirals",
                //     Space::with_width(ui_size.checkbox_spacing()),
                //     self.nb_spirals_state_input
                //         .input_view(RevolutionParameterId::NbSpiral),
                // ]
                nb_spirals_buttons,
                winding_buttons,
            ]
            .spacing(2),
            column![
                extra_jump(),
                subsection("Discretization parameters", ui_size),
                row![
                    "Nb section per segments",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.nb_section_per_segment_input
                        .input_view(RevolutionParameterId::NbSectionPerSegment),
                ]
                .align_items(Alignment::Center),
                row![
                    "Target length",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.scaffold_len_target
                        .input_view(RevolutionParameterId::ScaffoldLenTarget),
                ]
                .align_items(Alignment::Center),
            ]
            .spacing(2),
            column![
                extra_jump(),
                subsection("Simulation parameters", ui_size),
                row![
                    "Spring Stiffness",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.spring_stiffness
                        .input_view(RevolutionParameterId::SpringStiffness),
                ]
                .align_items(Alignment::Center),
                row![
                    "Torsion Stiffness",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.torsion_stiffness
                        .input_view(RevolutionParameterId::TorsionStiffness),
                ]
                .align_items(Alignment::Center),
                row![
                    "Fluid Friction",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.fluid_friction
                        .input_view(RevolutionParameterId::FluidFriction),
                ]
                .align_items(Alignment::Center),
                row![
                    "Ball Mass",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.ball_mass.input_view(RevolutionParameterId::BallMass),
                ]
                .align_items(Alignment::Center),
                row![
                    "Solving Method",
                    Space::with_width(ui_size.checkbox_spacing()),
                    pick_list(
                        EquadiffSolvingMethod::ALL_METHODS,
                        Some(self.equadiff_method),
                        LeftPanelMessage::RevolutionEquadiffSolvingMethodPicked,
                    ),
                ]
                .align_items(Alignment::Center),
                row![
                    "Tie Span",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.time_span.input_view(RevolutionParameterId::TimeSpan),
                ]
                .align_items(Alignment::Center),
                row![
                    "Simulation Step",
                    Space::with_width(ui_size.checkbox_spacing()),
                    self.simulation_step
                        .input_view(RevolutionParameterId::SimulationStep),
                ]
                .align_items(Alignment::Center),
            ]
            .spacing(2),
            column![
                extra_jump(),
                section("Relaxation computation", ui_size),
                simulation_buttons,
            ],
        ]
        .spacing(5);

        scrollable(content).width(Length::Fill).into()
    }
}
