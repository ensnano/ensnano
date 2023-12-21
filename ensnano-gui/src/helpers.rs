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
//! Additional Iced view helpers for ENSnano.
//!
//! Repetitive, complex widgets are factored here.
use super::UiSize;
use crate::material_icons_light::{self, LightIcon};
use iced::{theme, widget, Length};
use iced_native::widget::helpers::*;

const CHECKBOXSPACING: u16 = 5;
const JUMP_SIZE: f32 = 4.0;
pub(super) const ENSNANO_FONT: iced::Font = iced::Font::External {
    name: "EnsNanoFont",
    bytes: include_bytes!("../../font/ensnano.ttf"),
};
const LIGHT_ICONFONT: iced::Font = iced::Font::External {
    name: "IconFontLight",
    bytes: material_icons_light::MATERIAL_ICON_LIGHT,
};

/// Add vertical space of [JUMP_SIZE] amount
pub fn extra_jump() -> widget::Space {
    jump_by(JUMP_SIZE)
}

/// Add vertical space of specified amount.
pub fn jump_by(amount: impl Into<Length>) -> widget::Space {
    vertical_space(amount)
}

/// Section title widget
pub fn section<'a>(title: impl ToString, ui_size: UiSize) -> widget::Text<'a> {
    text(title).size(ui_size.head_text())
}

/// Section subtitle widget
pub fn subsection<'a>(title: impl ToString, ui_size: UiSize) -> widget::Text<'a> {
    text(title).size(ui_size.intermediate_text())
}

/// Return a text widget containing the rotation arrow.
pub fn rotation_text<'a>(i: usize, ui_size: UiSize) -> widget::Text<'a> {
    match i {
        0 => material_icons_light::dark_icon(LightIcon::ArrowBack, ui_size),
        1 => material_icons_light::dark_icon(LightIcon::ArrowForward, ui_size),
        2 => material_icons_light::dark_icon(LightIcon::ArrowUpward, ui_size),
        3 => material_icons_light::dark_icon(LightIcon::ArrowDownward, ui_size),
        4 => material_icons_light::dark_icon(LightIcon::Undo, ui_size),
        _ => material_icons_light::dark_icon(LightIcon::Redo, ui_size),
    }
}
/// Return a text widget containing an icon in the light theme.
pub fn light_icon<'a>(icon: LightIcon, ui_size: UiSize) -> widget::Text<'a> {
    text(format!("{}", material_icons_light::icon_to_char(icon)))
        .font(LIGHT_ICONFONT)
        .size(ui_size.icon())
}
/// Return a button containing an icon in the light theme.
pub fn light_icon_button<'a, Message>(
    icon: LightIcon,
    ui_size: UiSize,
) -> widget::Button<'a, Message> {
    button(light_icon(icon, ui_size)).height(ui_size.button())
}

/// Return a text button.
///
/// WARNING: If the length of `inner_text` is 1, then it is assumed to represent an icon.
pub fn text_button<'a, Message: Clone>(
    inner_text: &'a str,
    ui_size: UiSize,
) -> widget::Button<'a, Message> {
    let size = if inner_text.len() > 1 {
        ui_size.main_text()
    } else {
        ui_size.icon()
    };
    button(text(inner_text).size(size)).height(ui_size.button())
}

/// A button containing an icon.
pub fn icon_button<'a, Message: Clone>(
    icon_char: char,
    ui_size: UiSize,
) -> widget::Button<'a, Message> {
    button(
        text(icon_char.to_string())
            .font(ENSNANO_FONT)
            .size(ui_size.icon()),
    )
    .height(ui_size.button())
}

/// Return a checkbox widget with its label placed on the left.
pub fn right_checkbox<'a, Message: 'a>(
    is_checked: bool,
    label: impl ToString,
    f: impl Fn(bool) -> Message + 'a,
    ui_size: UiSize,
) -> iced::widget::Row<'a, Message> {
    iced_native::row![
        text(label),
        checkbox("", is_checked, f).size(ui_size.checkbox()),
    ]
    .spacing(CHECKBOXSPACING)
}
