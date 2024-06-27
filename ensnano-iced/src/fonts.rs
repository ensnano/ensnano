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

pub mod material_icons;

const ENSNANO_FONT_BYTES: &[u8] = include_bytes!("../../font/ensnano2.ttf");

// NOTE: We export here all fonts used in ENSnano.
pub use iced_aw::BOOTSTRAP_FONT;
pub const ENSNANO_FONT: Font = Font::with_name("Ensnano");
pub use material_icons::{
    icon_to_char, MaterialIcon, MaterialIconStyle, MATERIAL_ICONS_DARK, MATERIAL_ICONS_LIGHT,
};

pub use iced::{font::Error, Font};

use iced::{
    advanced,
    widget::{text, Text},
};

// https://rsms.me/inter

/// Load custom font for ENSnano GUI.
pub fn load_fonts(renderer: &mut impl advanced::text::Renderer) {
    let fonts = [
        iced_aw::BOOTSTRAP_FONT_BYTES,
        material_icons::MATERIAL_ICONS_LIGHT_BYTES,
        material_icons::MATERIAL_ICONS_DARK_BYTES,
        ENSNANO_FONT_BYTES,
    ];
    for font in fonts {
        renderer.load_font(Cow::from(font));
    }
}

pub fn light_icon<'a, Theme, Renderer>(
    icon: MaterialIcon,
    //ui_size: UiSize,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: advanced::Renderer + advanced::text::Renderer,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    text(icon_to_char(icon)).font(MATERIAL_ICONS_LIGHT)
    //.size(ui_size.icon())
}

pub fn dark_icon<'a, Theme, Renderer>(
    icon: MaterialIcon,
    //ui_size: UiSize,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: advanced::Renderer + advanced::text::Renderer,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    text(icon_to_char(icon)).font(MATERIAL_ICONS_DARK)
    //.size(ui_size.icon())
}
