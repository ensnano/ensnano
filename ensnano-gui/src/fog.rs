use crate::{
    helpers::{extra_jump, subsection},
    theme,
};
use ensnano_state::gui::{
    messages::{ALL_FOG_CHOICES, FogChoices, LeftPanelMessage},
    state::GuiAppState,
};
use ensnano_utils::{graphics::FogParameters, ui_size::UiSize};
use iced::{
    Alignment,
    widget::{column, pick_list, row, slider, text},
};

/// Parameters for the Distance Fog in the 3D view.
pub struct FogGuiParameters {
    // Is the Distance Fog activated or not.
    pub is_activated: bool,
    // Compute Distance Fog from the camera or pivot position.
    pub from_camera: bool,
    // Turn object into dark instead of disappearing.
    pub dark: bool,
    // Deepness of the Distance Fog.
    pub length: f32,
    // Softness of the Distance Fog cutoff.
    pub softness: f32,
    // Reverse the effect.
    pub is_reversed: bool,
}

impl FogGuiParameters {
    pub fn view<State: GuiAppState>(
        &self,
        ui_size: UiSize,
    ) -> iced::Element<'_, LeftPanelMessage<State>> {
        let radius_text = if self.is_activated {
            text("Radius")
        } else {
            text("Radius").style(theme::DISABLED_TEXT)
        };

        let gradient_text = if self.is_activated {
            text("Softness")
        } else {
            text("Softness").style(theme::DISABLED_TEXT)
        };

        let length_slider = if self.is_activated {
            slider(0f32..=100f32, self.length, LeftPanelMessage::FogLength)
        } else {
            slider(0f32..=100f32, self.length, |_| LeftPanelMessage::Nothing)
                .style(theme::DeactivatedSlider)
        };

        let softness_slider = if self.is_activated {
            slider(0f32..=100f32, self.softness, LeftPanelMessage::FogRadius)
        } else {
            slider(0f32..=100f32, self.softness, |_| LeftPanelMessage::Nothing)
                .style(theme::DeactivatedSlider)
        };

        // Hand method to
        let label_width = 65.0f32;

        column![
            extra_jump(),
            row![
                subsection("Distance Fog", ui_size),
                pick_list(
                    ALL_FOG_CHOICES,
                    Some(FogChoices::from_param(
                        self.is_activated,
                        self.from_camera,
                        self.dark,
                        self.is_reversed,
                    )),
                    LeftPanelMessage::FogChoice,
                )
                .padding(ui_size.button_spacing()),
            ]
            .align_items(Alignment::Center)
            .spacing(5),
            row![radius_text.width(label_width), length_slider,].spacing(5),
            row![gradient_text.width(label_width), softness_slider,].spacing(5),
        ]
        .spacing(5)
        .into()
    }

    /// Compute Distance Fog parameters from GUI values.
    pub fn request(&self) -> FogParameters {
        FogParameters {
            softness: self.softness,
            fog_kind: FogChoices::from_param(
                self.is_activated,
                self.from_camera,
                self.dark,
                self.is_reversed,
            )
            .fog_kind(),
            length: self.length,
            from_camera: self.from_camera,
            alt_fog_center: None,
        }
    }
}

impl Default for FogGuiParameters {
    fn default() -> Self {
        Self {
            is_activated: false,
            dark: false,
            length: 10.,
            softness: 10.,
            from_camera: true,
            is_reversed: false,
        }
    }
}
