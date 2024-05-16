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
use iced::{Font, Length};
pub use iced_widget::*;

const JUMP_SIZE: f32 = 4.0;
//pub(super) const ENSNANO_FONT: Font = Font::External {
//    name: "EnsNanoFont",
//    bytes: include_bytes!("../../font/ensnano.ttf"),
//};
pub(super) const ENSNANO_FONT: Font = Font::with_name("EnsNanoFont");

pub fn load_fonts2() -> iced::Command<Result<(), iced::font::Error>> {
    let command = iced::Command::batch(vec![iced::font::load(
        include_bytes!("../../font/ensnano.ttf").as_slice(),
    )]);
    command
}

/// Add vertical space of [JUMP_SIZE] amount
pub fn extra_jump() -> Space {
    jump_by(JUMP_SIZE)
}

/// Add vertical space of specified amount.
pub fn jump_by(amount: impl Into<Length>) -> Space {
    Space::with_height(amount)
}

/// Section title widget
pub fn section<'a, Theme, Renderer>(
    title: impl ToString,
    ui_size: UiSize,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
{
    text(title).size(ui_size.head_text())
}

/// Section subtitle widget
pub fn subsection<'a, Theme, Renderer>(
    title: impl ToString,
    ui_size: UiSize,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
{
    text(title).size(ui_size.intermediate_text())
}

/// Return a text widget containing the rotation arrow.
pub fn rotation_text<'a>(i: usize, ui_size: UiSize) -> Text<'a> {
    match i {
        0 => material_icons_light::dark_icon(LightIcon::ArrowBack, ui_size),
        1 => material_icons_light::dark_icon(LightIcon::ArrowForward, ui_size),
        2 => material_icons_light::dark_icon(LightIcon::ArrowUpward, ui_size),
        3 => material_icons_light::dark_icon(LightIcon::ArrowDownward, ui_size),
        4 => material_icons_light::dark_icon(LightIcon::Undo, ui_size),
        _ => material_icons_light::dark_icon(LightIcon::Redo, ui_size),
    }
}

/// Return a button containing an icon in the light theme.
pub fn light_icon_button<'a, Message, Theme, Renderer>(
    icon: material_icons_light::LightIcon,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::StyleSheet + text::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
    <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
{
    button(material_icons_light::light_icon(icon, ui_size)).height(ui_size.button())
}

/// Return a button containing an icon in the light theme.
pub fn dark_icon_button<'a, Message, Theme, Renderer>(
    icon: material_icons_light::LightIcon,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::StyleSheet + text::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
    <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
{
    button(material_icons_light::dark_icon(icon, ui_size)).height(ui_size.button())
}

/// Return a text button.
pub fn text_button<'a, Message, Theme, Renderer>(
    label: impl ToString,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::StyleSheet + text::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
{
    button(text(label).size(ui_size.main_text())).height(ui_size.button())
}

/// A button containing an icon.
pub fn icon_button<'a, Message: Clone>(icon_char: char, ui_size: UiSize) -> Button<'a, Message> {
    button(
        text(icon_char.to_string())
            .font(ENSNANO_FONT)
            .size(ui_size.icon()),
    )
    .height(ui_size.button())
}

/// Return a button that starts, then stops something.
pub fn start_stop_button<'a, F, Message, Theme, Renderer>(
    label: impl ToString,
    ui_size: UiSize,
    start_stop_switch: Option<F>,
    is_started: bool,
) -> Button<'a, Message, Theme, Renderer>
where
    F: 'static + Fn(bool) -> Message,
    Theme: button::StyleSheet + text::StyleSheet,
    <Theme as iced_widget::button::StyleSheet>::Style: From<iced::theme::Button>,
    Renderer: iced::advanced::text::Renderer,
    <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
{
    let style = if is_started {
        theme::Button::Destructive
    } else {
        theme::Button::Positive
    };
    let mut start_stop_button = text_button(label, ui_size).style(style);
    // NOTE: In the previous version of the start_stop_button (i.g. GoStop),
    //       the label was replaced by “Stop”, whereas here only the color changes.
    //       It may be a good idea tho visually reintroduce the current state, via
    //       logos such as: ⏵ ⏸ ⏺ ⏹
    if let Some(send_start_stop_message) = start_stop_switch {
        start_stop_button = start_stop_button.on_press(send_start_stop_message(!is_started));
        // The action is to reverset the state.
    }
    start_stop_button
}

/// Return a checkbox widget with its label placed on the left.
pub fn right_checkbox<'a, Message, Theme, Renderer>(
    is_checked: bool,
    label: impl ToString,
    toggle_message: impl Fn(bool) -> Message + 'a,
    ui_size: UiSize,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: text::StyleSheet + checkbox::StyleSheet,
    Renderer: iced::advanced::text::Renderer,
{
    row![
        text(label),
        checkbox("", is_checked)
            .on_toggle(toggle_message)
            .size(ui_size.checkbox()),
    ]
    .spacing(ui_size.checkbox_spacing())
}
