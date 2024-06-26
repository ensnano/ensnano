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
use std::marker::PhantomData;

use super::tabs::GuiTab;
use super::{AppState, GridTypeDescr, Message, UiSize, ICON_HONEYCOMB_GRID, ICON_SQUARE_GRID};
use ensnano_iced::{
    fonts::{icon_to_char, MaterialIcon, MaterialIconStyle},
    helpers::*,
    iced::Element,
    iced_aw::TabLabel,
};

const NEW_BEZIER_PLANE_ICON: MaterialIcon = MaterialIcon::HistoryEdu;
const EDIT_BEZIER_PATH_ICON: MaterialIcon = MaterialIcon::LinearScale;

#[derive(Default)]
pub struct PenTab<State: AppState> {
    _state_type: PhantomData<State>,
}

impl<State: AppState> GuiTab<State> for PenTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Draw)))
    }

    fn content(
        &self,
        ui_size: UiSize,
        app_state: &State,
    ) -> Element<Self::Message, ensnano_iced::Theme, crate::Renderer> {
        let selected_path_id = app_state.get_selected_bezier_path();
        let path_txt = selected_path_id
            .map(|p| format!("{:?}", p))
            .unwrap_or_else(|| "None".to_string());

        let content = self::column![
            section("Bezier Planes", ui_size),
            self::column![
                material_icon_button(MaterialIcon::FileOpen, MaterialIconStyle::Light, ui_size)
                    .on_press(Message::LoadSvgFile),
                // add_buttons!
                row![
                    material_icon_button(NEW_BEZIER_PLANE_ICON, MaterialIconStyle::Light, ui_size)
                        .on_press(Message::NewBezierPlane),
                    material_icon_button(EDIT_BEZIER_PATH_ICON, MaterialIconStyle::Light, ui_size)
                        .on_press(Message::StartBezierPath),
                ]
                .spacing(ui_size.button_spacing()),
            ]
            .spacing(ui_size.button_spacing()),
            // add_grid_buttons!
            if let Some(path_id) = app_state.get_selected_bezier_path() {
                row![
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
                row![] // Yes, an empty row…
            },
            text(format!("Selected path {path_txt}")),
            if let Some(b) =
                selected_path_id.and_then(|p_id| app_state.get_reader().is_bezier_path_cyclic(p_id))
            {
                row![checkbox("Cyclic", b).on_toggle(move |cyclic| {
                    Message::MakeBezierPathCyclic {
                        path_id: selected_path_id.unwrap(),
                        cyclic,
                    }
                })]
            } else {
                row![] // This is trickery to always return the same object.
            },
            extra_jump(),
            checkbox("Show bezier paths", app_state.get_show_bezier_paths())
                .on_toggle(Message::SetShowBezierPaths,),
        ]
        .spacing(5);
        content.into()
    }
}
