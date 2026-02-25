use crate::{
    helpers::{right_checkbox, section, start_stop_button, subsection, text_button},
    left_panel::{
        BrownianParametersFactory, LeftPanelMessage, RigidBodyFactory, RigidBodyParametersRequest,
        discrete_value::RequestFactory,
        tabs::{GuiTab, gostop::GoStop},
    },
};
use ensnano_physics::parameters::{RapierFloatParameter, RapierParameters};
use ensnano_state::{
    app_state::AppState,
    gui::messages::{FactoryId, ValueId},
    requests::Requests,
};
use ensnano_utils::{
    RollRequest, SimulationState, consts::ICON_PHYSICAL_ENGINE,
    keyboard_priority::keyboard_priority, ui_size::UiSize,
};
use iced::{
    Alignment, Length,
    widget::{Column, Space, checkbox, column, row, scrollable, slider, text, text_input},
};
use iced_aw::TabLabel;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct SimulationTab {
    rigid_body_factory: RequestFactory<RigidBodyFactory>,
    brownian_factory: RequestFactory<BrownianParametersFactory>,
    //rigid_grid_button: GoStop,
    rigid_helices_button: GoStop,
    physical_simulation: PhysicalSimulation,
    pub rapier_parameters: RapierParameters,
    // holds the value of the string fields
    pub rapier_parameter_fields: HashMap<String, String>,
}

impl SimulationTab {
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
                LeftPanelMessage::RigidHelicesSimulation,
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

    pub fn leave_tab(&self, requests: Arc<Mutex<Requests>>, app_state: &AppState) {
        if SimulationState::RigidGrid == app_state.get_simulation_state() {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop grids");
        } else if SimulationState::RigidHelices == app_state.get_simulation_state() {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop helices");
        }
    }

    fn request_stop_rigid_body_simulation(&self, requests: Arc<Mutex<Requests>>) {
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
        go_stop: &'a GoStop,
        app_state: &AppState,
        ui_size: UiSize,
    ) -> iced::Element<'a, LeftPanelMessage> {
        let sim_state = app_state.get_simulation_state();
        if sim_state.is_paused() {
            row![
                go_stop.view(true, false),
                text_button("Reset", ui_size).on_press(LeftPanelMessage::ResetSimulation),
            ]
            .spacing(3)
            .into()
        } else {
            let helices_active = sim_state.is_none() || sim_state.simulating_helices();
            go_stop.view(helices_active, sim_state.simulating_helices())
        }
    }

    /// Updates the fields using the parameters.
    ///
    /// Used when modifications to the parameters are made by
    /// actors that are not the fields, like other parts of the GUI.
    pub fn update_parameters_fields(&mut self) {
        for parameter in RapierFloatParameter::values() {
            self.rapier_parameter_fields.insert(
                parameter.name().to_owned(),
                self.rapier_parameters.get_parameter(parameter).to_string(),
            );
        }

        self.rapier_parameter_fields.insert(
            "Target UPS:".to_owned(),
            self.rapier_parameters.target_ups.to_string(),
        );
    }
}

impl GuiTab for SimulationTab {
    type Message = LeftPanelMessage;

    fn label(&self) -> TabLabel {
        TabLabel::Icon(ICON_PHYSICAL_ENGINE)
    }

