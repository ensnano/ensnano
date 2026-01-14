//! Customize the theme of ENSnano
use iced::{
    Background, Border, Color, Shadow, Vector,
    border::Radius,
    theme::{Button, Container},
    widget::{button, container},
};

#[derive(Debug, Clone, Copy)]
struct ColorGradient {
    left: Color,
    middle: Option<Color>,
    right: Color,
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
                let lerp = |a, b| a * (1. - x) + b * x;
                Color::from_rgb(
                    lerp(self.left.r, middle.r),
                    lerp(self.left.g, middle.g),
                    lerp(self.left.b, middle.b),
                )
            } else if x <= 1. {
                let x = (x - 0.5) * 2.;
                let lerp = |a, b| a * (1. - x) + b * x;
                Color::from_rgb(
                    lerp(middle.r, self.right.r),
                    lerp(middle.g, self.right.g),
                    lerp(middle.b, self.right.b),
                )
            } else {
                self.right
            }
        } else if x <= 0. {
            self.left
        } else if x <= 1. {
            let lerp = |a, b| a * (1. - x) + b * x;
            Color::from_rgb(
                lerp(self.left.r, self.right.r),
                lerp(self.left.g, self.right.g),
                lerp(self.left.b, self.right.b),
            )
        } else {
            self.right
        }
    }
}

/// “Parent” theme
pub(super) struct OrganizerTheme {
    gradient: ColorGradient,
    text_color: Color,
    border_color: Color,
    partial_select_color: Color,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SelectionType {
    None,
    Partial,
    Full,
}

/// “Selection” theme
struct OrganizerThemeSelection {
    selected: SelectionType,
    text_color: Color,
    partial_select_color: Color,
    border_color: Color,
}

/// Implements the [Button](button::Button) style sheet for [`OrganizerThemeSelection`]
impl button::StyleSheet for OrganizerThemeSelection {
    type Style = iced::Theme;
    //type Style = iced_style::theme::Button;
    // I think the good way to do it is to implement a custom Style.

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::new(0., 0.),
            background: None,
            text_color: self.text_color,
            border: Border {
                color: match self.selected {
                    SelectionType::Partial => self.partial_select_color,
                    SelectionType::None | SelectionType::Full => self.border_color,
                },
                width: if self.selected != SelectionType::None {
                    4.
                } else {
                    0.
                },
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

/// Implements the [Button](button::Button) style sheet for [`OrganizerThemeLevel`]
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

/// Implements the [Container](container::Container) style sheet for [`OrganizerThemeLevel`]
impl container::StyleSheet for OrganizerThemeLevel {
    type Style = iced::Theme;
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

impl OrganizerTheme {
    pub(super) fn level(&self, n: usize) -> <iced::Theme as container::StyleSheet>::Style {
        Container::Custom(Box::new(OrganizerThemeLevel {
            gradient: self.gradient,
            text_color: self.text_color,
            border_color: self.border_color,
            gradient_value: n as f32 / self.max_level as f32,
            selected: false,
        }))
    }

    pub(super) fn selected(
        &self,
        selected: SelectionType,
    ) -> <iced::Theme as button::StyleSheet>::Style {
        Button::Custom(Box::new(OrganizerThemeSelection {
            selected,
            text_color: self.text_color,
            partial_select_color: self.partial_select_color,
            border_color: self.border_color,
        }))
    }

    pub(super) fn grey() -> Self {
        Self {
            gradient: grey_gradient(),
            text_color: Color::WHITE,
            border_color: Color::from_rgb8(0x83, 0x1a, 0x1a),
            partial_select_color: Color::from_rgb(0.9, 0.5, 0.0),
            max_level: 5,
        }
    }
}
