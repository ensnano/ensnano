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
use super::{AppState, GridTypeDescr, Message, UiSize, ICON_HONEYCOMB_GRID, ICON_SQUARE_GRID};
use crate::helpers::*;
use crate::material_icons_light::LightIcon;
use iced::Element;
use iced_native::widget::helpers::*;

const NEW_BEZIER_PLANE_ICON: LightIcon = LightIcon::HistoryEdu;
const EDIT_BEZIER_PATH_ICON: LightIcon = LightIcon::LinearScale;

#[derive(Default)]
pub struct PenTab {}

impl PenTab {
    pub fn view<S>(&self, ui_size: UiSize, app_state: &S) -> Element<Message<S>>
    where
        S: AppState,
    {
        let selected_path_id = app_state.get_selected_bezier_path();
        let path_txt = selected_path_id
            .map(|p| format!("{:?}", p))
            .unwrap_or_else(|| "None".to_string());

        let content = iced_native::column![
            section("Bezier Planes", ui_size),
            light_icon_button(LightIcon::FileOpen, ui_size).on_press(Message::LoadSvgFile),
            // add_buttons!
            iced_native::row![
                light_icon_button(NEW_BEZIER_PLANE_ICON, ui_size).on_press(Message::NewBezierPlane),
                light_icon_button(EDIT_BEZIER_PATH_ICON, ui_size)
                    .on_press(Message::StartBezierPath),
            ],
            // add_grid_buttons!
            if let Some(path_id) = app_state.get_selected_bezier_path() {
                iced_native::row![
                    icon_button(ICON_SQUARE_GRID, ui_size).on_press(Message::TurnPathIntoGrid {
                        path_id,
                        grid_type: GridTypeDescr::Square { twist: None },
                    }),
                    icon_button(ICON_HONEYCOMB_GRID, ui_size).on_press(Message::TurnPathIntoGrid {
                        path_id,
                        grid_type: GridTypeDescr::Honeycomb { twist: None },
                    }),
                ]
                .spacing(5)
            } else {
                iced_native::row![] // Yes, an empty row…
            },
            text(format!("Selected path {path_txt}")),
            if let Some(b) =
                selected_path_id.and_then(|p_id| app_state.get_reader().is_bezier_path_cyclic(p_id))
            {
                iced_native::row![checkbox("Cyclic", b, move |cyclic| {
                    Message::MakeBezierPathCyclic {
                        path_id: selected_path_id.unwrap(),
                        cyclic,
                    }
                })]
            } else {
                iced_native::row![] // This is trickery to always return the same object.
            },
            extra_jump(),
            checkbox(
                "Show bezier paths",
                app_state.get_show_bezier_paths(),
                Message::SetShowBezierPaths,
            ),
        ]
        .spacing(5);
        content.into()
    }
}
