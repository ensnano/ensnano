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
    color_picker::{ColorPicker, ColorPickerMessage},
    fonts::{icon_to_char, MaterialIcon},
    helpers::*,
    iced_aw::TabLabel,
};
use std::marker::PhantomData;

use super::tabs::GuiTab;
use super::{
    AppState, DesignElementKey, FactoryId, HelixRoll, Message, RequestFactory, RollRequest, UiSize,
    ValueId,
};

pub struct EditionTab<State: AppState> {
    helix_roll_factory: RequestFactory<HelixRoll>,
    color_picker: ColorPicker,
    //_sequence_input: SequenceInput,
    //roll_target_btn: GoStop<State>,
    _state_type: PhantomData<State>,
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

    pub fn current_strand_color(&mut self) -> u32 {
        let color = self.color_picker.current_color();
        super::color_to_u32(color)
    }

    pub fn update_color_picker(&mut self, message: ColorPickerMessage) {
        self.color_picker.update(message)
    }
}

impl<State: AppState> GuiTab<State> for EditionTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Edit)))
    }

    fn update(&mut self, _app_state: &mut State) -> Option<Self::Message> {
        None
    }

    fn content(
        &self,
        ui_size: UiSize,
        app_state: &State,
    ) -> ensnano_iced::Element<'_, Self::Message> {
        let roll_target_helices =
            self.get_roll_target_helices(&app_state.get_selection_as_design_element());
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
                    self.color_picker
                        .view()
                        .map(|m| Message::ColorPickerMessage(m)),
                    //self.color_picker.color_square(),
                    // memory_color_column(&self.memory_color_squares, 4),
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
