use crate::{
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{extra_jump, jump_by, right_checkbox, section, subsection},
    left_panel::{
        LeftPanelMessage, ScrollSensitivity, discrete_value::RequestFactory, tabs::GuiTab,
    },
};
use ensnano_design::{ensnano_version, parameters::NAMED_DNA_PARAMETERS};
use ensnano_state::{
    app_state::AppState,
    gui::messages::{FactoryId, ValueId},
};
use ensnano_utils::ui_size::{ALL_UI_SIZES, UiSize};
use iced::widget::{column, pick_list, scrollable, text};
use iced_aw::TabLabel;

pub struct ParametersTab {
    scroll_sensitivity_factory: RequestFactory<ScrollSensitivity>,
}

impl ParametersTab {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            scroll_sensitivity_factory: RequestFactory::new(
                FactoryId::Scroll,
                ScrollSensitivity {
                    initial_value: app_state.get_scroll_sensitivity(),
                },
            ),
        }
    }

<<<<<<< HEAD
    pub fn view<'a, S: AppState>(
        &'a mut self,
        ui_size: UiSize,
        app_state: &S,
    ) -> Element<'a, Message<S>> {
        let mut ret = Column::new();
        section!(ret, ui_size, "Parameters");
        extra_jump!(ret);
        subsection!(ret, ui_size, "Font size");
        ret = ret.push(PickList::new(
            &mut self.size_pick_list,
            &super::super::super::ALL_UI_SIZE[..],
            Some(ui_size.clone()),
            Message::UiSizePicked,
        ));

        extra_jump!(ret);
        subsection!(ret, ui_size, "Scrolling");
        for view in self
            .scroll_sensitivity_factory
            .view(true, ui_size.main_text())
            .into_iter()
        {
            ret = ret.push(view);
        }

        ret = ret.push(right_checkbox(
            app_state.get_invert_y_scroll(),
            "Inverse direction",
            Message::InvertScroll,
            ui_size.clone(),
        ));

        extra_jump!(10, ret);
        section!(ret, ui_size, "DNA/RNA model");
        ret = ret.push(PickList::new(
            &mut self.dna_parameters_picklist,
            &ensnano_design::NAMED_DNA_PARAMETERS[..],
            Some(app_state.get_dna_parameters().name().clone()),
            Message::NewDnaParameters,
        ));
        for line in app_state.get_dna_parameters().formated_string().lines() {
            ret = ret.push(Text::new(line));
        }
        ret = ret.push(iced::Space::with_height(Length::Units(10)));
        ret = ret.push(Text::new("About").size(ui_size.head_text()));
        ret = ret.push(Text::new(format!(
            "Version {}",
            ensnano_design::ensnano_version()
        )));

        subsection!(ret, ui_size, "Development:");
        ret = ret.push(Text::new("Nicolas Levy"));
        ret = ret.push(Text::new("Nicolas Schabanel"));
        ret = ret.push(Text::new("Joris Picot"));
        ret = ret.push(Text::new("Pierre Marcus"));
        ret = ret.push(Text::new("Octave Hazard"));
        ret = ret.push(Text::new("Daria Pchelina"));
        extra_jump!(ret);
        subsection!(ret, ui_size, "Conception:");
        ret = ret.push(Text::new("Nicolas Schabanel"));
        ret = ret.push(Text::new("Nicolas Levy"));
        extra_jump!(ret);
        subsection!(ret, ui_size, "License:");
        ret = ret.push(Text::new("GPLv3"));

        Scrollable::new(&mut self.scroll).push(ret).into()
    }

=======
>>>>>>> dev_private
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

impl GuiTab for ParametersTab {
    type Message = LeftPanelMessage;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Settings)))
    }

    fn content(&self, ui_size: UiSize, app_state: &AppState) -> iced::Element<'_, Self::Message> {
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
                true
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
