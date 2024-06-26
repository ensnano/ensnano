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
use ensnano_iced::{helpers::*, iced::Element};

pub struct SequenceInput {
    #[allow(dead_code)]
    text_input_state: text_input::State<iced_graphics::text::Paragraph>,
    sequence: String,
}

impl SequenceInput {
    pub fn new() -> Self {
        Self {
            text_input_state: Default::default(),
            sequence: String::new(),
        }
    }

    #[allow(dead_code)]
    pub fn view<'a, S>(&'a mut self) -> Element<'a, Message<S>>
    where
        S: AppState,
    {
        row![
            Space::with_width(5),
            text_input("Sequence", &self.sequence).on_input(Message::SequenceChanged,),
            button(text("Load File")).on_press(Message::SequenceFileRequested),
        ]
        .into()
    }

    pub fn update_sequence(&mut self, sequence: String) {
        self.sequence = sequence;
    }

    pub fn has_keyboard_priority(&self) -> bool {
        self.text_input_state.is_focused()
    }
}
