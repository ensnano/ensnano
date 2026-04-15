use crate::{
    fonts::material_icons::{MaterialIcon, MaterialIconStyle, icon_to_char},
    helpers::{extra_jump, icon_button, material_icon_button, section},
    left_panel::{LeftPanelMessage, tabs::GuiTab},
};
use ensnano_design::{grid::GridTypeDescr, interaction_modes::ActionMode};
use ensnano_state::app_state::AppState;
use ensnano_utils::{
    consts::{ICON_HONEYCOMB_GRID, ICON_ROTATED_HONEYCOMB_GRID, ICON_SQUARE_GRID},
    ui_size::UiSize,
};
use iced::widget::{checkbox, column, row, text, tooltip};
use iced_aw::TabLabel;

const NEW_BEZIER_PLANE_ICON: MaterialIcon = MaterialIcon::HistoryEdu;
const EDIT_BEZIER_PATH_ICON: MaterialIcon = MaterialIcon::LinearScale;

#[derive(Default)]
pub struct PenTab;

impl GuiTab for PenTab {
    type Message = LeftPanelMessage;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Draw)))
    }

    fn content(&self, ui_size: UiSize, app_state: &AppState) -> iced::Element<'_, Self::Message> {
        let selected_path_id = app_state.get_selected_bezier_path();
        let path_txt = match selected_path_id {
            Some(p) => format!("{p:?}"),
            None => "None".to_owned(),
        };

        let content = column![
            section("Bezier Planes", ui_size),
            column![
                // add_buttons!
                row![
                    tooltip(
                        material_icon_button(
                            MaterialIcon::FileOpen,
                            MaterialIconStyle::Light,
                            ui_size
                        )
                        .on_press(LeftPanelMessage::LoadSvgFile),
                        "Open bezier file",
                        tooltip::Position::FollowCursor,
                    )
                    .style(iced::theme::Container::Box),
                    tooltip(
                        material_icon_button(
                            NEW_BEZIER_PLANE_ICON,
                            MaterialIconStyle::Light,
                            ui_size
                        )
                        .on_press(LeftPanelMessage::NewBezierPlane),
                        "Create a new bezier plane",
                        tooltip::Position::FollowCursor,
                    )
                    .style(iced::theme::Container::Box),
                    tooltip(
                        material_icon_button(
                            EDIT_BEZIER_PATH_ICON,
                            MaterialIconStyle::Light,
                            ui_size
                        )
                        .on_press(LeftPanelMessage::StartBezierPath)
                        .style(
                            if app_state.get_action_mode() == ActionMode::EditBezierPath {
                                iced::theme::Button::Positive
                            } else {
                                iced::theme::Button::Primary
                            }
                        )
                        .on_press(LeftPanelMessage::StartBezierPath),
                        "Add bezier vertex mode",
                        tooltip::Position::FollowCursor,
                    )
                    .style(iced::theme::Container::Box),
                ]
                .spacing(ui_size.button_spacing()),
            ]
            .spacing(ui_size.button_spacing()),
            // add_grid_buttons!
            if let Some(path_id) = app_state.get_selected_bezier_path() {
                row![
                    tooltip(
                        icon_button(ICON_SQUARE_GRID, ui_size).on_press(
                            LeftPanelMessage::TurnPathIntoGrid {
                                path_id,
                                grid_type: GridTypeDescr::Square { twist: None },
                            }
                        ),
                        "Attach square grid",
                        tooltip::Position::FollowCursor,
                    )
                    .style(iced::theme::Container::Box),
                    tooltip(
                        icon_button(ICON_HONEYCOMB_GRID, ui_size).on_press(
                            LeftPanelMessage::TurnPathIntoGrid {
                                path_id,
                                grid_type: GridTypeDescr::Honeycomb { twist: None },
                            }
                        ),
                        "Attach honeycomb grid",
                        tooltip::Position::FollowCursor,
                    )
                    .style(iced::theme::Container::Box),
                    tooltip(
                        icon_button(ICON_ROTATED_HONEYCOMB_GRID, ui_size).on_press(
                            LeftPanelMessage::TurnPathIntoGrid {
                                path_id,
                                grid_type: GridTypeDescr::RotatedHoneycomb { twist: None },
                            }
                        ),
                        "Attach rotated honeycomb grid",
                        tooltip::Position::FollowCursor,
                    )
                    .style(iced::theme::Container::Box),
                ]
                .spacing(ui_size.button_spacing())
            } else {
                row![
                    icon_button(ICON_SQUARE_GRID, ui_size),
                    icon_button(ICON_HONEYCOMB_GRID, ui_size),
                    icon_button(ICON_ROTATED_HONEYCOMB_GRID, ui_size),
                ] // same buttons inactivated
                .spacing(ui_size.button_spacing())
            },
            text(format!("Selected path: {path_txt}")),
            if let Some(b) =
                selected_path_id.and_then(|p_id| app_state.get_reader().is_bezier_path_cyclic(p_id))
            {
                row![
                    tooltip(
                        checkbox("Cyclic", b).on_toggle(move |cyclic| {
                            LeftPanelMessage::MakeBezierPathCyclic {
                                path_id: selected_path_id.unwrap(),
                                cyclic,
                            }
                        }),
                        "Make path cyclic",
                        tooltip::Position::FollowCursor,
                    )
                    .style(iced::theme::Container::Box),
                ]
            } else {
                row![checkbox("Cyclic", false)] // This is trickery to always return the same object.
            },
            extra_jump(),
            tooltip(
                checkbox(
                    "Show bezier paths and planes",
                    app_state.get_show_bezier_paths()
                )
                .on_toggle(LeftPanelMessage::SetShowBezierPaths,),
                "Show/Hide Bezier paths and planes",
                tooltip::Position::FollowCursor,
            )
            .style(iced::theme::Container::Box),
        ]
        .spacing(ui_size.button_spacing());
        content.into()
    }
}