    fn content(&self, ui_size: UiSize, app_state: &AppState) -> iced::Element<'_, Self::Message> {
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
                    rigid_grid_is_active.then_some(LeftPanelMessage::RigidGridSimulation),
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
                LeftPanelMessage::VolumeExclusion,
                ui_size,
                true
            ),
            right_checkbox(
                brownian_motion,
                "Unmatched nt jiggling",
                LeftPanelMessage::BrownianMotion,
                ui_size,
                true
            ),
            Column::with_children(
                self.brownian_factory
                    .view(brownian_motion, ui_size.main_text())
            ),
            section("Relaxation", ui_size),
            column![row![
                text_button("Start", ui_size).on_press_maybe(
                    if self.rapier_parameters.is_simulation_running {
                        None
                    } else {
                        Some(LeftPanelMessage::UpdateRapierParameters(
                            apply_parameter_fields(
                                &self.rapier_parameter_fields,
                                &RapierParameters {
                                    is_simulation_running: true,
                                    ..self.rapier_parameters
                                },
                            ),
                        ))
                    }
                ),
                Space::with_width(ui_size.button_spacing()),
                text_button("Stop", ui_size).on_press_maybe(
                    if !self.rapier_parameters.is_simulation_running || sim_state.is_paused() {
                        None
                    } else {
                        Some(LeftPanelMessage::StopSimulation)
                    }
                ),
                Space::with_width(ui_size.button_spacing()),
                text_button("Reset", ui_size).on_press_maybe(
                    sim_state
                        .is_paused()
                        .then(|| LeftPanelMessage::ResetSimulation)
                ),
            ],]
            .spacing(ui_size.button_spacing()),
            ignore_local_parameters_checkbox(&self.rapier_parameters, ui_size),
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

fn ignore_local_parameters_checkbox(
    parameters: &RapierParameters,
    ui_size: UiSize,
) -> iced::Element<'static, LeftPanelMessage> {
    let parameters = *parameters;
    right_checkbox(
        parameters.ignore_local_parameters,
        "Ignore local helix parameters",
        move |value| {
            LeftPanelMessage::UpdateRapierParameters(RapierParameters {
                ignore_local_parameters: value,
                ..parameters
            })
        },
        ui_size,
        !parameters.is_simulation_running,
    )
    .into()
}

/// Updates the parameters using the fields.
fn apply_parameter_fields(
    fields: &HashMap<String, String>,
    parameters: &RapierParameters,
) -> RapierParameters {
    let mut result = *parameters;

    for parameter in RapierFloatParameter::values() {
        if let Some(value) = fields
            .get(&parameter.name().to_owned())
            .and_then(|str| str.parse::<f32>().ok())
        {
            result.set_parameter(parameter, value);
        }
    }

    if let Some(value) = fields.get("Target UPS:") {
        result.target_ups = value.parse::<u32>().unwrap_or(parameters.target_ups);
    }

    result
}

