use crate::{
    helpers::{extra_jump, right_checkbox, section, text_button},
    left_panel::{LeftPanelMessage, tabs::GuiTab},
    theme,
};
use ensnano_design::design_element::DesignElementKey;
use ensnano_state::gui::state::GuiAppState;
use ensnano_utils::{
    StandardSequence, consts::ICON_ATGC, keyboard_priority::keyboard_priority, ui_size::UiSize,
};
use iced::{
    Length,
    widget::{column, row, scrollable, text, text_input},
};
use iced_aw::TabLabel;
use std::marker::PhantomData;

pub struct SequenceTab<State: GuiAppState> {
    toggle_text_value: bool,
    scaffold_position_str: String,
    scaffold_position: usize,
    _state_type: PhantomData<State>,
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

impl<State: GuiAppState> SequenceTab<State> {
    pub fn new() -> Self {
        Self {
            toggle_text_value: false,
            scaffold_position_str: "0".to_owned(),
            scaffold_position: 0,
            _state_type: PhantomData,
        }
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

impl<State: GuiAppState> GuiTab<State> for SequenceTab<State> {
    type Message = LeftPanelMessage<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Icon(ICON_ATGC)
    }

    fn content(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Self::Message> {
        // TODO: This update should happen, but somewhere else in the code.
        //       I think it must happen inside LeftPanel::update
        //
        //if !self.scaffold_input.is_focused() {
        //    if let Some(n) = app_state.get_scaffold_info().and_then(|info| info.shift) {
        //        self.update_pos_str(n.to_string());
        //    }
        //}

        let content = column![
            section("Sequence", ui_size),
            extra_jump(),
            // add_show_sequence_button!
            {
                if self.toggle_text_value {
                    text_button("Hide Sequences", ui_size)
                        .on_press(LeftPanelMessage::ToggleText(false))
                } else {
                    text_button("Show Sequences", ui_size)
                        .on_press(LeftPanelMessage::ToggleText(true))
                }
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
                        button_selection_from_scaffold.on_press(LeftPanelMessage::SelectScaffold);
                }
                let selection = app_state.get_selection_as_design_element();
                if let Some(n) = Self::get_candidate_scaffold(&selection) {
                    button_selection_to_scaffold = button_selection_to_scaffold
                        .on_press(LeftPanelMessage::ScaffoldIdSet(n, true));
                }
                row![button_selection_to_scaffold, button_selection_from_scaffold,]
                    .spacing(ui_size.button_spacing())
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
                    ("NOT SET".to_owned(), format!(scaffold_length_fmt!(), "—"))
                };
                let mut length_text = text(length_text);
                if app_state.get_scaffold_info().is_none() {
                    length_text = length_text.style(theme::DISABLED_TEXT);
                }
                column![text(scaffold_text).size(ui_size.main_text()), length_text,]
            },
            extra_jump(),
            // add_rainbow_scaffold_checkbox!
            right_checkbox(
                app_state.get_reader().rainbow_scaffold(),
                "Rainbow Scaffold",
                LeftPanelMessage::RainbowScaffold,
                ui_size,
            ),
            extra_jump(),
            // add_set_scaffold_sequence_button!
            text_button("Set scaffold sequence", ui_size)
                .on_press(LeftPanelMessage::SetScaffoldSeqButtonPressed),
            // show_current_sequence_name!
            {
                let name = app_state
                    .get_reader()
                    .get_scaffold_sequence()
                    .map_or("None", get_sequence_name);
                text(format!("current sequence: {name}"))
            },
            extra_jump(),
            // add_scaffold_position_input_row!
            row![
                text("Starting position").width(Length::FillPortion(2)),
                keyboard_priority(
                    "Scaffold position",
                    LeftPanelMessage::SetKeyboardPriority,
                    text_input("Scaffold position", &self.scaffold_position_str)
                        .on_input(LeftPanelMessage::ScaffoldPositionInput,)
                        .style(theme::BadValue(
                            self.scaffold_position_str == self.scaffold_position.to_string(),
                        ))
                )
                .width(Length::FillPortion(1))
            ],
            // add_optimize_scaffold_shift_button!
            text_button("Optimize starting position", ui_size)
                .on_press(LeftPanelMessage::OptimizeScaffoldShiftPressed),
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
                    nucl_text = nucl_text.style(theme::DISABLED_TEXT);
                }
                nucl_text
            },
            extra_jump(),
            section("Staples", ui_size),
            extra_jump(),
            // add_download_staples_button!
            column![
                text_button("Export Staples", ui_size).on_press(LeftPanelMessage::StaplesRequested),
                text_button("Export Origamis", ui_size)
                    .on_press(LeftPanelMessage::OrigamisRequested),
            ]
            .spacing(ui_size.button_spacing()),
        ];
        scrollable(content).into()
    }
}
