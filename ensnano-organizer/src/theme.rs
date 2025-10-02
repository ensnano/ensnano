//! Customize the theme of ENSnano
use ensnano_iced::iced::{
    border::Radius,
    theme,
    widget::{button, container},
    Background, Border, Color, Shadow, Vector,
};

#[derive(Debug, Clone, Copy)]
struct ColorGradient {
    left: Color,
    middle: Option<Color>,
    right: Color,
}

fn moon_gradient() -> ColorGradient {
    ColorGradient {
        left: Color::from_rgb8(0x0f, 0x20, 0x27),
        middle: Some(Color::from_rgb8(0x20, 0x3a, 0x43)),
        right: Color::from_rgb8(0x2c, 0x53, 0x64),
    }
}

fn grey_gradient() -> ColorGradient {
    ColorGradient {
        left: Color::from_rgb8(0x2c, 0x3e, 0x50),
        middle: None,
        right: Color::from_rgb8(0xbd, 0xc3, 0xc7),
    }
}

impl ColorGradient {
    fn linear_interpolation(&self, x: f32) -> Color {
        if let Some(middle) = self.middle.as_ref() {
            if x <= 0. {
                self.left
            } else if x <= 0.5 {
                let x = x * 2.;
                let interp = |a, b| a * (1. - x) + b * x;
                Color::from_rgb(
                    interp(self.left.r, middle.r),
                    interp(self.left.g, middle.g),
                    interp(self.left.b, middle.b),
                )
            } else if x <= 1. {
                let x = (x - 0.5) * 2.;
                let interp = |a, b| a * (1. - x) + b * x;
                Color::from_rgb(
                    interp(middle.r, self.right.r),
                    interp(middle.g, self.right.g),
                    interp(middle.b, self.right.b),
                )
            } else {
                self.right
            }
        } else {
            if x <= 0. {
                self.left
            } else if x <= 1. {
                let interp = |a, b| a * (1. - x) + b * x;
                Color::from_rgb(
                    interp(self.left.r, self.right.r),
                    interp(self.left.g, self.right.g),
                    interp(self.left.b, self.right.b),
                )
            } else {
                self.right
            }
        }
    }
}

/// “Parent” theme
pub struct OrganizerTheme {
    gradient: ColorGradient,
    text_color: Color,
    border_color: Color,
    max_level: usize,
}

/// “Level” theme
#[derive(Debug, Copy, Clone)]
pub(super) struct OrganizerThemeLevel {
    gradient: ColorGradient,
    text_color: Color,
    border_color: Color,
    gradient_value: f32,
    selected: bool,
}

/// “Selection” theme
pub(super) struct OrganizerThemeSelection {
    selected: bool,
    text_color: Color,
    selected_color: Color,
    border_color: Color,
}

/// Implements the [Button](button::Button) style sheet for [ThemeSelection]
impl button::StyleSheet for OrganizerThemeSelection {
    type Style = ();
    //type Style = iced_style::theme::Button;
    // I think the good way to do it is to implement a custom Style.

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::new(0., 0.),
            background: None,
            text_color: if self.selected {
                self.selected_color
            } else {
                self.text_color
            },
            border: Border {
                color: self.border_color,
                width: if self.selected { 4. } else { 0. },
                radius: Radius::from(0),
            },
            shadow: Shadow {
                color: self.border_color,
                offset: Vector::new(0., 0.),
                blur_radius: 0.,
            },
            // TODO: Check on these values.
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            border: Border {
                width: self.active(style).border.width + 1.,
                ..self.active(style).border
            },
            ..self.active(style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            border: Border {
                width: self.active(style).border.width + 1.,
                ..self.active(style).border
            },
            ..self.active(style)
        }
    }
}

impl From<OrganizerThemeSelection> for theme::Button {
    fn from(_: OrganizerThemeSelection) -> Self {
        Default::default()
    }
}

/// Implements the [Button](button::Button) style sheet for [ThemeLevel]
impl button::StyleSheet for OrganizerThemeLevel {
    type Style = ();
    //type Style = iced_style::theme::Button;
    // I think the good way to do it is to implement a custom Style.

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::new(0., 0.),
            background: None,
            text_color: self.text_color,
            border: Border {
                color: self.border_color,
                width: if self.selected { 4. } else { 0. },
                radius: Radius::from(0),
            },
            shadow: Shadow {
                color: self.border_color,
                offset: Vector::new(0., 0.),
                blur_radius: 0.,
            },
            // TODO: Check on these values.
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            border: Border {
                width: if self.selected { 5. } else { 1. },
                ..self.active(style).border
            },
            ..self.active(style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            border: Border {
                width: 1.,
                ..self.active(style).border
            },
            ..self.active(style)
        }
    }
}

impl From<OrganizerThemeLevel> for theme::Button {
    fn from(_: OrganizerThemeLevel) -> Self {
        Default::default()
    }
}

/// Implements the [Container](container::Container) style sheet for [ThemeLevel]
impl container::StyleSheet for OrganizerThemeLevel {
    type Style = ();
    //type Style = iced_style::theme::Container;
    // I think the good way to do it is to implement a custom Style.

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(
                self.gradient.linear_interpolation(self.gradient_value),
            )),
            ..Default::default()
        }
    }
}

impl From<OrganizerThemeLevel> for theme::Container {
    fn from(_: OrganizerThemeLevel) -> Self {
        Default::default()
    }
}

impl OrganizerTheme {
    pub(super) fn level(&self, n: usize) -> OrganizerThemeLevel {
        OrganizerThemeLevel {
            gradient: self.gradient.clone(),
            text_color: self.text_color.clone(),
            border_color: self.border_color.clone(),
            gradient_value: n as f32 / self.max_level as f32,
            selected: false,
        }
    }

    pub(super) fn level_selected(&self, n: usize) -> OrganizerThemeLevel {
        OrganizerThemeLevel {
            gradient: self.gradient.clone(),
            text_color: self.text_color.clone(),
            border_color: self.border_color.clone(),
            gradient_value: n as f32 / self.max_level as f32,
            selected: true,
        }
    }

    pub fn moon() -> Self {
        Self {
            gradient: moon_gradient(),
            text_color: Color::WHITE,
            border_color: Color::from_rgb8(0x83, 0x1a, 0x1a),
            max_level: 5,
        }
    }

    pub fn grey() -> Self {
        Self {
            gradient: grey_gradient(),
            text_color: Color::WHITE,
            border_color: Color::from_rgb8(0x83, 0x1a, 0x1a),
            max_level: 5,
        }
    }
}
