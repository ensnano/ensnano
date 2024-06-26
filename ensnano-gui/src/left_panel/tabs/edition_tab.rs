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
use ensnano_iced::{
    fonts::{icon_to_char, MaterialIcon},
    helpers::*,
    iced_aw::TabLabel,
};
use std::marker::PhantomData;

use super::tabs::GuiTab;
use super::{
    AppState, Color, ColorPicker, ColorSquare, DesignElementKey, FactoryId, HelixRoll, Length,
    Message, RequestFactory, RollRequest, UiSize, ValueId, VecDeque, MEMORY_COLOR_COLUMNS,
    MEMORY_COLOR_ROWS, NB_MEMORY_COLOR,
};

pub struct EditionTab<State: AppState> {
    helix_roll_factory: RequestFactory<HelixRoll>,
    color_picker: ColorPicker,
    //_sequence_input: SequenceInput,
    //roll_target_btn: GoStop<State>,
    memory_color_squares: VecDeque<MemoryColorSquare>,
    _state_type: PhantomData<State>,
}

/// An entry of the stack of last picked colors.
struct MemoryColorSquare {
    color: Color,
}

impl PartialEq<MemoryColorSquare> for MemoryColorSquare {
    fn eq(&self, other: &MemoryColorSquare) -> bool {
        self.color == other.color
    }
}

impl MemoryColorSquare {
    fn new(color: Color) -> Self {
        Self { color }
    }
}

/// Arrange memory colors in a few rows.
fn memory_color_column<State: AppState>(
    memory_color_squares: &VecDeque<MemoryColorSquare>,
    fill_portion: u16,
) -> iced::Element<Message<State>, ensnano_iced::Theme, crate::Renderer> {
    let mut content = Vec::with_capacity(MEMORY_COLOR_ROWS);
    let mut current_row = Vec::with_capacity(MEMORY_COLOR_COLUMNS);
    for memory_color_square in memory_color_squares.iter() {
        if current_row.len() >= MEMORY_COLOR_COLUMNS {
            // Create a new row
            content.push(row(current_row).into());
            current_row = Vec::with_capacity(MEMORY_COLOR_COLUMNS);
        }
        // Append to row
        let color_square = ColorSquare::new(
            memory_color_square.color,
            Message::ColorPicked,
            Message::FinishChangingColor,
        );
        current_row.push(color_square.into());
    }
    column(content)
        .width(Length::FillPortion(fill_portion))
        .into()
}

impl<State: AppState> EditionTab<State> {
    pub fn new() -> Self {
        Self {
            helix_roll_factory: RequestFactory::new(FactoryId::HelixRoll, HelixRoll {}),
            color_picker: ColorPicker::new(),
            //_sequence_input: SequenceInput::new(),
            //roll_target_btn: GoStop::new(
            //    "Autoroll selected helices".to_owned(),
            //    Message::RollTargeted,
            //),
            memory_color_squares: VecDeque::new(),
            _state_type: PhantomData,
        }
    }

    fn get_roll_target_helices(&self, selection: &[DesignElementKey]) -> Vec<usize> {
        let mut ret = vec![];
        for s in selection.iter() {
            if let DesignElementKey::Helix(h) = s {
                ret.push(*h)
            }
        }
        ret
    }

    pub fn update_roll_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<f32>,
    ) {
        self.helix_roll_factory
            .update_request(value_id, value, request);
    }

    pub fn get_roll_request(&mut self, selection: &[DesignElementKey]) -> Option<RollRequest> {
        let roll_target_helices = self.get_roll_target_helices(selection);
        if roll_target_helices.len() > 0 {
            Some(RollRequest {
                roll: true,
                springs: false,
                target_helices: Some(roll_target_helices.clone()),
            })
        } else {
            None
        }
    }

    pub fn strand_color_change(&mut self) -> u32 {
        let color = self.color_picker.update_color();
        super::color_to_u32(color)
    }

    pub fn change_sat_value(&mut self, sat: f64, hsv_value: f64) {
        self.color_picker.set_hsv_value(hsv_value);
        self.color_picker.set_saturation(sat);
    }

    pub fn change_hue(&mut self, hue: f64) {
        self.color_picker.change_hue(hue)
    }

    pub fn add_color(&mut self) {
        let color = self.color_picker.update_color();
        let memory_color = MemoryColorSquare::new(color);
        if !self.memory_color_squares.contains(&memory_color) {
            log::info!("adding color");
            self.memory_color_squares.push_front(memory_color);
            self.memory_color_squares.truncate(NB_MEMORY_COLOR);
            log::info!("color len {}", self.memory_color_squares.len());
        }
    }
}

impl<State: AppState> GuiTab<State> for EditionTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Edit)))
    }

    fn content(
        &self,
        ui_size: UiSize,
        app_state: &State,
    ) -> iced::Element<Self::Message, ensnano_iced::Theme, crate::Renderer> {
        let roll_target_helices =
            self.get_roll_target_helices(&app_state.get_selection_as_designelement());
        let sim_state = &app_state.get_simulation_state();
        let autoroll_is_active = sim_state.is_rolling() || roll_target_helices.len() > 0;
        let selection_contains_strand =
            ensnano_interactor::extract_strands_from_selection(app_state.get_selection()).len() > 0;
        let suggestion_parameters = app_state.get_suggestion_parameters().clone();
        let mut tighten_helices_button = text_button("Selected", ui_size);
        if !roll_target_helices.is_empty() {
            tighten_helices_button =
                tighten_helices_button.on_press(Message::Redim2dHelices(false));
        }

        let content = self::column![
            section("Edition", ui_size),
            // add_roll_slider!
            column(
                self.helix_roll_factory
                    .view(roll_target_helices.len() >= 1, ui_size.intermediate_text())
            ),
            // add_autoroll_button!
            start_stop_button(
                "Autoroll selected helices",
                ui_size,
                if autoroll_is_active {
                    Some(Message::RollTargeted)
                } else {
                    None
                },
                sim_state.is_rolling()
            ),
            // add_color_square!
            if selection_contains_strand {
                row![
                    self.color_picker.view(),
                    self.color_picker.color_square(),
                    memory_color_column(&self.memory_color_squares, 4),
                ]
            } else {
                row![]
            },
            subsection("Suggestions Parameters", ui_size),
            // add_suggestion_parameters_checkboxes!
            right_checkbox(
                suggestion_parameters.include_scaffold,
                "Include scaffold",
                move |b| {
                    Message::NewSuggestionParameters(suggestion_parameters.with_include_scaffod(b))
                },
                ui_size,
            ),
            right_checkbox(
                suggestion_parameters.include_intra_strand,
                "Intra strand suggestions",
                move |b| Message::NewSuggestionParameters(
                    suggestion_parameters.with_intra_strand(b)
                ),
                ui_size,
            ),
            right_checkbox(
                suggestion_parameters.include_xover_ends,
                "Include Xover ends",
                move |b| Message::NewSuggestionParameters(suggestion_parameters.with_xover_ends(b)),
                ui_size,
            ),
            right_checkbox(
                suggestion_parameters.ignore_groups,
                "All helices",
                move |b| Message::NewSuggestionParameters(
                    suggestion_parameters.with_ignore_groups(b)
                ),
                ui_size,
            ),
            subsection("Tighten 2D helices", ui_size),
            // add_tighten_helices_button!
            row![
                tighten_helices_button,
                text_button("All", ui_size).on_press(Message::Redim2dHelices(true)),
            ]
            .spacing(ui_size.button_spacing()),
        ]
        .spacing(5);

        scrollable(content).into()
    }
}
