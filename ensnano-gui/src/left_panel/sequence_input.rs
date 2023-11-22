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
use iced_native::{row, widget::helpers::horizontal_space};
use iced_native::{widget, widget::text_input};

pub struct SequenceInput {
    #[allow(dead_code)]
    text_input_state: text_input::State,
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
    pub fn view<S, R>(&mut self) -> widget::Row<Message<S>, R>
    where
        S: AppState,
        R: iced_native::text::Renderer,
        R::Theme: text_input::StyleSheet,
        R::Theme: widget::text::StyleSheet,
        R::Theme: widget::button::StyleSheet,
    {
        row![
            horizontal_space(5),
            text_input::TextInput::new("Sequence", &self.sequence, Message::SequenceChanged,),
            widget::Button::new(widget::Text::new("Load File"))
                .on_press(Message::SequenceFileRequested),
        ]
    }

    pub fn update_sequence(&mut self, sequence: String) {
        self.sequence = sequence;
    }

    pub fn has_keyboard_priority(&self) -> bool {
        self.text_input_state.is_focused()
    }
}
