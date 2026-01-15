use crate::{
    GuiAppState, GuiRequests,
    helpers::{right_checkbox, section, start_stop_button, subsection, text_button},
    left_panel::{
        BrownianParametersFactory, Message, RigidBodyFactory, RigidBodyParametersRequest,
        discrete_value::{FactoryId, RequestFactory, ValueId},
        tabs::{GuiTab, gostop::GoStop},
    },
    theme,
};
use ensnano_physics::parameters::{
    RAPIER_FLOAT_PARAMETERS_COUNT, RapierParameters, RapierSimulationType,
};
use ensnano_utils::{
    RollRequest, SimulationState, consts::ICON_PHYSICAL_ENGINE,
    keyboard_priority::keyboard_priority, ui_size::UiSize,
};
use iced::{
    Alignment,
    widget::{Column, Space, column, pick_list, row, scrollable, text, text_input},
};
use iced_aw::TabLabel;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct SimulationTab<State: GuiAppState> {
    rigid_body_factory: RequestFactory<RigidBodyFactory>,
    brownian_factory: RequestFactory<BrownianParametersFactory>,
    //rigid_grid_button: GoStop<State>,
    rigid_helices_button: GoStop<State>,
    physical_simulation: PhysicalSimulation,
    pub rapier_parameters: RapierParameters,
    // holds the value of the string fields
    pub rapier_parameter_fields: HashMap<String, String>,
}

impl<State: GuiAppState> SimulationTab<State> {
    pub fn new() -> Self {
        let init_brownian = BrownianParametersFactory {
            rate: 0.,
            amplitude: 0.08,
        };
        Self {
            rigid_body_factory: RequestFactory::new(
                FactoryId::RigidBody,
                RigidBodyFactory {
                    volume_exclusion: false,
                    brownian_motion: false,
                    brownian_parameters: init_brownian.clone(),
                },
            ),
            brownian_factory: RequestFactory::new(FactoryId::Brownian, init_brownian),
            rigid_helices_button: GoStop::new(
                String::from("Rigid Helices"),
                Message::RigidHelicesSimulation,
            ),
            physical_simulation: Default::default(),
            rapier_parameters: Default::default(),
            rapier_parameter_fields: Default::default(),
        }
    }

    pub fn set_volume_exclusion(&mut self, volume_exclusion: bool) {
        self.rigid_body_factory.requestable.volume_exclusion = volume_exclusion;
    }

    pub fn set_brownian_motion(&mut self, brownian_motion: bool) {
        self.rigid_body_factory.requestable.brownian_motion = brownian_motion;
    }

    pub fn make_rigid_body_request(&self, request: &mut Option<RigidBodyParametersRequest>) {
        self.rigid_body_factory.make_request(request);
    }

    pub fn update_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<RigidBodyParametersRequest>,
    ) {
        self.rigid_body_factory
            .update_request(value_id, value, request);
    }

    pub fn update_brownian(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<RigidBodyParametersRequest>,
    ) {
        let new_brownian = self.brownian_factory.update_value(value_id, value);
        self.rigid_body_factory.requestable.brownian_parameters = new_brownian;
        self.rigid_body_factory.make_request(request);
    }

    pub fn get_physical_simulation_request(&self) -> RollRequest {
        self.physical_simulation.request()
    }

    pub fn leave_tab<R: GuiRequests>(&self, requests: Arc<Mutex<R>>, app_state: &State) {
        if SimulationState::RigidGrid == app_state.get_simulation_state() {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop grids");
        } else if SimulationState::RigidHelices == app_state.get_simulation_state() {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop helices");
        }
    }

    fn request_stop_rigid_body_simulation<R: GuiRequests>(&self, requests: Arc<Mutex<R>>) {
        let mut request = None;
        self.make_rigid_body_request(&mut request);
        if let Some(request) = request {
            requests
                .lock()
                .unwrap()
                .update_rigid_body_simulation_parameters(request);
        }
    }

    fn helix_btns<'a>(
        go_stop: &'a GoStop<State>,
        app_state: &State,
        ui_size: UiSize,
    ) -> iced::Element<'a, Message<State>> {
        let sim_state = app_state.get_simulation_state();
        if sim_state.is_paused() {
            row![
                go_stop.view(true, false),
                text_button("Reset", ui_size).on_press(Message::ResetSimulation),
            ]
            .spacing(3)
            .into()
        } else {
            let helices_active = sim_state.is_none() || sim_state.simulating_helices();
            go_stop.view(helices_active, sim_state.simulating_helices())
        }
    }
}

