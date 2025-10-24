//! Iced interface to ENSnano
//!
//! The objective of this crate is two-fold:
//! 1. Ensure all other crates uses the same version of iced,
//! 2. Provide customized widgets and tools to ease the building of the GUI.
//!
//! Therefore, in other crates, do not use Iced directly, but go through ensnano_iced.
pub use iced;
pub use iced::Renderer;
pub use iced_aw;
pub use iced_futures;
pub use iced_graphics;
pub use iced_runtime;
pub use iced_wgpu;
pub use iced_widget;
pub use iced_winit;

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
