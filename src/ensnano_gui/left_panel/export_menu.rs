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

use super::{AppState, ExportType, Message};
use crate::ensnano_iced::helpers::*;

#[derive(Default)]
pub struct ExportMenu {}

impl ExportMenu {
    pub fn view<State>(&self) -> iced::Element<'_, Message<State>>
    where
        State: AppState,
    {
        let content = self::column![
            button(text("Cancel")).on_press(Message::CancelExport),
            button(text("Oxdna")).on_press(Message::Export(ExportType::Oxdna)),
            button(text("Pdb")).on_press(Message::Export(ExportType::Pdb)),
            button(text("Cadnano")).on_press(Message::Export(ExportType::Cadnano)),
        ];

        scrollable(content).into()
    }
}