impl<State: GuiAppState> GuiTab<State> for SimulationTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Icon(ICON_PHYSICAL_ENGINE)
    }

    fn content(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Self::Message> {
        let sim_state = &app_state.get_simulation_state();
        let rigid_grid_is_active = sim_state.is_none() || sim_state.simulating_grid();
        let roll_active = sim_state.is_none() || sim_state.is_rolling();

        let volume_exclusion = self.rigid_body_factory.requestable.volume_exclusion;
        let brownian_motion = self.rigid_body_factory.requestable.brownian_motion;

        let content = column![
            section("Simulation (Beta)", ui_size),
            column![
                self.physical_simulation
                    .view(ui_size, "Roll", roll_active, sim_state.is_rolling(),),
                start_stop_button(
                    "Rigid Grids",
                    ui_size,
                    rigid_grid_is_active.then_some(Message::RigidGridSimulation),
                    sim_state.simulating_grid()
                ),
                Self::helix_btns(&self.rigid_helices_button, app_state, ui_size,),
            ]
            .spacing(ui_size.button_spacing()),
            subsection("Parameters for helices simulation", ui_size),
            Column::with_children(self.rigid_body_factory.view(true, ui_size.main_text())),
            right_checkbox(
                volume_exclusion,
                "Volume exclusion",
                Message::VolumeExclusion,
                ui_size,
            ),
            right_checkbox(
                brownian_motion,
                "Unmatched nt jiggling",
                Message::BrownianMotion,
                ui_size,
            ),
            Column::with_children(
                self.brownian_factory
                    .view(brownian_motion, ui_size.main_text())
            ),
            section("Relaxation", ui_size),
            column![
                row![pick_list(
                    [
                        RapierSimulationType::Full,
                        RapierSimulationType::Rigid,
                        RapierSimulationType::Cut,
                        RapierSimulationType::KCut,
                    ],
                    Some(self.rapier_parameters.simulation_type),
                    |simulation_type| Message::UpdateRapierParameters(RapierParameters {
                        simulation_type,
                        ..self.rapier_parameters
                    }),
                )],
                row![
                    text_button("Start", ui_size).on_press_maybe(
                        if self.rapier_parameters.is_simulation_running {
                            None
                        } else {
                            Some(Message::UpdateRapierParameters(apply_parameter_fields(
                                &self.rapier_parameter_fields,
                                &RapierParameters {
                                    is_simulation_running: true,
                                    ..self.rapier_parameters
                                },
                            )))
                        }
                    ),
                    Space::with_width(ui_size.button_spacing()),
                    text_button("Stop", ui_size).on_press_maybe(
                        if !self.rapier_parameters.is_simulation_running || sim_state.is_paused() {
                            None
                        } else {
                            Some(Message::StopSimulation)
                        }
                    ),
                    Space::with_width(ui_size.button_spacing()),
                    text_button("Reset", ui_size)
                        .on_press_maybe(sim_state.is_paused().then(|| Message::ResetSimulation)),
                ],
            ]
            .spacing(ui_size.button_spacing()),
            kcut_threshold_editor(
                &self.rapier_parameters,
                &self.rapier_parameter_fields,
                ui_size
            ),
            view_rapier_parameters(
                self.rapier_parameters,
                &self.rapier_parameter_fields,
                ui_size,
            )
        ]
        .spacing(5);

        scrollable(content).into()
    }
}

fn kcut_threshold_editor<State: GuiAppState>(
    parameters: &RapierParameters,
    fields: &HashMap<String, String>,
    ui_size: UiSize,
) -> iced::Element<'static, Message<State>> {
    row![
        "KCut threshold",
        Space::with_width(ui_size.checkbox_spacing()),
        text_button("-", ui_size).on_press_maybe(
            (parameters.simulation_type == RapierSimulationType::KCut).then(|| {
                let new_value = if parameters.k_cut_threshold <= 1 {
                    1
                } else {
                    parameters.k_cut_threshold - 1
                };
                Message::UpdateRapierParameters(apply_parameter_fields(
                    fields,
                    &RapierParameters {
                        k_cut_threshold: new_value,
                        ..*parameters
                    },
                ))
            })
        ),
        text_button("+", ui_size).on_press_maybe(
            (parameters.simulation_type == RapierSimulationType::KCut).then(|| {
                let new_value = parameters.k_cut_threshold + 1;
                Message::UpdateRapierParameters(apply_parameter_fields(
                    fields,
                    &RapierParameters {
                        k_cut_threshold: new_value,
                        ..*parameters
                    },
                ))
            })
        ),
        Space::with_width(ui_size.checkbox_spacing()),
        text(parameters.k_cut_threshold)
    ]
    .align_items(Alignment::Center)
    .into()
}

