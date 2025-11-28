
use ensnano_consts::ICON_PHYSICAL_ENGINE;
use crate::ensnano_gui::{
    AppState, Requests,
    left_panel::{
        BrownianParametersFactory, Message, RigidBodyFactory, RigidBodyParametersRequest,
        discrete_value::{FactoryId, RequestFactory, ValueId},
        tabs::{GuiTab, gostop::GoStop},
    },
};
use ensnano_iced::{
    helpers::{right_checkbox, section, start_stop_button, subsection, text_button},
    ui_size::UiSize,
};
use crate::ensnano_interactor::{RollRequest, SimulationState};
use iced::widget::{Column, column, row, scrollable};
use iced_aw::TabLabel;
use std::sync::{Arc, Mutex};

pub struct SimulationTab<State: AppState> {
    rigid_body_factory: RequestFactory<RigidBodyFactory>,
    brownian_factory: RequestFactory<BrownianParametersFactory>,
    //rigid_grid_button: GoStop<State>,
    rigid_helices_button: GoStop<State>,
    physical_simulation: PhysicalSimulation,
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
                text_button("Rapier Simulation", ui_size,).on_press(Message::StartRapierSimulation),
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
        ]
        .spacing(5);

        scrollable(content).into()
    }
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
    ) -> iced::Element<'_, Message<State>, iced::Theme, iced::Renderer> {
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
