//! Iced interface to ENSnano
//!
//! The objective of this crate is two-fold:
//! 1. Ensure all other crates uses the same version of iced,
//! 2. Provide customized widgets and tools to ease the building of the GUI.
//!
//! Therefore, in other crates, do not use Iced directly, but go through ensnano_iced.

pub mod color_picker;
pub mod fonts;
pub mod helpers;
pub mod theme;
pub mod widgets;

mod ui_size;
pub use ui_size::{ALL_UI_SIZES, UiSize};

mod icons;
pub use icons::icon_to_svg;
