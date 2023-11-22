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
use super::color_picker::{ColorSquare, ColorState};
use super::*;
use ensnano_design::CameraId;
use ensnano_interactor::{RollRequest, SimulationState};
use std::collections::VecDeque;

const MEMORY_COLOR_ROWS: usize = 3;
const MEMORY_COLOR_COLUMN: usize = 8;
const NB_MEMORY_COLOR: usize = MEMORY_COLOR_ROWS * MEMORY_COLOR_COLUMN;
const JUMP_SIZE: f32 = 4.0;

use super::super::material_icons_light;
use material_icons_light::LightIcon;
const LIGHT_ICONFONT: iced::Font = iced::Font::External {
    name: "IconFontLight",
    bytes: material_icons_light::MATERIAL_ICON_LIGHT,
};
fn light_icon<'a>(icon: LightIcon, ui_size: UiSize) -> iced::widget::Text<'a> {
    iced::widget::Text::new(format!("{}", material_icons_light::icon_to_char(icon)))
        .font(LIGHT_ICONFONT)
        .size(ui_size.icon())
}

fn light_icon_btn<'a, Message: Clone>(
    icon: LightIcon,
    ui_size: UiSize,
) -> iced::widget::Button<'a, Message> {
    let content = light_icon(icon, ui_size);
    Button::new(content).height(ui_size.button())
}

/// Additional Iced widget helpers
mod helpers {
    use super::{UiSize, JUMP_SIZE};
    use iced::Length;
    use iced_native::widget::{self, helpers::*};

    /// Section title widget
    pub(crate) fn section<'a, R>(title: &'a str, ui_size: UiSize) -> widget::Text<'a, R>
    where
        R: iced_native::text::Renderer,
        R::Theme: widget::text::StyleSheet,
    {
        text(title).size(ui_size.head_text())
    }

    /// Section subtitle widget
    pub(crate) fn subsection<'a, R>(title: &'a str, ui_size: UiSize) -> widget::Text<'a, R>
    where
        R: iced_native::text::Renderer,
        R::Theme: widget::text::StyleSheet,
    {
        text(title).size(ui_size.intermediate_text())
    }

    /// Add vertical space of [JUMP_SIZE] amount
    pub(crate) fn extra_jump() -> widget::Space {
        jump_by(JUMP_SIZE)
    }

    /// Add vertical space of specified amount.
    pub(crate) fn jump_by(amount: impl Into<Length>) -> widget::Space {
        vertical_space(amount)
    }
}

mod edition_tab;
pub use edition_tab::EditionTab;
mod grids_tab;
pub use grids_tab::GridTab;
mod camera_shortcut;
pub use camera_shortcut::CameraShortcut;
mod camera_tab;
pub use camera_tab::{CameraTab, FogChoice};
mod simulation_tab;
pub use simulation_tab::SimulationTab;
mod parameters_tab;
pub use parameters_tab::ParametersTab;
mod sequence_tab;
pub use sequence_tab::SequenceTab;
mod pen_tab;
pub use pen_tab::PenTab;
pub(super) mod revolution_tab;
pub use revolution_tab::*;

struct GoStop<S: AppState> {
    pub name: String,
    on_press: Box<dyn Fn(bool) -> Message<S>>,
}

impl<S: AppState> GoStop<S> {
    fn new<F>(name: String, on_press: F) -> Self
    where
        F: 'static + Fn(bool) -> Message<S>,
    {
        Self {
            name,
            on_press: Box::new(on_press),
        }
    }

    fn view(&self, active: bool, running: bool) -> iced::Element<Message<S>> {
        use iced_native::widget::helpers::*;
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
        iced::widget::row![button].into()
    }
}
