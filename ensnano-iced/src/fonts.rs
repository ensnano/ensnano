/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// NOTE: Custom fonts became much harder to use since iced 0.10. See:
//
//         https://github.com/iced-rs/iced/discussions/1988
//         https://github.com/fmonniot/pathfinder-wotr-editor/commit/c86fb9a5d2b77b63f284026de3c269fb798dc9ef#diff-42cb6807ad74b3e201c5a7ca98b911c5fa08380e942be6e4ac5807f8377f87fcR106-R116
//
// NOTE: Other help from forums
//
//        https://github.com/BillyDM/iced_baseview/issues/39
use std::borrow::Cow;

use iced::font;

pub mod material_icons;
pub use material_icons::{MATERIAL_ICONS_DARK, MaterialIcon, MaterialIconStyle, icon_to_char};

const ENSNANO_FONT_BYTES: &[u8] = include_bytes!("../../font/ensnano2.ttf");
const INTER_BOLD_FONT_BYTES: &[u8] = include_bytes!("../../font/Inter-Bold.ttf");
const INTER_REGULAR_FONT_BYTES: &[u8] = include_bytes!("../../font/Inter-Regular.ttf");

// NOTE: We export here all fonts used in ENSnano.
pub const ENSNANO_FONT: Font = Font::with_name("Ensnano");
pub const INTER_BOLD_FONT: Font = Font {
    family: font::Family::Name("Inter"),
    weight: font::Weight::Bold,
    style: font::Style::Normal,
    stretch: font::Stretch::Normal,
};
pub const INTER_REGULAR_FONT: Font = Font {
    family: font::Family::Name("Inter"),
    weight: font::Weight::Normal,
    style: font::Style::Normal,
    stretch: font::Stretch::Normal,
};

pub use iced::Font;

// https://rsms.me/inter

/// Load custom font for ENSnano GUI.
pub fn load_fonts(renderer: &mut impl iced::advanced::text::Renderer) {
    let fonts = [
        material_icons::MATERIAL_ICONS_LIGHT_BYTES,
        material_icons::MATERIAL_ICONS_DARK_BYTES,
        ENSNANO_FONT_BYTES,
        INTER_BOLD_FONT_BYTES,
        INTER_REGULAR_FONT_BYTES,
    ];
    for font in fonts {
        renderer.load_font(Cow::from(font));
    }
}
