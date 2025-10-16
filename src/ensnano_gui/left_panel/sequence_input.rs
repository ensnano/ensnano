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
use super::{AppState, Message};
use ensnano_iced::helpers::*;

pub struct SequenceInput {
    #[allow(dead_code)]
    sequence: String,
}

impl SequenceInput {
    pub fn new() -> Self {
        Self {
            sequence: String::new(),
        }
    }

    #[allow(dead_code)]
    pub fn view<'a, S>(&'a mut self) -> ensnano_iced::Element<'a, Message<S>>
    where
        S: AppState,
    {
        row![
            Space::with_width(5),
            keyboard_priority(
                text_input("Sequence", &self.sequence).on_input(Message::SequenceChanged,),
            )
            .on_priority(Message::SetKeyboardPriority(true))
            .on_unpriority(Message::SetKeyboardPriority(false)),
            button(text("Load File")).on_press(Message::SequenceFileRequested),
        ]
        .into()
    }

    pub fn update_sequence(&mut self, sequence: String) {
        self.sequence = sequence;
    }
}
