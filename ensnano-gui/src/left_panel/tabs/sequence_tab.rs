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
use ensnano_interactor::StandardSequence;

use super::*;
use crate::helpers::*;
use iced::Element;
use iced_native::widget;

pub struct SequenceTab {
    toggle_text_value: bool,
    scaffold_position_str: String,
    scaffold_position: usize,
    scaffold_input: widget::text_input::State,
}

macro_rules! scaffold_length_fmt {
    () => {
        "Length: {} nt"
    };
}

macro_rules! nucl_text_fmt {
    () => {
        "   Helix #{}\n   Strand: {}\n   Nt #{}"
    };
}

fn get_sequence_name(sequence: &str) -> &'static str {
    let n = sequence.len();
    let candidate = StandardSequence::from_length(n);
    if sequence == candidate.sequence() {
        candidate.description()
    } else {
        "custom"
    }
}

impl SequenceTab {
    pub fn new() -> Self {
        Self {
            toggle_text_value: false,
            scaffold_position_str: "0".to_string(),
            scaffold_position: 0,
            scaffold_input: Default::default(),
        }
    }

    pub fn view<S: AppState>(&self, ui_size: UiSize, app_state: &S) -> Element<Message<S>> {
        // TODO: This update should happen, but somewhere else in the code.
        //       I think it must happen inside LeftPanel::update
        //
        //if !self.scaffold_input.is_focused() {
        //    if let Some(n) = app_state.get_scaffold_info().and_then(|info| info.shift) {
        //        self.update_pos_str(n.to_string());
        //    }
        //}

        let content = iced_native::column![
            section("Sequence", ui_size),
            extra_jump(),
            // add_show_sequence_button!
            {
                let button_show_sequence = if self.toggle_text_value {
                    text_button("Hide Sequences", ui_size).on_press(Message::ToggleText(false))
                } else {
                    text_button("Show Sequences", ui_size).on_press(Message::ToggleText(true))
                };
                button_show_sequence
            },
            extra_jump(),
            section("Scaffold", ui_size),
            extra_jump(),
            // add_scaffold_from_to_selection_buttons!
            {
                let mut button_selection_to_scaffold = text_button("From selection", ui_size);
                let mut button_selection_from_scaffold = text_button("Show", ui_size);
                if app_state.get_scaffold_info().is_some() {
                    button_selection_from_scaffold =
                        button_selection_from_scaffold.on_press(Message::SelectScaffold);
                }
                let selection = app_state.get_selection_as_designelement();
                if let Some(n) = Self::get_candidate_scaffold(&selection) {
                    button_selection_to_scaffold =
                        button_selection_to_scaffold.on_press(Message::ScaffoldIdSet(n, true));
                }
                iced_native::row![button_selection_to_scaffold, button_selection_from_scaffold,]
                    .spacing(ui_size.button_pad())
            },
            extra_jump(),
            // add_scaffold_info!
            {
                let (scaffold_text, length_text) = if let Some(info) = app_state.get_scaffold_info()
                {
                    (
                        format!("Strand #{}", info.id),
                        format!(scaffold_length_fmt!(), info.length),
                    )
                } else {
                    (
                        "NOT SET".to_owned(),
                        format!(scaffold_length_fmt!(), "—").to_owned(),
                    )
                };
                let mut length_text = text(length_text);
                if app_state.get_scaffold_info().is_none() {
                    length_text = length_text.style(iced::theme::Text::Color(innactive_color()))
                }
                iced_native::column![text(scaffold_text).size(ui_size.main_text()), length_text,]
            },
            extra_jump(),
            // add_rainbow_scaffold_checkbox!
            right_checkbox(
                app_state.get_reader().rainbow_scaffold(),
                "Rainbow Scaffold",
                Message::RainbowScaffold,
                ui_size,
            ),
            extra_jump(),
            // add_set_scaffold_sequence_button!
            button(text("Set scaffold sequence"))
                .height(ui_size.button())
                .on_press(Message::SetScaffoldSeqButtonPressed),
            // show_current_sequence_name!
            {
                let name = app_state
                    .get_reader()
                    .get_scaffold_sequence()
                    .map(get_sequence_name)
                    .unwrap_or("None");
                text(format!("current sequence: {name}"))
            },
            extra_jump(),
            // add_scaffold_position_input_row!
            iced_native::row![
                text("Starting position").width(Length::FillPortion(2)),
                text_input("Scaffold position", &self.scaffold_position_str)
                    .on_input(Message::ScaffoldPositionInput,)
                    .style(BadValue(
                        self.scaffold_position_str == self.scaffold_position.to_string(),
                    ))
                    .width(iced::Length::FillPortion(1)),
            ],
            // add_optimize_scaffold_shift_button!
            button(text("Optimize starting position"))
                .height(ui_size.button())
                .on_press(Message::OptimizeScaffoldShiftPressed),
            // add_scaffold_start_position!
            {
                let starting_nucl = app_state
                    .get_scaffold_info()
                    .as_ref()
                    .and_then(|info| info.starting_nucl);
                let nucl_text = if let Some(nucl) = starting_nucl {
                    format!(
                        nucl_text_fmt!(),
                        nucl.helix,
                        if nucl.forward {
                            "→ forward"
                        } else {
                            "← backward"
                        },
                        nucl.position
                    )
                } else {
                    format!(nucl_text_fmt!(), " —", " —", " —")
                };
                let mut nucl_text = text(nucl_text).size(ui_size.main_text());
                if starting_nucl.is_none() {
                    nucl_text = nucl_text.style(iced::theme::Text::Color(innactive_color()))
                }
                nucl_text
            },
            extra_jump(),
            section("Staples", ui_size),
            extra_jump(),
            // add_download_staples_button!
            iced_native::column![
                button(text("Export Staples"))
                    .height(ui_size.button())
                    .on_press(Message::StapplesRequested),
                button(text("Export Origamis"))
                    .height(ui_size.button())
                    .on_press(Message::OrigamisRequested),
            ]
            .spacing(ui_size.button_pad()),
        ];
        scrollable(content).into()
    }

    pub fn toggle_text_value(&mut self, b: bool) {
        self.toggle_text_value = b;
    }

    pub fn update_pos_str(&mut self, position_str: String) -> Option<usize> {
        self.scaffold_position_str = position_str;
        if let Ok(pos) = self.scaffold_position_str.parse::<usize>() {
            self.scaffold_position = pos;
            Some(pos)
        } else {
            None
        }
    }

    pub fn has_keyboard_priority(&self) -> bool {
        self.scaffold_input.is_focused()
    }

    fn get_candidate_scaffold(selection: &[DesignElementKey]) -> Option<usize> {
        if selection.len() == 1 {
            if let DesignElementKey::Strand(n) = selection[0] {
                Some(n)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_scaffold_shift(&self) -> usize {
        self.scaffold_position
    }
}
