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
    AppState, CheckXoversParameter, DesactivatedSlider, Fog, HBoundDisplay, Message, UiSize,
};
use crate::helpers::*;
use ensnano_interactor::graphics::{
    Background3D, RenderingMode, ALL_BACKGROUND3D, ALL_RENDERING_MODE,
};
use iced::Element;
use iced_native::widget;
use iced_native::widget::helpers::*;

pub struct CameraTab {
    fog: FogParameters,
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

    //pub fn view<'a, S, R>(
    //    &'a mut self,
    //    ui_size: UiSize,
    //    app_state: &S,
    //) -> Element<'a, Message<S>, R>
    //where
    //    S: AppState,
    //    R: Renderer + iced_native::text::Renderer,
    //    R::Theme: widget::button::StyleSheet
    //        + widget::checkbox::StyleSheet
    //        + widget::container::StyleSheet
    //        + widget::pick_list::StyleSheet
    //        + widget::scrollable::StyleSheet
    //        + widget::text::StyleSheet
    //        + iced::overlay::menu::StyleSheet,
    //    <R::Theme as iced::overlay::menu::StyleSheet>::Style:
    //        From<<R::Theme as iced::overlay::menu::StyleSheet>::Style>,
    //{
    //    let mut content = widget::Column::new().spacing(5);
    //    let content = iced_native::column![
    //        section("Camera", ui_size),
    //        subsection("Visibility", ui_size),
    //        text_btn("Toggle Selected Visibility", ui_size.clone())
    //            .on_press(Message::ToggleVisibility(false)),
    //        text_btn("Toggle NonSelected Visibility", ui_size.clone())
    //            .on_press(Message::ToggleVisibility(true)),
    //        text_btn("Everything visible", ui_size.clone()).on_press(Message::AllVisible),
    //        self.fog.view(&ui_size),
    //        subsection("Visibility", ui_size),
    //        pick_list(
    //            [
    //                HBoundDisplay::No,
    //                HBoundDisplay::Stick,
    //                HBoundDisplay::Ellipsoid,
    //            ],
    //            Some(app_state.get_h_bounds_display()),
    //            Message::ShowHBonds,
    //        ),
    //        right_checkbox(
    //            app_state.show_stereographic_camera(),
    //            "Show stereographic camera",
    //            Message::ShowStereographicCamera,
    //            ui_size,
    //        ),
    //        right_checkbox(
    //            app_state.follow_stereographic_camera(),
    //            "Follow stereographic camera",
    //            Message::FollowStereographicCamera,
    //            ui_size,
    //        ),
    //        subsection("Highlight Xovers", ui_size),
    //        pick_list(
    //            CheckXoversParameter::ALL,
    //            Some(app_state.get_checked_xovers_parameters()),
    //            Message::CheckXoversParameter,
    //        ),
    //        subsection("Rendering", ui_size),
    //        text("Style"),
    //        pick_list(
    //            &ALL_RENDERING_MODE[..],
    //            Some(self.rendering_mode),
    //            Message::RenderingMode,
    //        ),
    //        text("Background"),
    //        pick_list(
    //            &ALL_BACKGROUND3D[..],
    //            Some(self.background3d),
    //            Message::Background3D,
    //        ),
    //        checkbox(
    //            app_state.expand_insertions(),
    //            "Expand insertions",
    //            Message::SetExpandInsertions,
    //        ),
    //    ]
    //    .spacing(5);

    //    scrollable(content).into()
    //}

    pub fn view<S: AppState>(&self, ui_size: UiSize, app_state: &S) -> Element<Message<S>> {
        let content = iced_native::column![
            section("Camera", ui_size),
            subsection("Visibility", ui_size),
            text_button("Toggle Selected Visibility", ui_size.clone())
                .on_press(Message::ToggleVisibility(false)),
            text_button("Toggle NonSelected Visibility", ui_size.clone())
                .on_press(Message::ToggleVisibility(true)),
            text_button("Everything visible", ui_size.clone()).on_press(Message::AllVisible),
            self.fog.view(ui_size),
            subsection("Visibility", ui_size),
            pick_list(
                vec![
                    HBoundDisplay::No,
                    HBoundDisplay::Stick,
                    HBoundDisplay::Ellipsoid,
                ],
                Some(app_state.get_h_bounds_display()),
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
        self.fog.visible = visible
    }

    pub fn fog_dark(&mut self, dark: bool) {
        self.fog.dark = dark
    }

    pub fn fog_reversed(&mut self, reversed: bool) {
        self.fog.reversed = reversed
    }

    pub fn fog_length(&mut self, length: f32) {
        self.fog.length = length
    }

    pub fn fog_radius(&mut self, radius: f32) {
        self.fog.radius = radius
    }

    pub fn fog_camera(&mut self, from_camera: bool) {
        self.fog.from_camera = from_camera;
    }

    pub fn get_fog_request(&self) -> Fog {
        self.fog.request()
    }
}

struct FogParameters {
    visible: bool,
    from_camera: bool,
    dark: bool,
    radius: f32,
    length: f32,
    reversed: bool,
}

impl FogParameters {
    fn view<S: AppState>(&self, ui_size: UiSize) -> Element<Message<S>> {
        let radius_text = if self.visible {
            text("Radius")
        } else {
            text("Radius").style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.6, 0.6, 0.6,
            )))
        };

        let gradient_text = if self.visible {
            text("Softness")
        } else {
            text("Softness").style(iced::theme::Text::Color(iced::Color::from_rgb(
                0.6, 0.6, 0.6,
            )))
        };

        let length_slider = if self.visible {
            widget::Slider::new(0f32..=100f32, self.length, Message::FogLength)
        } else {
            widget::Slider::new(0f32..=100f32, self.length, |_| Message::Nothing)
                .style(DesactivatedSlider)
        };

        let softness_slider = if self.visible {
            widget::Slider::new(0f32..=100f32, self.radius, Message::FogRadius)
        } else {
            widget::Slider::new(0f32..=100f32, self.radius, |_| Message::Nothing)
                .style(DesactivatedSlider)
        };

        iced_native::column![
            subsection("Fog", ui_size),
            pick_list(
                &ALL_FOG_CHOICE[..],
                Some(FogChoice::from_param(
                    self.visible,
                    self.from_camera,
                    self.dark,
                    self.reversed,
                )),
                Message::FogChoice,
            ),
            iced_native::row![radius_text, length_slider,].spacing(5),
            iced_native::row![gradient_text, softness_slider,].spacing(5),
        ]
        .into()
    }

    fn request(&self) -> Fog {
        Fog {
            radius: self.radius,
            fog_kind: FogChoice::from_param(
                self.visible,
                self.from_camera,
                self.dark,
                self.reversed,
            )
            .fog_kind(),
            length: self.length,
            from_camera: self.from_camera,
            alt_fog_center: None,
        }
    }
}

impl Default for FogParameters {
    fn default() -> Self {
        Self {
            visible: false,
            dark: false,
            length: 10.,
            radius: 10.,
            from_camera: true,
            reversed: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum FogChoice {
    None,
    FromCamera,
    FromPivot,
    DarkFromCamera,
    DarkFromPivot,
    ReversedFromPivot,
}

impl Default for FogChoice {
    fn default() -> Self {
        Self::None
    }
}

const ALL_FOG_CHOICE: &'static [FogChoice] = &[
    FogChoice::None,
    FogChoice::FromCamera,
    FogChoice::FromPivot,
    FogChoice::DarkFromCamera,
    FogChoice::DarkFromPivot,
    FogChoice::ReversedFromPivot,
];

impl std::fmt::Display for FogChoice {
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

impl FogChoice {
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
