pub mod camera_shortcut;
pub mod camera_tab;
pub mod edition_tab;
pub mod grids_tab;
pub mod parameters_tab;
pub mod pen_tab;
pub mod revolution_tab;
pub mod sequence_tab;
pub mod simulation_tab;

use crate::left_panel::LeftPanelMessage;
use ensnano_state::app_state::AppState;
use ensnano_utils::ui_size::UiSize;
use iced::{Command, Length, widget::container};
use iced_aw::TabLabel;

pub trait GuiTab {
    type Message;

    fn label(&self) -> TabLabel;

    fn update(&mut self, _app_state: &mut AppState) -> Command<LeftPanelMessage> {
        Command::none()
    }

    fn view(&self, ui_size: UiSize, app_state: &AppState) -> iced::Element<'_, Self::Message> {
        container(self.content(ui_size, app_state))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn content(&self, ui_size: UiSize, app_state: &AppState) -> iced::Element<'_, Self::Message>;
}

// TODO: Turn this into a widget
pub mod gostop {
    use crate::left_panel::LeftPanelMessage;
    use iced::widget::{button, row, text};

    pub struct GoStop {
        pub name: String,
        on_press: Box<dyn Fn(bool) -> LeftPanelMessage>,
        // TODO: Use a checkbox-like approach with Option<Box<…>>
    }

    impl GoStop {
        pub fn new<F>(name: String, on_press: F) -> Self
        where
            F: 'static + Fn(bool) -> LeftPanelMessage,
        {
            Self {
                name,
                on_press: Box::new(on_press),
            }
        }

        pub fn view(&self, active: bool, running: bool) -> iced::Element<'_, LeftPanelMessage> {
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
