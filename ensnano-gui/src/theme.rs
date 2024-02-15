//! ENSnano theme
//!
//! The theme of the GUI is defined here

use iced::{theme, theme::Palette, Background, Color, Theme};
use iced_native::renderer::Style;
use iced_native::widget::{container, slider};

/// Color palette
pub const GUI_PALETTE: Palette = Palette {
    background: Color::from_rgb(0.1, 0.1, 0.1),
    text: Color::WHITE,
    primary: Color::from_rgb(0.2, 0.2, 0.3),
    success: Color::from_rgb(0.5, 1.0, 0.5),
    danger: Color::from_rgb(1.0, 0.5, 0.5),
};

pub fn gui_theme() -> Theme {
    Theme::custom(GUI_PALETTE)
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
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
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
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Rectangle {
                    width: 8,
                    border_radius: 4.0,
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
