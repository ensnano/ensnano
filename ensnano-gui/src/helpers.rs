use crate::fonts::{
    ENSNANO_FONT, INTER_BOLD_FONT,
    material_icons::{
        MATERIAL_ICONS_DARK, MATERIAL_ICONS_LIGHT, MaterialIcon, MaterialIconStyle, icon_to_char,
    },
};
use ensnano_iced::ui_size::UiSize;
use iced::{
    Length,
    alignment::{Alignment, Horizontal},
    widget::{Button, Image, Row, Space, Text, button, checkbox, image::Handle, row, text},
};

// === SPACING FUNCTIONS ===

const JUMP_SIZE: f32 = 4.0;

/// Add vertical space of fixed amount
pub(crate) fn extra_jump() -> Space {
    jump_by(JUMP_SIZE)
}

/// Add vertical space of specified amount.
pub(crate) fn jump_by(amount: impl Into<Length>) -> Space {
    Space::with_height(amount)
}

// === TEXT FUNCTIONS ===

/// Section title widget
pub(crate) fn section<'a>(title: impl ToString, ui_size: UiSize) -> Text<'a> {
    text(title).size(ui_size.head_text()).font(INTER_BOLD_FONT)
}

/// Section subtitle widget
pub(crate) fn subsection<'a>(title: impl ToString, ui_size: UiSize) -> Text<'a> {
    text(title).size(ui_size.intermediate_text())
}

// === ICON FUNCTIONS ===

pub(crate) fn material_icon<'a>(
    icon: MaterialIcon,
    style: MaterialIconStyle,
    ui_size: UiSize,
) -> Text<'a> {
    text(icon_to_char(icon))
        .font(match style {
            MaterialIconStyle::Light => MATERIAL_ICONS_LIGHT,
            MaterialIconStyle::Dark => MATERIAL_ICONS_DARK,
        })
        .size(ui_size.icon())
}

// === BUTTON FUNCTIONS ===

// NOTE: It seems since iced 0.12 that giving a size to a button make the (text) content disappear,
//       therefore we give the size to the underlying text.

// NOTE: This wrapper ensures that every button has a consistent shape.
macro_rules! button_text_wrapper {
    ($text:expr, $ui_size:ident) => {
        iced::widget::button(
            $text
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
        .padding($ui_size.button_pad())
    };
}
pub(crate) use button_text_wrapper;

/// Return a text button.
pub(crate) fn text_button<'a, Message>(
    label: impl ToString,
    ui_size: UiSize,
) -> Button<'a, Message> {
    button_text_wrapper!(text(label).size(ui_size.main_text()), ui_size).height(ui_size.button())
}

pub(crate) fn fixed_text_button<'a, Message>(
    label: impl ToString,
    width_factor: f32,
    ui_size: UiSize,
) -> Button<'a, Message> {
    button_text_wrapper!(
        text(label)
            .size(ui_size.main_text())
            .width(width_factor * ui_size.button()),
        ui_size
    )
    .height(ui_size.button())
}

/// Return a button containing an icon in the light theme.
pub(crate) fn material_icon_button<'a, Message>(
    icon: MaterialIcon,
    style: MaterialIconStyle,
    ui_size: UiSize,
) -> Button<'a, Message> {
    button_text_wrapper!(material_icon(icon, style, ui_size), ui_size)
        .height(ui_size.button())
        .width(ui_size.button())
}

/// A button containing an icon from the ENSNANO font.
pub(crate) fn icon_button<'a, Message>(icon_char: char, ui_size: UiSize) -> Button<'a, Message>
where
    Message: Clone,
{
    button_text_wrapper!(
        text(icon_char).font(ENSNANO_FONT).size(ui_size.icon()),
        ui_size
    )
    .height(ui_size.button())
    .width(ui_size.button())
}

/// A button containing an icon.
pub(crate) fn image_button<'a, Message>(
    image: Image<Handle>,
    ui_size: UiSize,
) -> Button<'a, Message>
where
    Message: Clone + 'a,
{
    button(row![image].align_items(Alignment::Center))
        .height(ui_size.button())
        .width(ui_size.button())
}

/// Return a button that starts, then stops something.
pub(crate) fn start_stop_button<'a, F, Message>(
    label: impl ToString,
    ui_size: UiSize,
    start_stop_switch: Option<F>,
    is_started: bool,
) -> Button<'a, Message>
where
    F: 'static + Fn(bool) -> Message,
{
    let style = if is_started {
        iced::theme::Button::Destructive
    } else {
        iced::theme::Button::Positive
    };
    let mut start_stop_button = text_button(label, ui_size).style(style);
    // NOTE: In the previous version of the start_stop_button (i.e. GoStop),
    //       the label was replaced by “Stop”, whereas here only the color changes.
    //       It may be a good idea tho visually reintroduce the current state, via
    //       logos such as: ⏵ ⏸ ⏺ ⏹
    if let Some(send_start_stop_message) = start_stop_switch {
        start_stop_button = start_stop_button.on_press(send_start_stop_message(!is_started));
    }
    start_stop_button
}

// === CHECKBOX FUNCTIONS ===

/// Return a checkbox widget with its label placed on the left.
pub(crate) fn right_checkbox<'a, Message>(
    is_checked: bool,
    label: impl ToString,
    toggle_message: impl Fn(bool) -> Message + 'a,
    ui_size: UiSize,
) -> Row<'a, Message>
where
    Message: 'a,
{
    row![
        text(label),
        checkbox("", is_checked)
            .on_toggle(toggle_message)
            .size(ui_size.checkbox()),
    ]
    .spacing(ui_size.checkbox_spacing())
}
