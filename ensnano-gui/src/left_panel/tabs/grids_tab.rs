use crate::{
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{extra_jump, icon_button, section, subsection, text_button},
    left_panel::{Hyperboloid_, LeftPanelMessage, discrete_value::RequestFactory, tabs::GuiTab},
};
use ensnano_design::grid::GridTypeDescr;
use ensnano_state::{
    app_state::AppState,
    design::operation::HyperboloidRequest,
    gui::messages::{FactoryId, ValueId},
};
use ensnano_utils::{
    consts::{ICON_HONEYCOMB_GRID, ICON_NANOTUBE, ICON_SQUARE_GRID},
    ui_size::UiSize,
};
use iced::{
    Length,
    widget::{Column, column, row, scrollable, text, tooltip},
};
use iced_aw::TabLabel;

pub struct GridTab {
    hyperboloid_factory: RequestFactory<Hyperboloid_>,
}

impl GridTab {
    pub fn new() -> Self {
        Self {
            hyperboloid_factory: RequestFactory::new(FactoryId::Hyperboloid, Hyperboloid_),
        }
    }

    pub fn new_hyperboloid(&mut self, requests: &mut Option<HyperboloidRequest>) {
        self.hyperboloid_factory = RequestFactory::new(FactoryId::Hyperboloid, Hyperboloid_);
        self.hyperboloid_factory.make_request(requests);
    }

    pub fn update_hyperboloid_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<HyperboloidRequest>,
    ) {
        self.hyperboloid_factory
            .update_request(value_id, value, request);
    }
}

impl GuiTab for GridTab {
    type Message = LeftPanelMessage;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::GridOn)))
    }

    fn content(&self, ui_size: UiSize, app_state: &AppState) -> iced::Element<'_, Self::Message> {
        let content = column![
            section("Grids", ui_size),
            subsection("New Grid", ui_size),
            // add_grid_buttons!
            row![
                icon_button(ICON_SQUARE_GRID, ui_size).on_press(LeftPanelMessage::NewGrid(
                    GridTypeDescr::Square { twist: None }
                )),
                icon_button(ICON_HONEYCOMB_GRID, ui_size).on_press(LeftPanelMessage::NewGrid(
                    GridTypeDescr::Honeycomb { twist: None }
                )),
            ]
            .spacing(ui_size.button_spacing()),
            extra_jump(),
            subsection("New nanotube", ui_size),
            // add_start_cancel_hyperboloid_button!
            if app_state.is_building_hyperboloid() {
                row![
                    text_button("Cancel", ui_size)
                        .on_press(LeftPanelMessage::CancelHyperboloid)
                        .style(iced::theme::Button::Destructive),
                    text_button("Finish", ui_size)
                        .on_press(LeftPanelMessage::FinalizeHyperboloid)
                        .style(iced::theme::Button::Positive),
                ]
                .spacing(ui_size.button_spacing())
            } else {
                row![
                        icon_button(ICON_NANOTUBE, ui_size)
                            .on_press(LeftPanelMessage::NewHyperboloid),
                    ]
                    .spacing(ui_size.button_spacing())
            },
            // add hyperboloid sliders!
            Column::with_children(
                self.hyperboloid_factory
                    .view(app_state.is_building_hyperboloid(), ui_size.main_text()),
            ),
            extra_jump(),
            subsection("Guess grid", ui_size),
            // add_guess_grid_button!
            tooltip(
                text_button("From Selection", ui_size).on_press_maybe(
                    app_state
                        .can_make_grid()
                        .then_some(LeftPanelMessage::MakeGrids)
                ),
                text("Select ≥4 unattached helices").size(ui_size.main_text()),
                tooltip::Position::FollowCursor,
            )
            .style(iced::theme::Container::Box),
        ]
        .spacing(5);
        scrollable(content).width(Length::Fill).into()
    }
}
