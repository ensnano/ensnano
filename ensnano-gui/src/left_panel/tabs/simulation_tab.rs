use crate::{
    AppState, Requests,
    left_panel::{
        BrownianParametersFactory, Message, RigidBodyFactory, RigidBodyParametersRequest,
        discrete_value::{FactoryId, RequestFactory, ValueId},
        tabs::{GuiTab, gostop::GoStop},
    },
};
use ensnano_consts::ICON_PHYSICAL_ENGINE;
use ensnano_iced::{
    helpers::{right_checkbox, section, start_stop_button, subsection, text_button},
    ui_size::UiSize,
    widgets::keyboard_priority::keyboard_priority,
};
use ensnano_interactor::{RollRequest, SimulationState};
use ensnano_physics::parameters::{RapierParameters, RapierSimulationType};
use iced::widget::{Column, column, pick_list, row, scrollable, text, text_input};
use iced_aw::TabLabel;
use std::sync::{Arc, Mutex};

pub struct SimulationTab<State: AppState> {
    rigid_body_factory: RequestFactory<RigidBodyFactory>,
    brownian_factory: RequestFactory<BrownianParametersFactory>,
    //rigid_grid_button: GoStop<State>,
    rigid_helices_button: GoStop<State>,
    physical_simulation: PhysicalSimulation,
    pub rapier_parameters: RapierParameters,
}

impl<State: AppState> SimulationTab<State> {
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

    pub fn leave_tab<R: Requests>(&self, requests: Arc<Mutex<R>>, app_state: &State) {
        if app_state.get_simulation_state() == SimulationState::RigidGrid {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop grids");
        } else if app_state.get_simulation_state() == SimulationState::RigidHelices {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop helices");
        }
    }

    fn request_stop_rigid_body_simulation<R: Requests>(&self, requests: Arc<Mutex<R>>) {
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

impl<State: AppState> GuiTab<State> for SimulationTab<State> {
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

        let content = self::column![
            section("Simulation (Beta)", ui_size),
            self::column![
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
            section("Rapier Simulation", ui_size),
            self::column![
                self::row![pick_list(
                    [
                        RapierSimulationType::Full,
                        RapierSimulationType::Rigid,
                        RapierSimulationType::Cut
                    ],
                    Some(RapierSimulationType::Cut),
                    |_| Message::StartRapierSimulation,
                )],
                self::row![
                    text_button("Start", ui_size).on_press(Message::StartRapierSimulation),
                    text_button("Stop", ui_size),
                ],
            ],
            view_rapier_parameters(self.rapier_parameters, ui_size)
        ]
        .spacing(5);

        scrollable(content).into()
    }
}

fn rapier_parameters_field_editor<State: AppState, F>(
    description: impl ToString,
    value: f32,
    message_builder: F,
) -> iced::Element<'static, Message<State>>
where
    F: Fn(f32) -> Message<State> + 'static,
{
    let current_value = value.to_string();

    let description = description.to_string();

    row![
        text(&description),
        keyboard_priority(
            "Rapier parameters ".to_owned() + &description,
            Message::<State>::SetKeyboardPriority,
            text_input(&current_value, &current_value).on_input(move |str| {
                match str.parse::<f32>() {
                    Ok(new_value) => message_builder(new_value),
                    _ => message_builder(value),
                }
            })
        )
    ]
    .into()
}

fn view_rapier_parameters<State: AppState>(
    parameters: RapierParameters,
    ui_size: UiSize,
) -> iced::Element<'static, Message<State>> {
    self::column![
        subsection("Rapier parameters", ui_size),
        rapier_parameters_field_editor("Linear damping", parameters.linear_damping, move |value| {
            Message::UpdateRapierParameters(RapierParameters {
                linear_damping: value,
                ..parameters
            })
        }),
        rapier_parameters_field_editor(
            "Angular damping",
            parameters.angular_damping,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    angular_damping: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Interbase spring stiffness",
            parameters.interbase_spring_stiffness,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    interbase_spring_stiffness: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Interbase spring damping",
            parameters.interbase_spring_damping,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    interbase_spring_damping: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Crossover stiffness",
            parameters.crossover_stiffness,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    crossover_stiffness: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Crossover damping",
            parameters.crossover_damping,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    crossover_damping: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Crossover rest length",
            parameters.crossover_rest_length,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    crossover_rest_length: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Free nucleotide stiffness",
            parameters.free_nucleotide_stiffness,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    free_nucleotide_stiffness: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Free nucleotide damping",
            parameters.free_nucleotide_damping,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    free_nucleotide_damping: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Free nucleotide rest length",
            parameters.free_nucleotide_rest_length,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    free_nucleotide_rest_length: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Repulsion strength",
            parameters.repulsion_strength,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    repulsion_strength: value,
                    ..parameters
                })
            }
        ),
        rapier_parameters_field_editor(
            "Repulsion range",
            parameters.repulsion_range,
            move |value| {
                Message::UpdateRapierParameters(RapierParameters {
                    repulsion_range: value,
                    ..parameters
                })
            }
        ),
    ]
    .into()
}

#[derive(Default)]
struct PhysicalSimulation;

impl PhysicalSimulation {
    fn view<State: AppState>(
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
