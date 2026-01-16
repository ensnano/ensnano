use crate::{
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{extra_jump, jump_by, right_checkbox, section, subsection},
    left_panel::{
        LeftPanelMessage, ScrollSensitivity, discrete_value::RequestFactory, tabs::GuiTab,
    },
    messages::{FactoryId, ValueId},
    state::GuiAppState,
};
use ensnano_design::{ensnano_version, parameters::NAMED_DNA_PARAMETERS};
use ensnano_utils::ui_size::{ALL_UI_SIZES, UiSize};
use iced::widget::{column, pick_list, scrollable, text};
use iced_aw::TabLabel;
use std::marker::PhantomData;

pub struct ParametersTab<State: GuiAppState> {
    scroll_sensitivity_factory: RequestFactory<ScrollSensitivity>,
    _invert_y_scroll: bool,
    _state_type: PhantomData<State>,
}

impl<State: GuiAppState> ParametersTab<State> {
    pub fn new<S: GuiAppState>(app_state: &S) -> Self {
        Self {
            scroll_sensitivity_factory: RequestFactory::new(
                FactoryId::Scroll,
                ScrollSensitivity {
                    initial_value: app_state.get_scroll_sensitivity(),
                },
            ),
            _invert_y_scroll: false,
            _state_type: PhantomData,
        }
    }

    pub fn update_scroll_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<f32>,
    ) {
        self.scroll_sensitivity_factory
            .update_request(value_id, value, request);
    }
}

impl<State: GuiAppState> GuiTab<State> for ParametersTab<State> {
    type Message = LeftPanelMessage<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Settings)))
    }

    fn content(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Self::Message> {
        let dna_params = &app_state.get_dna_parameters();

        let content = column![
            section("Parameters", ui_size),
            extra_jump(),
            subsection("Font size", ui_size),
            pick_list(
                &ALL_UI_SIZES[..],
                Some(ui_size),
                LeftPanelMessage::UiSizePicked,
            ),
            extra_jump(),
            subsection("Scrolling", ui_size),
            column(
                self.scroll_sensitivity_factory
                    .view(true, ui_size.main_text())
            ),
            right_checkbox(
                app_state.get_invert_y_scroll(),
                "Inverse direction",
                LeftPanelMessage::InvertScroll,
                ui_size,
            ),
            jump_by(10),
            section("DNA/RNA model", ui_size),
            pick_list(
                &NAMED_DNA_PARAMETERS[..],
                Some(app_state.get_dna_parameters().name().clone()),
                LeftPanelMessage::NewDnaParameters,
            ),
            column![
                text(format!("  Radius: {:.3} nm", dna_params.helix_radius)),
                text(format!("  Radius: {:.3} nm", dna_params.helix_radius)),
                text(format!("  Rise: {:.3} nm", dna_params.rise)),
                text(format!("  Inclination {:.3} nm", dna_params.inclination)),
                text(format!("  Helicity: {:.2} bp", dna_params.bases_per_turn)),
                text(format!(
                    "  Axis: {:.1}°",
                    dna_params.groove_angle.to_degrees()
                )),
                text(format!(
                    "  Inter helix gap: {:.2} nm",
                    dna_params.inter_helix_gap
                )),
                text(format!(
                    " Expected xover length: {:.2} nm",
                    dna_params.dist_ac()
                )),
            ],
            jump_by(10),
            section("About", ui_size),
            text(format!("Version {}", ensnano_version())),
            subsection("Development:", ui_size),
            "Nicolas Levy",
            extra_jump(),
            subsection("Conception:", ui_size),
            "Nicolas Levy",
            "Nicolas Schabanel",
            extra_jump(),
            subsection("License:", ui_size),
            "GPLv3",
        ];
        scrollable(content).into()
    }
}
