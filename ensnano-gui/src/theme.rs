//! ENSnano theme
//!
//! The theme of the GUI is defined here

use iced::{theme, theme::Palette, Background, Color, Theme};
use iced_native::renderer::Style;
use iced_native::widget::container;

/// Color palette
pub const GUI_PALETTE: Palette = Palette {
    background: Color::from_rgb(0.1, 0.1, 0.1),
    text: Color::WHITE,
    primary: Color::from_rgb(0.2, 0.2, 0.3),
    success: Color::from_rgb(0.5, 1.0, 0.5),
    danger: Color::from_rgb(1.0, 5.0, 0.5),
};

pub fn gui_theme() -> Theme {
    Theme::custom(GUI_PALETTE)
}

pub fn gui_style(theme: &Theme) -> Style {
    Style {
        text_color: theme.palette().text,
    }
}

/// Custom styleSheet for the background of top_bar, status_bar, and left_pannel.
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
