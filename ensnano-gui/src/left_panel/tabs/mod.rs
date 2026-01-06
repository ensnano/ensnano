pub mod camera_shortcut;
pub mod camera_tab;
pub mod edition_tab;
pub mod grids_tab;
pub mod parameters_tab;
pub mod pen_tab;
pub mod revolution_tab;
pub mod sequence_tab;
pub mod simulation_tab;

use crate::{AppState, left_panel::Message};
use ensnano_utils::ui_size::UiSize;
use iced::{Command, Length, widget::container};
use iced_aw::TabLabel;

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

// TODO: Turn this into a widget
pub mod gostop {
    use crate::{AppState, left_panel::Message};
    use iced::widget::{button, row, text};

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

        pub fn view(&self, active: bool, running: bool) -> iced::Element<'_, Message<State>> {
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
