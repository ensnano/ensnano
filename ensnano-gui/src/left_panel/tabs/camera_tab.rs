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
    AppState, CheckXoversParameter, DesactivatedSlider, FogParameters, HBondDisplay, Message,
    UiSize,
};
use crate::helpers::*;
use ensnano_interactor::graphics::{
    Background3D, RenderingMode, ALL_BACKGROUND3D, ALL_RENDERING_MODE,
};
use iced::{theme, Element};
use iced_native::widget::helpers::*;

pub struct CameraTab {
    fog: FogGuiParameters,
    pub background3d: Background3D,
    pub rendering_mode: RenderingMode,
}

impl CameraTab {
    pub fn new() -> Self {
        Self {
            fog: Default::default(),
            background3d: Default::default(),
            rendering_mode: Default::default(),
        }
    }

    pub fn view<S: AppState>(&self, ui_size: UiSize, app_state: &S) -> Element<Message<S>> {
        let content = iced_native::column![
            section("Camera", ui_size),
            subsection("Visibility", ui_size),
            iced_native::column![
                text_button("Toggle Selected Visibility", ui_size)
                    .on_press(Message::ToggleVisibility(false)),
                text_button("Toggle NonSelected Visibility", ui_size)
                    .on_press(Message::ToggleVisibility(true)),
                text_button("Everything visible", ui_size).on_press(Message::AllVisible),
            ]
            .spacing(ui_size.button_pad()),
            self.fog.view(ui_size),
            subsection("Visibility", ui_size),
            pick_list(
                vec![
                    HBondDisplay::No,
                    HBondDisplay::Stick,
                    HBondDisplay::Ellipsoid,
                ],
                Some(app_state.get_h_bonds_display()),
                Message::ShowHBonds,
            ),
            right_checkbox(
                app_state.show_stereographic_camera(),
                "Show stereographic camera",
                Message::ShowStereographicCamera,
                ui_size,
            ),
            right_checkbox(
                app_state.follow_stereographic_camera(),
                "Follow stereographic camera",
                Message::FollowStereographicCamera,
                ui_size,
            ),
            subsection("Highlight Xovers", ui_size),
            pick_list(
                CheckXoversParameter::ALL,
                Some(app_state.get_checked_xovers_parameters()),
                Message::CheckXoversParameter,
            ),
            subsection("Rendering", ui_size),
            text("Style"),
            pick_list(
                &ALL_RENDERING_MODE[..],
                Some(self.rendering_mode),
                Message::RenderingMode,
            ),
            text("Background"),
            pick_list(
                &ALL_BACKGROUND3D[..],
                Some(self.background3d),
                Message::Background3D,
            ),
            checkbox(
                "Expand insertions",
                app_state.expand_insertions(),
                Message::SetExpandInsertions,
            ),
        ]
        .spacing(5);

        scrollable(content).into()
    }

    pub fn fog_visible(&mut self, visible: bool) {
        self.fog.is_activated = visible
    }

    pub fn fog_dark(&mut self, dark: bool) {
        self.fog.dark = dark
    }

    pub fn fog_reversed(&mut self, reversed: bool) {
        self.fog.is_reversed = reversed
    }

    pub fn fog_length(&mut self, length: f32) {
        self.fog.length = length
    }

    pub fn fog_radius(&mut self, radius: f32) {
        self.fog.softness = radius
    }

    pub fn fog_camera(&mut self, from_camera: bool) {
        self.fog.from_camera = from_camera;
    }

    pub fn get_fog_request(&self) -> FogParameters {
        self.fog.request()
    }
}

/// Parameters for the Distance Fog in the 3D view.
struct FogGuiParameters {
    // Is the Distance Fog activated or not.
    is_activated: bool,
    // Compute Distance Fog from the camera or pivot position.
    from_camera: bool,
    // Turn object into dark instead of disapearing.
    dark: bool,
    // Deepness of the Distance Fog.
    length: f32,
    // Softness of the Distance Fog cutoff.
    softness: f32,
    // Reverse the effect.
    is_reversed: bool,
}

impl FogGuiParameters {
    fn view<S: AppState>(&self, ui_size: UiSize) -> Element<Message<S>> {
        let deactivated_text_color = theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6));
        let radius_text = if self.is_activated {
            text("Radius")
        } else {
            text("Radius").style(deactivated_text_color)
        };

        let gradient_text = if self.is_activated {
            text("Softness")
        } else {
            text("Softness").style(deactivated_text_color)
        };

        let length_slider = if self.is_activated {
            slider(0f32..=100f32, self.length, Message::FogLength)
        } else {
            slider(0f32..=100f32, self.length, |_| Message::Nothing).style(DesactivatedSlider)
        };

        let softness_slider = if self.is_activated {
            slider(0f32..=100f32, self.softness, Message::FogRadius)
        } else {
            slider(0f32..=100f32, self.softness, |_| Message::Nothing).style(DesactivatedSlider)
        };

        iced_native::column![
            subsection("Distance Fog", ui_size),
            pick_list(
                &ALL_FOG_CHOICES[..],
                Some(FogChoices::from_param(
                    self.is_activated,
                    self.from_camera,
                    self.dark,
                    self.is_reversed,
                )),
                Message::FogChoice,
            ),
            iced_native::row![radius_text, length_slider,].spacing(5),
            iced_native::row![gradient_text, softness_slider,].spacing(5),
        ]
        .into()
    }

    /// Compute Distance Fog parameters from GUI values.
    fn request(&self) -> FogParameters {
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

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum FogChoices {
    None,
    FromCamera,
    FromPivot,
    DarkFromCamera,
    DarkFromPivot,
    ReversedFromPivot,
}

impl Default for FogChoices {
    fn default() -> Self {
        Self::None
    }
}

const ALL_FOG_CHOICES: &'static [FogChoices] = &[
    FogChoices::None,
    FogChoices::FromCamera,
    FogChoices::FromPivot,
    FogChoices::DarkFromCamera,
    FogChoices::DarkFromPivot,
    FogChoices::ReversedFromPivot,
];

impl std::fmt::Display for FogChoices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::None => "None",
            Self::FromCamera => "From Camera",
            Self::FromPivot => "From Pivot",
            Self::DarkFromCamera => "Dark from Camera",
            Self::DarkFromPivot => "Dark from Pivot",
            Self::ReversedFromPivot => "Reversed from Pivot",
        };
        write!(f, "{}", ret)
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

    pub fn to_param(&self) -> (bool, bool, bool, bool) {
        (
            self.is_visible(),
            self.is_from_camera(),
            self.is_dark(),
            self.is_reversed(),
        )
    }

    fn visible(self, visible: bool) -> Self {
        if visible {
            if let Self::None = self {
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
        use ensnano_interactor::graphics::fog_kind;
        match self {
            Self::None => fog_kind::NO_FOG,
            Self::FromCamera | Self::FromPivot => fog_kind::TRANSPARENT_FOG,
            Self::DarkFromPivot | Self::DarkFromCamera => fog_kind::DARK_FOG,
            Self::ReversedFromPivot => fog_kind::REVERSED_FOG,
        }
    }
}
