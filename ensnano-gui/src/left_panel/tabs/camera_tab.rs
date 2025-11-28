use crate::{
    AppState,
    left_panel::{Message, tabs::GuiTab},
};
use ensnano_iced::{
    fonts::material_icons::{MaterialIcon, icon_to_char},
    helpers::{extra_jump, right_checkbox, section, subsection, text_button},
    theme,
    ui_size::UiSize,
};
use ensnano_interactor::{
    app_state_parameters::{AppStateParameters, check_xovers_parameter::CheckXoversParameter},
    graphics::{
        ALL_BACKGROUND3D, ALL_RENDERING_MODE, Background3D, FogParameters, HBondDisplay,
        RenderingMode, fog_kind,
    },
};
use iced::{
    Alignment, Length,
    widget::{checkbox, column, pick_list, row, scrollable, slider, text},
};
use iced_aw::TabLabel;
use std::marker::PhantomData;

pub struct CameraTab<State: AppState> {
    fog: FogGuiParameters,
    pub background3d: Background3D,
    pub rendering_mode: RenderingMode,
    _state_type: PhantomData<State>,
}

impl<State: AppState> CameraTab<State> {
    pub fn new(parameters: &AppStateParameters) -> Self {
        Self {
            fog: Default::default(),
            background3d: parameters.background3d,
            rendering_mode: parameters.rendering_mode,
            _state_type: PhantomData,
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

impl<State: AppState> GuiTab<State> for CameraTab<State> {
    type Message = Message<State>;

    fn label(&self) -> TabLabel {
        TabLabel::Text(format!("{}", icon_to_char(MaterialIcon::Videocam)))
    }

    fn content(&self, ui_size: UiSize, app_state: &State) -> iced::Element<'_, Message<State>> {
        let content = self::column![
            section("Camera", ui_size),
            subsection("Toggle visibility", ui_size),
            row![
                text_button("Selected", ui_size).on_press(Message::ToggleVisibility(false)),
                text_button("Non-selected", ui_size).on_press(Message::ToggleVisibility(true)),
                text_button("All", ui_size).on_press(Message::AllVisible),
            ]
            .width(Length::Fill)
            .spacing(ui_size.button_spacing()),
            self.fog.view(ui_size),
            extra_jump(),
            row![
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
            ]
            .align_items(Alignment::Center)
            .spacing(5),
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
            extra_jump(),
            row![
                subsection("Highlight Xovers", ui_size),
                pick_list(
                    CheckXoversParameter::ALL,
                    Some(app_state.get_checked_xovers_parameters()),
                    Message::CheckXoversParameter,
                ),
            ]
            .align_items(Alignment::Center)
            .spacing(5),
            extra_jump(),
            subsection("Rendering", ui_size),
            row![
                row![
                    text("Style"),
                    pick_list(
                        ALL_RENDERING_MODE,
                        Some(self.rendering_mode),
                        Message::RenderingMode,
                    ),
                ]
                .align_items(Alignment::Center)
                .spacing(5)
                .width(Length::FillPortion(1)),
                row![
                    text("Background"),
                    pick_list(
                        ALL_BACKGROUND3D,
                        Some(self.background3d),
                        Message::Background3D,
                    ),
                ]
                .align_items(Alignment::Center)
                .spacing(5)
                .width(Length::FillPortion(1)),
            ],
            checkbox("Expand insertions", app_state.expand_insertions())
                .on_toggle(Message::SetExpandInsertions),
        ]
        .spacing(5);

        scrollable(content).into()
    }
}

/// Parameters for the Distance Fog in the 3D view.
struct FogGuiParameters {
    // Is the Distance Fog activated or not.
    is_activated: bool,
    // Compute Distance Fog from the camera or pivot position.
    from_camera: bool,
    // Turn object into dark instead of disappearing.
    dark: bool,
    // Deepness of the Distance Fog.
    length: f32,
    // Softness of the Distance Fog cutoff.
    softness: f32,
    // Reverse the effect.
    is_reversed: bool,
}

impl FogGuiParameters {
    fn view<State: AppState>(&self, ui_size: UiSize) -> iced::Element<'_, Message<State>> {
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
            slider(0f32..=100f32, self.length, Message::FogLength)
        } else {
            slider(0f32..=100f32, self.length, |_| Message::Nothing).style(theme::DeactivatedSlider)
        };

        let softness_slider = if self.is_activated {
            slider(0f32..=100f32, self.softness, Message::FogRadius)
        } else {
            slider(0f32..=100f32, self.softness, |_| Message::Nothing)
                .style(theme::DeactivatedSlider)
        };

        // Hand method to
        let label_width = 65.0f32;

        self::column![
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
                    Message::FogChoice,
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
