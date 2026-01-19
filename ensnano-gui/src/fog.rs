use crate::{
    helpers::{extra_jump, subsection},
    messages::LeftPanelMessage,
    state::GuiAppState,
    theme,
};
use ensnano_utils::{
    graphics::{FogParameters, fog_kind},
    ui_size::UiSize,
};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FogChoices {
    #[default]
    None,
    FromCamera,
    FromPivot,
    DarkFromCamera,
    DarkFromPivot,
    ReversedFromPivot,
}

const ALL_FOG_CHOICES: &[FogChoices] = &[
    FogChoices::None,
    FogChoices::FromCamera,
    FogChoices::FromPivot,
    FogChoices::DarkFromCamera,
    FogChoices::DarkFromPivot,
    FogChoices::ReversedFromPivot,
];

impl std::fmt::Display for FogChoices {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ret = match self {
            Self::None => "None",
            Self::FromCamera => "From Camera",
            Self::FromPivot => "From Pivot",
            Self::DarkFromCamera => "Dark from Camera",
            Self::DarkFromPivot => "Dark from Pivot",
            Self::ReversedFromPivot => "Reversed from Pivot",
        };
        write!(f, "{ret}")
    }
}

impl FogChoices {
    fn from_param(visible: bool, from_camera: bool, dark: bool, reversed: bool) -> Self {
        Self::None
            .visible(visible)
            .dark(dark)
            .from_camera(from_camera)
            .reversed(reversed)
    }

    pub fn to_param(self) -> (bool, bool, bool, bool) {
        (
            self.is_visible(),
            self.is_from_camera(),
            self.is_dark(),
            self.is_reversed(),
        )
    }

    fn visible(self, visible: bool) -> Self {
        if visible {
            if self == Self::None {
                Self::FromPivot
            } else {
                self
            }
        } else {
            Self::None
        }
    }

    fn from_camera(self, from_camera: bool) -> Self {
        if from_camera {
            match self {
                Self::FromPivot => Self::FromCamera,
                Self::DarkFromPivot => Self::DarkFromCamera,
                _ => self,
            }
        } else {
            match self {
                Self::FromCamera => Self::FromPivot,
                Self::DarkFromCamera => Self::DarkFromPivot,
                _ => self,
            }
        }
    }

    fn reversed(self, reversed: bool) -> Self {
        match (self, reversed) {
            (Self::FromPivot, true) => Self::ReversedFromPivot,
            (Self::ReversedFromPivot, false) => Self::FromPivot,
            _ => self,
        }
    }

    fn dark(self, dark: bool) -> Self {
        if dark {
            match self {
                Self::FromCamera => Self::DarkFromCamera,
                Self::FromPivot => Self::DarkFromPivot,
                _ => self,
            }
        } else {
            match self {
                Self::DarkFromCamera => Self::FromCamera,
                Self::DarkFromPivot => Self::FromPivot,
                _ => self,
            }
        }
    }

    fn is_visible(&self) -> bool {
        !matches!(self, Self::None)
    }

    fn is_from_camera(&self) -> bool {
        matches!(self, Self::FromCamera | Self::DarkFromCamera)
    }

    fn is_dark(&self) -> bool {
        matches!(self, Self::DarkFromCamera | Self::DarkFromPivot)
    }

    fn is_reversed(&self) -> bool {
        matches!(self, Self::ReversedFromPivot)
    }

    fn fog_kind(&self) -> u32 {
        match self {
            Self::None => fog_kind::NO_FOG,
            Self::FromCamera | Self::FromPivot => fog_kind::TRANSPARENT_FOG,
            Self::DarkFromPivot | Self::DarkFromCamera => fog_kind::DARK_FOG,
            Self::ReversedFromPivot => fog_kind::REVERSED_FOG,
        }
    }
}
