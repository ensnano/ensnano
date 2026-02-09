use crate::{
    fog::FogGuiParameters,
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{extra_jump, right_checkbox, section, subsection, text_button},
    left_panel::{LeftPanelMessage, tabs::GuiTab},
};
use ensnano_state::app_state::AppState;
use ensnano_utils::{
    app_state_parameters::{AppStateParameters, check_xovers_parameter::CheckXoversParameter},
    graphics::{
        ALL_BACKGROUND3D, ALL_RENDERING_MODE, Background3D, FogParameters, HBondDisplay,
        RenderingMode,
    },
    ui_size::UiSize,
};
use iced::{
    Alignment, Length,
    widget::{checkbox, column, pick_list, row, scrollable},
};
use iced_aw::TabLabel;

pub struct CameraTab {
    fog: FogGuiParameters,
    pub background3d: Background3D,
    pub rendering_mode: RenderingMode,
}

impl CameraTab {
    pub fn new(parameters: &AppStateParameters) -> Self {
        Self {
            fog: Default::default(),
            background3d: parameters.background3d,
            rendering_mode: parameters.rendering_mode,
        }
    }

    pub fn fog_visible(&mut self, visible: bool) {
        self.fog.is_activated = visible;
    }

    pub fn fog_dark(&mut self, dark: bool) {
        self.fog.dark = dark;
    }

    pub fn fog_reversed(&mut self, reversed: bool) {
        self.fog.is_reversed = reversed;
    }

    pub fn fog_length(&mut self, length: f32) {
        self.fog.length = length;
    }

    pub fn fog_radius(&mut self, radius: f32) {
        self.fog.softness = radius;
    }

    pub fn fog_camera(&mut self, from_camera: bool) {
        self.fog.from_camera = from_camera;
    }

    pub fn get_fog_request(&self) -> FogParameters {
        self.fog.request()
    }
}

impl GuiTab for CameraTab {
    type Message = LeftPanelMessage;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Videocam)))
    }

    fn content(
        &self,
        ui_size: UiSize,
        app_state: &AppState,
    ) -> iced::Element<'_, LeftPanelMessage> {
        let content = column![
            section("Camera", ui_size),
            subsection("Toggle visibility", ui_size),
            row![
                text_button("Selected", ui_size)
                    .on_press(LeftPanelMessage::ToggleVisibility(false)),
                text_button("Non-selected", ui_size)
                    .on_press(LeftPanelMessage::ToggleVisibility(true)),
                text_button("All", ui_size).on_press(LeftPanelMessage::AllVisible),
            ]
            .width(Length::Fill)
            .spacing(ui_size.button_spacing()),
            self.fog.view(ui_size),
            extra_jump(),
            row![
                subsection("H-bond", ui_size),
                pick_list(
                    vec![
                        HBondDisplay::No,
                        HBondDisplay::Stick,
                        HBondDisplay::Ellipsoid,
                    ],
                    Some(app_state.get_h_bonds_display()),
                    LeftPanelMessage::ShowHBonds,
                ),
            ]
            .align_items(Alignment::Center)
            .spacing(5),
            right_checkbox(
                app_state.show_stereographic_camera(),
                "Show stereographic camera",
                LeftPanelMessage::ShowStereographicCamera,
                ui_size,
            ),
            right_checkbox(
                app_state.follow_stereographic_camera(),
                "Follow stereographic camera",
                LeftPanelMessage::FollowStereographicCamera,
                ui_size,
            ),
            extra_jump(),
            row![
                subsection("Highlight Xovers", ui_size),
                pick_list(
                    CheckXoversParameter::ALL,
                    Some(app_state.get_checked_xovers_parameters()),
                    LeftPanelMessage::CheckXoversParameter,
                ),
            ]
            .align_items(Alignment::Center)
            .spacing(5),
            extra_jump(),
            subsection("Rendering", ui_size),
            row![
                row![
                    "Style",
                    pick_list(
                        ALL_RENDERING_MODE,
                        Some(self.rendering_mode),
                        LeftPanelMessage::RenderingMode,
                    ),
                ]
                .align_items(Alignment::Center)
                .spacing(5)
                .width(Length::FillPortion(1)),
                row![
                    "Background",
                    pick_list(
                        ALL_BACKGROUND3D,
                        Some(self.background3d),
                        LeftPanelMessage::Background3D,
                    ),
                ]
                .align_items(Alignment::Center)
                .spacing(5)
                .width(Length::FillPortion(1)),
            ],
            checkbox("Expand insertions", app_state.expand_insertions())
                .on_toggle(LeftPanelMessage::SetExpandInsertions),
        ]
        .spacing(5);

        scrollable(content).into()
    }
}
