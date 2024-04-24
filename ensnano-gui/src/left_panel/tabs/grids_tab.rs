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

use super::{
    AppState, FactoryId, GridTypeDescr, HyperboloidRequest, Hyperboloid_, Message, RequestFactory,
    UiSize, ValueId, ICON_HONEYCOMB_GRID, ICON_NANOTUBE, ICON_SQUARE_GRID,
};
use crate::helpers::*;
use iced::{Element, Length};

pub struct GridTab {
    hyperboloid_factory: RequestFactory<Hyperboloid_>,
}

impl GridTab {
    pub fn new() -> Self {
        Self {
            hyperboloid_factory: RequestFactory::new(FactoryId::Hyperboloid, Hyperboloid_ {}),
        }
    }

    pub fn view<S>(&self, ui_size: UiSize, app_state: &S) -> Element<Message<S>>
    where
        S: AppState,
    {
        let content = self::column![
            section("Grids", ui_size),
            subsection("New Grid", ui_size),
            // add_grid_buttons!
            row![
                icon_button(ICON_SQUARE_GRID, ui_size)
                    .on_press(Message::NewGrid(GridTypeDescr::Square { twist: None })),
                icon_button(ICON_HONEYCOMB_GRID, ui_size)
                    .on_press(Message::NewGrid(GridTypeDescr::Honeycomb { twist: None })),
            ]
            .spacing(ui_size.button_pad()),
            extra_jump(),
            subsection("New nanotube", ui_size),
            // add_start_cancel_hyperboloid_button!
            if app_state.is_building_hyperboloid() {
                row![
                    text_button("Cancel", ui_size)
                        .on_press(Message::CancelHyperboloid)
                        .style(iced::theme::Button::Destructive),
                    text_button("Finish", ui_size)
                        .on_press(Message::FinalizeHyperboloid)
                        .style(iced::theme::Button::Positive),
                ]
                .spacing(ui_size.button_pad())
            } else {
                row![icon_button(ICON_NANOTUBE, ui_size).on_press(Message::NewHyperboloid),]
                    .spacing(ui_size.button_pad())
            },
            // add hyperboloid sliders!
            Column::with_children(
                self.hyperboloid_factory
                    .view(app_state.is_building_hyperboloid(), ui_size.main_text()),
            ),
            extra_jump(),
            subsection("Guess grid", ui_size),
            // add_guess_grid_button!
            if app_state.can_make_grid() {
                button(text("From Selection"))
                    .height(ui_size.button())
                    .on_press(Message::MakeGrids)
            } else {
                button(text("From Selection")).height(ui_size.button())
            },
            text("Select ≥4 unattached helices").size(ui_size.main_text()),
        ]
        .spacing(5);
        scrollable(content).width(Length::Fill).into()
    }

    pub fn new_hyperboloid(&mut self, requests: &mut Option<HyperboloidRequest>) {
        self.hyperboloid_factory = RequestFactory::new(FactoryId::Hyperboloid, Hyperboloid_ {});
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
