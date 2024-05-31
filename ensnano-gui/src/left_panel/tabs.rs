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
use super::color_picker::ColorSquare;
use super::*;
use ensnano_interactor::{RollRequest, SimulationState};
use std::collections::VecDeque;

const MEMORY_COLOR_ROWS: usize = 3;
const MEMORY_COLOR_COLUMNS: usize = 8;
const NB_MEMORY_COLOR: usize = MEMORY_COLOR_ROWS * MEMORY_COLOR_COLUMNS;

mod edition_tab;
pub use edition_tab::EditionTab;
mod grids_tab;
pub use grids_tab::GridTab;
mod camera_shortcut;
pub use camera_shortcut::CameraShortcutPanel;
mod camera_tab;
pub use camera_tab::{CameraTab, FogChoices};
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

struct GoStop<State: AppState> {
    pub name: String,
    on_press: Box<dyn Fn(bool) -> Message<State>>,
}

impl<State: AppState> GoStop<State> {
    fn new<F>(name: String, on_press: F) -> Self
    where
        F: 'static + Fn(bool) -> Message<State>,
    {
        Self {
            name,
            on_press: Box::new(on_press),
        }
    }

    fn view(
        &self,
        active: bool,
        running: bool,
    ) -> iced::Element<Message<State>, crate::Theme, crate::Renderer> {
        use crate::helpers::*;
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
