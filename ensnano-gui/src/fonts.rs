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
// NOTE: Icon font used to be loaded by hand, but now the bootstrap icons
//       are included in iced_aw, so we use them directly.
//
// NOTE: Other help from forums
//
//        https://github.com/BillyDM/iced_baseview/issues/39
use std::borrow::Cow;

use crate::material_icons_light;
use iced::{advanced::text, Font};

pub const ENSNANO_FONT_BYTES: &[u8] = include_bytes!("../../font/ensnano2.ttf");
pub const ENSNANO_FONT: Font = Font::with_name("Ensnano");

// https://rsms.me/inter

/// Load custom font for ENSnano GUI.
pub fn load_fonts(renderer: &mut impl text::Renderer) {
    let fonts = [
        material_icons_light::MATERIAL_ICONS_LIGHT_BYTES,
        material_icons_light::MATERIAL_ICONS_DARK_BYTES,
        crate::fonts::ENSNANO_FONT_BYTES,
    ];
    for font in fonts {
        renderer.load_font(Cow::from(font));
    }
}
