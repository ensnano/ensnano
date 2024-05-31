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

use super::*;
use crate::helpers::*;

pub struct SimulationTab<S: AppState> {
    rigid_body_factory: RequestFactory<RigidBodyFactory>,
    brownian_factory: RequestFactory<BrownianParametersFactory>,
    rigid_grid_button: GoStop<S>,
    rigid_helices_button: GoStop<S>,
    physical_simulation: PhysicalSimulation,
}

impl<S: AppState> SimulationTab<S> {
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
            rigid_grid_button: GoStop::new(
                String::from("Rigid Grids"),
                Message::RigidGridSimulation,
            ),
            physical_simulation: Default::default(),
        }
    }

    pub fn view(
        &self,
        ui_size: UiSize,
        app_state: &S,
    ) -> iced::Element<Message<S>, crate::Theme, crate::Renderer> {
        let sim_state = &app_state.get_simulation_state();
        let rigid_grid_is_active = sim_state.is_none() || sim_state.simulating_grid();
        let roll_active = sim_state.is_none() || sim_state.is_rolling();

        let volume_exclusion = self.rigid_body_factory.requestable.volume_exclusion;
        let brownian_motion = self.rigid_body_factory.requestable.brownian_motion;

        let content = self::column![
            section("Simulation (Beta)", ui_size),
            self::column![
                self.physical_simulation.view(
                    &ui_size,
                    "Roll",
                    roll_active,
                    sim_state.is_rolling(),
                ),
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
            .spacing(ui_size.button_pad()),
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
        ]
        .spacing(5);

        scrollable(content).into()
    }

    fn helix_btns<'a>(
        go_stop: &'a GoStop<S>,
        app_state: &S,
        ui_size: UiSize,
    ) -> iced::Element<'a, Message<S>, crate::Theme, crate::Renderer> {
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

    pub fn leave_tab<R: Requests>(&mut self, requests: Arc<Mutex<R>>, app_state: &S) {
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
}

#[derive(Default)]
struct PhysicalSimulation {}

impl PhysicalSimulation {
    fn view<'b, State: AppState>(
        &self,
        _ui_size: &'b UiSize,
        name: &'static str,
        active: bool,
        running: bool,
    ) -> iced::Element<Message<State>, crate::Theme, crate::Renderer> {
        let button_str = if running { "Stop" } else { name };
        let mut button = button(text(button_str));
        button = if running {
            button.style(theme::Button::Destructive)
        } else {
            button.style(theme::Button::Positive)
        };
        if active {
            button = button.on_press(Message::SimRequest);
        }
        row![button].into()
    }

    fn request(&self) -> RollRequest {
        RollRequest {
            roll: true,
            springs: false,
            target_helices: None,
        }
    }
}