fn rapier_parameters_field_editor(
    parameter: RapierFloatParameter,
    ui_size: UiSize,
    fields: &HashMap<String, String>,
    parameters: &RapierParameters,
    enabled: bool,
) -> iced::Element<'static, LeftPanelMessage> {
    let description = parameter.name().to_owned();
    let default_value = parameters.get_parameter(parameter);
    let default_field_value = default_value.to_string();
    let current_value = fields.get(&description).unwrap_or(&default_field_value);

    let copy = *parameters;

    row![
        row![text(&description)]
            .align_items(Alignment::Center)
            .width(Length::FillPortion(3)),
        Space::with_width(Length::Fill),
        row![
            keyboard_priority(
                "Rapier parameters ".to_owned() + &description,
                LeftPanelMessage::SetKeyboardPriority,
                // if parameters.is_simulation_running {
                //     text_input(current_value, current_value)
                // } else {
                if enabled {
                    text_input(current_value, current_value)
                        .on_input(move |str| {
                            LeftPanelMessage::UpdateRapierParameterField(description.clone(), str)
                        })
                        .on_submit(LeftPanelMessage::UpdateRapierParameters(
                            apply_parameter_fields(fields, parameters),
                        ))
                        // }
                        .width(70)
                } else {
                    text_input(current_value, current_value).width(70)
                }
            ),
            Space::with_width(ui_size.checkbox_spacing()),
            if enabled {
                slider(
                    parameter.min_value()..=parameter.max_value(),
                    parameters
                        .get_parameter(parameter)
                        .clamp(parameter.min_value(), parameter.max_value()),
                    move |value| {
                        LeftPanelMessage::UpdateRapierParameters(
                            copy.with_parameter(parameter, value),
                        )
                    },
                )
                .step(parameter.increment())
                .shift_step(parameter.increment() / 10.0)
                .width(200)
            } else {
                slider(
                    parameter.min_value()..=parameter.max_value(),
                    parameters
                        .get_parameter(parameter)
                        .clamp(parameter.min_value(), parameter.max_value()),
                    |_| LeftPanelMessage::Nothing,
                )
                .width(200)
            }
        ]
        .align_items(Alignment::Center)
        .width(Length::FillPortion(3)),
        Space::with_width(Length::FillPortion(1)),
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn view_ups(
    parameters: RapierParameters,
    fields: &HashMap<String, String>,
    ui_size: UiSize,
) -> iced::Element<'static, LeftPanelMessage> {
    let description = "Target UPS:";
    let default_field_value = parameters.target_ups.to_string();
    let current_value = fields.get(description).unwrap_or(&default_field_value);

    row![
        row![
            text(description),
            Space::with_width(ui_size.checkbox_spacing()),
            checkbox("", parameters.cap_ups).on_toggle(move |value| {
                LeftPanelMessage::UpdateRapierParameters(RapierParameters {
                    cap_ups: value,
                    ..parameters
                })
            }),
        ]
        .align_items(Alignment::Center)
        .width(Length::FillPortion(3)),
        Space::with_width(Length::Fill),
        row![
            keyboard_priority(
                "Rapier parameters ".to_owned() + description,
                LeftPanelMessage::SetKeyboardPriority,
                text_input(current_value, current_value)
                    .on_input(move |str| {
                        LeftPanelMessage::UpdateRapierParameterField(description.to_owned(), str)
                    })
                    .on_submit(LeftPanelMessage::UpdateRapierParameters(RapierParameters {
                        target_ups: current_value
                            .parse::<u32>()
                            .unwrap_or(parameters.target_ups)
                            // prevents division by 0 related crash
                            .max(1),
                        ..parameters
                    }))
                    .width(70),
            ),
            Space::with_width(ui_size.checkbox_spacing()),
            slider(1..=300, parameters.target_ups.clamp(1, 300), move |value| {
                LeftPanelMessage::UpdateRapierParameters(RapierParameters {
                    target_ups: value,
                    ..parameters
                })
            })
            .width(200),
        ]
        .align_items(Alignment::Center)
        .width(Length::FillPortion(3)),
        Space::with_width(Length::FillPortion(1)),
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn view_rapier_parameters(
    parameters: RapierParameters,
    fields: &HashMap<String, String>,
    ui_size: UiSize,
) -> iced::Element<'static, LeftPanelMessage> {
    let mut elements: Vec<iced::Element<'static, LeftPanelMessage>> =
        vec![subsection("Relaxation parameters", ui_size).into()];

    elements.push(view_ups(parameters, fields, ui_size));

    for parameter in RapierFloatParameter::values() {
        let enabled = parameter.live_editability() || !parameters.is_simulation_running;
        elements.push(rapier_parameters_field_editor(
            parameter,
            ui_size,
            fields,
            &parameters,
            enabled,
        ));
    }

    Column::from_vec(elements)
        .width(Length::Fill)
        .spacing(ui_size.button_spacing())
        .into()
}

#[derive(Default)]
struct PhysicalSimulation;

impl PhysicalSimulation {
    fn view(
        &self,
        ui_size: UiSize,
        name: &'static str,
        active: bool,
        running: bool,
    ) -> iced::Element<'_, LeftPanelMessage> {
        let button_str = if running { "Stop" } else { name };
        let mut button = text_button(button_str, ui_size);
        button = if running {
            button.style(iced::theme::Button::Destructive)
        } else {
            button.style(iced::theme::Button::Positive)
        };
        if active {
            button = button.on_press(LeftPanelMessage::RollSimulationRequest);
        }
        row![button].into()
    }

    fn request(&self) -> RollRequest {
        RollRequest {
            target_helices: None,
        }
    }
}
