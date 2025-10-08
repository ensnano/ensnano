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
pub use crate::widgets::*;
use crate::{UiSize, fonts::*};
pub use iced::widget::*;
use iced::{
    Font, Length, advanced,
    alignment::{Alignment, Horizontal, Vertical},
};

///
/// SPACING FUNCTIONS.
///

const JUMP_SIZE: f32 = 4.0;

/// Add vertical space of [JUMP_SIZE] amount
pub fn extra_jump() -> Space {
    jump_by(JUMP_SIZE)
}

/// Add vertical space of specified amount.
pub fn jump_by(amount: impl Into<Length>) -> Space {
    Space::with_height(amount)
}

///
/// TEXT FUNCTIONS.
///

/// Section title widget
pub fn section<'a, Theme, Renderer>(
    title: impl ToString,
    ui_size: UiSize,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: advanced::text::Renderer,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    text(title).size(ui_size.head_text()).font(INTER_BOLD_FONT)
}

/// Section subtitle widget
pub fn subsection<'a, Theme, Renderer>(
    title: impl ToString,
    ui_size: UiSize,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: advanced::text::Renderer,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    text(title).size(ui_size.intermediate_text())
}

///
/// ICON FUNCTIONS.
///

pub fn material_icon<'a, Theme, Renderer>(
    icon: MaterialIcon,
    style: MaterialIconStyle,
    ui_size: UiSize,
) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: advanced::Renderer + advanced::text::Renderer,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    text(icon_to_char(icon))
        .font(match style {
            MaterialIconStyle::Light => material_icons::MATERIAL_ICONS_LIGHT,
            MaterialIconStyle::Dark => material_icons::MATERIAL_ICONS_DARK,
        })
        .size(ui_size.icon())
}

/// Return a text widget containing the rotation arrow.
fn rotation_icon<'a, Theme, Renderer>(i: usize, ui_size: UiSize) -> Text<'a, Theme, Renderer>
where
    Theme: text::StyleSheet,
    Renderer: advanced::text::Renderer,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    match i {
        0 => material_icon(MaterialIcon::ArrowBack, MaterialIconStyle::Dark, ui_size),
        1 => material_icon(MaterialIcon::ArrowForward, MaterialIconStyle::Dark, ui_size),
        2 => material_icon(MaterialIcon::ArrowUpward, MaterialIconStyle::Dark, ui_size),
        3 => material_icon(
            MaterialIcon::ArrowDownward,
            MaterialIconStyle::Dark,
            ui_size,
        ),
        4 => material_icon(MaterialIcon::Undo, MaterialIconStyle::Dark, ui_size),
        _ => material_icon(MaterialIcon::Redo, MaterialIconStyle::Dark, ui_size),
    }
}

///
/// BUTTON FUNCTIONS.
///

// NOTE: It seems since iced 0.12 that giving a size to a button make the (text) content disappear,
//       therefore we give the size to the underlying text.

// NOTE: This wrapper ensures that every button has a consisent shape.
macro_rules! button_text_wrapper {
    ($text:expr, $ui_size:ident) => {
        button(
            $text
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center),
        )
        .padding($ui_size.button_pad())
    };
}

/// Return a text button.
pub fn text_button<'a, Message, Theme, Renderer>(
    label: impl ToString,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::StyleSheet + text::StyleSheet + 'a,
    Renderer: advanced::Renderer + advanced::text::Renderer + 'a,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    button_text_wrapper!(text(label).size(ui_size.main_text()), ui_size).height(ui_size.button())
}
pub fn fixed_text_button<'a, Message, Theme, Renderer>(
    label: impl ToString,
    width_factor: f32,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::StyleSheet + text::StyleSheet + 'a,
    Renderer: advanced::Renderer + advanced::text::Renderer + 'a,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    button_text_wrapper!(
        text(label)
            .size(ui_size.main_text())
            .width(width_factor * ui_size.button()),
        ui_size
    )
    .height(ui_size.button())
}

/// Return a button containing an icon in the light theme.
pub fn material_icon_button<'a, Message, Theme, Renderer>(
    icon: MaterialIcon,
    style: MaterialIconStyle,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::StyleSheet + text::StyleSheet + 'a,
    Renderer: advanced::text::Renderer + 'a,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    button_text_wrapper!(material_icon(icon, style, ui_size), ui_size)
        .height(ui_size.button())
        .width(ui_size.button())
}

pub fn rotation_icon_button<'a, Message, Theme, Renderer>(
    i: usize,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::StyleSheet + text::StyleSheet + 'a,
    Renderer: advanced::text::Renderer + 'a,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    button_text_wrapper!(rotation_icon(i, ui_size).height(ui_size.button()), ui_size)
}

/// A button containing an icon from the ENSNANO font.
pub fn icon_button<'a, Message, Theme, Renderer>(
    icon_char: char,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: button::StyleSheet + text::StyleSheet + 'a,
    Renderer: advanced::Renderer + advanced::text::Renderer + 'a,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    button_text_wrapper!(
        text(icon_char).font(ENSNANO_FONT).size(ui_size.icon()),
        ui_size
    )
    .height(ui_size.button())
    .width(ui_size.button())
}

/// A button containing an icon.
pub fn image_button<'a, Message, Theme, Renderer, Handle>(
    image: Image<Handle>,
    ui_size: UiSize,
) -> Button<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: button::StyleSheet + text::StyleSheet + 'a,
    Renderer: advanced::Renderer
        + advanced::text::Renderer
        + advanced::image::Renderer<Handle = Handle>
        + 'a,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
    Handle: std::hash::Hash + Clone + 'a,
{
    button(row![image].align_items(Alignment::Center))
        .height(ui_size.button())
        .width(ui_size.button())
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
    Theme: button::StyleSheet + text::StyleSheet + 'a,
    <Theme as button::StyleSheet>::Style: From<theme::Button>,
    Renderer: advanced::text::Renderer + 'a,
    <Renderer as advanced::text::Renderer>::Font: From<Font>,
{
    let style = if is_started {
        theme::Button::Destructive
    } else {
        theme::Button::Positive
    };
    let mut start_stop_button = text_button(label, ui_size).style(style);
    // NOTE: In the previous version of the start_stop_button (i.e. GoStop),
    //       the label was replaced by “Stop”, whereas here only the color changes.
    //       It may be a good idea tho visually reintroduce the current state, via
    //       logos such as: ⏵ ⏸ ⏺ ⏹
    if let Some(send_start_stop_message) = start_stop_switch {
        start_stop_button = start_stop_button.on_press(send_start_stop_message(!is_started));
        // The action is to reverset the state.
    }
    start_stop_button
}

///
/// CHECKBOXES
///

/// Return a checkbox widget with its label placed on the left.
pub fn right_checkbox<'a, Message, Theme, Renderer>(
    is_checked: bool,
    label: impl ToString,
    toggle_message: impl Fn(bool) -> Message + 'a,
    ui_size: UiSize,
) -> Row<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: text::StyleSheet + checkbox::StyleSheet + 'a,
    Renderer: advanced::text::Renderer + 'a,
{
    row![
        text(label),
        checkbox("", is_checked)
            .on_toggle(toggle_message)
            .size(ui_size.checkbox()),
    ]
    .spacing(ui_size.checkbox_spacing())
}
