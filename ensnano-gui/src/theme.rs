//! ENSnano theme
//!
//! The theme of the GUI is defined here

use iced::advanced::renderer::Style;
pub use iced::Theme;
use iced::{border::Radius, theme, theme::Palette, Background, Border, Color};
use iced_widget::{container, slider, text_input};

/// Color palette
pub const GUI_PALETTE: Palette = Palette {
    background: Color::from_rgb(0.1, 0.1, 0.1),
    text: Color::WHITE,
    primary: Color::from_rgb(0.2, 0.2, 0.3),
    success: Color::from_rgb(0.5, 1.0, 0.5),
    danger: Color::from_rgb(1.0, 0.5, 0.5),
};

pub fn gui_theme() -> Theme {
    Theme::custom("ENSnano UI Theme".to_string(), GUI_PALETTE)
}

pub fn gui_style(theme: &Theme) -> Style {
    Style {
        text_color: theme.palette().text,
    }
}

pub fn disabled_text() -> theme::Text {
    theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))
}

/// Custom StyleSheet for the background of top_bar, status_bar, and left_pannel.
#[derive(Default)]
pub struct GuiBackground;

// Implement the style sheet using GUI_PALETTE
impl container::StyleSheet for GuiBackground {
    type Style = iced::Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(Background::from(GUI_PALETTE.background)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::from(0.0),
            },
            shadow: Default::default(),
        }
    }
}

// Convenience function to use be able to write `container(…).style(GuiBackground)`
impl From<GuiBackground> for theme::Container {
    fn from(_: GuiBackground) -> Self {
        Self::Custom(Box::new(GuiBackground))
    }
}

/// Custom StyleSheet for disabled sliders.
#[derive(Default)]
pub struct DeactivatedSlider;

impl slider::StyleSheet for DeactivatedSlider {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: ([0.6, 0.6, 0.6, 0.5].into(), Color::WHITE),
                width: 8.0,
                border_radius: Radius::from(1.0),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 8,
                    border_radius: Radius::from(4.0),
                },
                color: Color::from_rgb(0.65, 0.65, 0.65),
                border_color: Color::from_rgb(0.6, 0.6, 0.6),
                border_width: 1.0,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        self.active(style)
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        self.active(style)
    }
}

impl From<DeactivatedSlider> for theme::Slider {
    fn from(_: DeactivatedSlider) -> Self {
        Default::default()
    }
}

// A TextInput that changes appareance when the contained value is bad.
pub struct BadValue(pub bool);

impl text_input::StyleSheet for BadValue {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::WHITE),
            border: Border {
                color: Color::from_rgb(0.7, 0.7, 0.7),
                width: Default::default(),
                radius: Radius::from(5.0),
            },
            icon_color: Default::default(), // TODO:Choose an appropriate value for this field.
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                ..self.active(style).border
            },
            ..self.active(style)
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        if self.0 {
            Color::from_rgb(0.3, 0.3, 0.3)
        } else {
            Color::from_rgb(1., 0.3, 0.3)
        }
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.4, 0.4, 0.4) // TODO: Choose an appropriate value for this field
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            // TODO: Choose an appropriate value for this field
            border: Border {
                color: Color::from_rgb(0.4, 0.4, 0.4),
                ..self.active(style).border
            },
            ..self.active(style)
        }
    }
}
impl From<BadValue> for iced::theme::TextInput {
    fn from(_: BadValue) -> Self {
        Default::default()
    }
}
