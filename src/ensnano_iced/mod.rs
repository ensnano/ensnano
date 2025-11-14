//! Iced interface to ENSnano
//!
//! The objective of this crate is two-fold:
//! 1. Ensure all other crates uses the same version of iced,
//! 2. Provide customized widgets and tools to ease the building of the GUI.
//!
//! Therefore, in other crates, do not use Iced directly, but go through ensnano_iced.

pub type Element<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> =
    iced::Element<'a, Message, Theme, Renderer>;

pub mod fonts;

mod ui_size;
pub use ui_size::{ALL_UI_SIZES, UiSize};

pub mod widgets;

pub mod helpers;

pub mod theme;
pub use theme::Theme;

pub mod color_picker;

mod icons;
pub use icons::icon_to_svg;
