use super::*;
use crate::ensnano_interactor::{RollRequest, SimulationState};

mod edition_tab;
pub use edition_tab::EditionTab;
mod grids_tab;
pub use grids_tab::GridTab;
mod camera_shortcut;
pub use camera_shortcut::CameraShortcutPanel;
mod camera_tab;
pub use camera_tab::{CameraTab, FogChoices};
mod simulation_tab;
use iced::widget::container;
pub use simulation_tab::SimulationTab;
mod parameters_tab;
pub use parameters_tab::ParametersTab;
mod sequence_tab;
pub use sequence_tab::SequenceTab;
mod pen_tab;
pub use pen_tab::PenTab;
pub(super) mod revolution_tab;

pub use gostop::*;
// TODO: Move gostop to widgets.

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TabId {
    Grid,
    Edition,
    Camera,
    Simulation,
    Sequence,
    Parameters,
    Pen,
    Revolution,
}

pub trait GuiTab<State: AppState> {
    type Message;

    fn label(&self) -> TabLabel;

    fn update(&mut self, _app_state: &mut State) -> Command<Message<State>> {
        Command::none()
    }

    fn view(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Self::Message> {
        container(self.content(ui_size, app_state))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn content(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Self::Message>;
}

mod gostop {
    // TODO: Turn this into a widget
    use super::{AppState, Message};
    use crate::ensnano_iced::helpers::*;

    pub struct GoStop<State: AppState> {
        pub name: String,
        on_press: Box<dyn Fn(bool) -> Message<State>>,
        // TODO: Use a checkbox-like approach with Option<Box<…>>
    }

    impl<State: AppState> GoStop<State> {
        pub fn new<F>(name: String, on_press: F) -> Self
        where
            F: 'static + Fn(bool) -> Message<State>,
        {
            Self {
                name,
                on_press: Box::new(on_press),
            }
        }

        pub fn view(
            &self,
            active: bool,
            running: bool,
        ) -> iced::Element<'_, Message<State>, Theme, Renderer> {
            let button_str = if running {
                "Stop".to_owned()
            } else {
                self.name.clone()
            };
            //let mut button = button(text(button_str)).style(ButtonColor::red_green(running));
            let mut button = button(text(button_str)).style(iced::theme::Button::Positive);
            // This is a dirty fix to compile.
            if active {
                button = button.on_press((self.on_press)(!running));
            }
            row![button].into()
        }
    }
}
