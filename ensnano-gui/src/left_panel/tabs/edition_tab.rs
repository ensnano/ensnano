use crate::{
    AppState,
    left_panel::{
        HelixRoll, Message, color_to_u32,
        discrete_value::{FactoryId, RequestFactory, ValueId},
        tabs::GuiTab,
    },
};
use ensnano_design::elements::DesignElementKey;
use ensnano_iced::{
    color_picker::{ColorPicker, ColorPickerMessage},
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{right_checkbox, section, start_stop_button, subsection, text_button},
    ui_size::UiSize,
};
use ensnano_interactor::{RollRequest, selection::extract_strands_from_selection};
use iced::{
    Command,
    widget::{column, row, scrollable},
};
use iced_aw::TabLabel;
use std::marker::PhantomData;

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
            _state_type: PhantomData,
        }
    }

    fn get_roll_target_helices(&self, selection: &[DesignElementKey]) -> Vec<usize> {
        let mut ret = vec![];
        for s in selection {
            if let DesignElementKey::Helix(h) = s {
                ret.push(*h);
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

    pub fn get_roll_request(&self, selection: &[DesignElementKey]) -> Option<RollRequest> {
        let roll_target_helices = self.get_roll_target_helices(selection);
        (!roll_target_helices.is_empty()).then(|| RollRequest {
            target_helices: Some(roll_target_helices.clone()),
        })
    }

    pub fn current_strand_color(&self) -> u32 {
        let color = self.color_picker.current_color();
        color_to_u32(color)
    }

    pub fn update_color_picker(&mut self, message: ColorPickerMessage) {
        self.color_picker.update(message);
    }
}

impl<State: AppState> GuiTab<State> for EditionTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Edit)))
    }

    fn update(&mut self, _app_state: &mut State) -> Command<Self::Message> {
        Command::none()
    }

    fn content(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Self::Message> {
        let roll_target_helices =
            self.get_roll_target_helices(&app_state.get_selection_as_design_element());
        let sim_state = &app_state.get_simulation_state();
        let autoroll_is_active = sim_state.is_rolling() || !roll_target_helices.is_empty();
        let selection_contains_strand =
            !extract_strands_from_selection(app_state.get_selection()).is_empty();
        let suggestion_parameters = *app_state.get_suggestion_parameters();
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
                    .view(!roll_target_helices.is_empty(), ui_size.intermediate_text())
            ),
            // add_autoroll_button!
            start_stop_button(
                "Autoroll selected helices",
                ui_size,
                autoroll_is_active.then_some(Message::RollTargeted),
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
                    Message::NewSuggestionParameters(suggestion_parameters.with_include_scaffold(b))
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
