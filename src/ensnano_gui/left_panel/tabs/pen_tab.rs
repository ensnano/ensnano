use crate::ensnano_consts::{ICON_HONEYCOMB_GRID, ICON_SQUARE_GRID};
use crate::ensnano_design::grid::GridTypeDescr;
use crate::ensnano_gui::AppState;
use crate::ensnano_gui::left_panel::Message;
use crate::ensnano_gui::left_panel::tabs::GuiTab;
use crate::ensnano_iced::fonts::material_icons::{MaterialIcon, MaterialIconStyle, icon_to_char};
use crate::ensnano_iced::helpers::{extra_jump, icon_button, material_icon_button, section};
use crate::ensnano_iced::ui_size::UiSize;
use iced::widget::{checkbox, column, row, text};
use iced_aw::TabLabel;
use std::marker::PhantomData;

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

    fn content(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Self::Message> {
        let selected_path_id = app_state.get_selected_bezier_path();
        let path_txt = match selected_path_id {
            Some(p) => format!("{p:?}"),
            None => "None".to_owned(),
        };

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