const PARAMETER_FIELD_NAMES: [&str; RAPIER_FLOAT_PARAMETERS_COUNT] = [
    "Linear damping",
    "Angular damping",
    "Interbase spring stiffness",
    "Interbase spring damping",
    "Crossover stiffness",
    "Crossover damping",
    "Crossover rest length",
    "Free nucleotide stiffness",
    "Free nucleotide damping",
    "Free nucleotide rest length",
    "Repulsion strength",
    "Repulsion range",
    "Brownian motion strength",
    "Entropic springs strength",
    "Entropic springs damping",
    "Planar squish strength",
    "Planar squish damping",
    "Planar squish soft cutoff",
];

fn apply_parameter_fields(
    fields: &HashMap<String, String>,
    parameters: &RapierParameters,
) -> RapierParameters {
    let default_array = parameters.parameters_array();

    let array = (0..PARAMETER_FIELD_NAMES.len())
        .map(|k| {
            fields
                .get(PARAMETER_FIELD_NAMES[k])
                .and_then(|str| str.parse::<f32>().ok())
                .unwrap_or(default_array[k])
        })
        .collect::<Vec<_>>();

    let mut result = *parameters;
    result.set_parameters_array(&array);

    result
}

fn rapier_parameters_field_editor<State: GuiAppState>(
    description: impl ToString,
    default_value: f32,
    ui_size: UiSize,
    fields: &HashMap<String, String>,
    parameters: &RapierParameters,
) -> iced::Element<'static, Message<State>> {
    let description = description.to_string();
    let default_field_value = default_value.to_string();
    let current_value = fields.get(&description).unwrap_or(&default_field_value);

    row![
        text(&description),
        Space::with_width(ui_size.checkbox_spacing()),
        keyboard_priority(
            "Rapier parameters ".to_owned() + &description,
            Message::<State>::SetKeyboardPriority,
            // if parameters.is_simulation_running {
            //     text_input(current_value, current_value)
            // } else {
            text_input(current_value, current_value)
                .on_input(move |str| {
                    Message::UpdateRapierParameterField(description.clone(), str)
                })
                .on_submit(Message::UpdateRapierParameters(apply_parameter_fields(
                    fields, parameters,
                )))
                // }
                .width(70)
                .style(theme::BadValue(true)),
        )
    ]
    .align_items(Alignment::Center)
    .into()
}

fn view_rapier_parameters<State: GuiAppState>(
    parameters: RapierParameters,
    fields: &HashMap<String, String>,
    ui_size: UiSize,
) -> iced::Element<'static, Message<State>> {
    let mut elements: Vec<iced::Element<'static, Message<State>>> =
        vec![subsection("Rapier parameters", ui_size).into()];

    let values = parameters.parameters_array();

    for k in 0..PARAMETER_FIELD_NAMES.len() {
        elements.push(rapier_parameters_field_editor(
            PARAMETER_FIELD_NAMES[k],
            values[k],
            ui_size,
            fields,
            &parameters,
        ));
    }

    Column::from_vec(elements)
        .spacing(ui_size.button_spacing())
        .into()
}

#[derive(Default)]
struct PhysicalSimulation;

impl PhysicalSimulation {
    fn view<State: GuiAppState>(
        &self,
        ui_size: UiSize,
        name: &'static str,
        active: bool,
        running: bool,
    ) -> iced::Element<'_, Message<State>> {
        let button_str = if running { "Stop" } else { name };
        let mut button = text_button(button_str, ui_size);
        button = if running {
            button.style(iced::theme::Button::Destructive)
        } else {
            button.style(iced::theme::Button::Positive)
        };
        if active {
            button = button.on_press(Message::RollSimulationRequest);
        }
        row![button].into()
    }

    fn request(&self) -> RollRequest {
        RollRequest {
            target_helices: None,
        }
    }
}
