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
use super::tabs::GuiTab;
use super::*;
use ensnano_consts::ICON_PHYSICAL_ENGINE;
use ensnano_iced::{helpers::*, iced_aw::TabLabel};
use ensnano_physics::parameters::{RapierParameters, RapierSimulationType};

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
            //rigid_grid_button: GoStop::new(
            //    String::from("Rigid Grids"),
            //    Message::RigidGridSimulation,
            //),
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

    pub fn make_rigid_body_request(&mut self, request: &mut Option<RigidBodyParametersRequest>) {
        self.rigid_body_factory.make_request(request)
    }

    pub fn update_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<RigidBodyParametersRequest>,
    ) {
        self.rigid_body_factory
            .update_request(value_id, value, request)
    }

    pub fn update_brownian(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<RigidBodyParametersRequest>,
    ) {
        let new_brownian = self.brownian_factory.update_value(value_id, value);
        self.rigid_body_factory.requestable.brownian_parameters = new_brownian;
        self.rigid_body_factory.make_request(request)
    }

    pub fn get_physical_simulation_request(&self) -> RollRequest {
        self.physical_simulation.request()
    }

    pub fn leave_tab<R: Requests>(&mut self, requests: Arc<Mutex<R>>, app_state: &State) {
        if app_state.get_simulation_state() == SimulationState::RigidGrid {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop grids");
        } else if app_state.get_simulation_state() == SimulationState::RigidHelices {
            self.request_stop_rigid_body_simulation(requests);
            println!("stop helices");
        }
    }

    fn request_stop_rigid_body_simulation<R: Requests>(&mut self, requests: Arc<Mutex<R>>) {
        let mut request = None;
        self.make_rigid_body_request(&mut request);
        if let Some(request) = request {
            requests
                .lock()
                .unwrap()
                .update_rigid_body_simulation_parameters(request)
        }
    }

    fn helix_btns<'a>(
        go_stop: &'a GoStop<State>,
        app_state: &State,
        ui_size: UiSize,
    ) -> ensnano_iced::Element<'a, Message<State>> {
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
            go_stop
                .view(helices_active, sim_state.simulating_helices())
                .into()
        }
    }
}

impl<State: AppState> GuiTab<State> for SimulationTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Icon(ICON_PHYSICAL_ENGINE)
    }

    fn content(
        &self,
        ui_size: UiSize,
        app_state: &State,
    ) -> ensnano_iced::Element<'_, Self::Message> {
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
                    if rigid_grid_is_active {
                        Some(Message::RigidGridSimulation)
                    } else {
                        None
                    },
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
) -> ensnano_iced::Element<'static, Message<State>, ensnano_iced::Theme, ensnano_iced::Renderer>
where
    F: Fn(f32) -> Message<State> + 'static,
{
    let current_value = value.to_string();

    row![
        text(description),
        text_input(&current_value, &current_value).on_input(move |str| {
            println!("str : {str}");
            match str.parse::<f32>() {
                Ok(new_value) => {
                    println!("new value : {new_value}");
                    message_builder(new_value)
                }
                _ => message_builder(value),
            }
        })
    ]
    .into()
}

fn view_rapier_parameters<State: AppState>(
    parameters: RapierParameters,
    ui_size: UiSize,
) -> ensnano_iced::Element<'static, Message<State>, ensnano_iced::Theme, ensnano_iced::Renderer> {
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
struct PhysicalSimulation {}

impl PhysicalSimulation {
    fn view<'b, State: AppState>(
        &self,
        ui_size: UiSize,
        name: &'static str,
        active: bool,
        running: bool,
    ) -> ensnano_iced::Element<'_, Message<State>, ensnano_iced::Theme, ensnano_iced::Renderer>
    {
        let button_str = if running { "Stop" } else { name };
        let mut button = text_button(button_str, ui_size);
        button = if running {
            button.style(theme::Button::Destructive)
        } else {
            button.style(theme::Button::Positive)
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
