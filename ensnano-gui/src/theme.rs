//! ENSnano theme
//!
//! The theme of the GUI is defined here

use iced::{theme::Palette, Color, Theme};
use iced_native::renderer::Style;

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
