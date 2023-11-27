//! Allow widgets to emmit messages when hovered
//!
//! A [`HoverableContainer`] is an `iced_native::Container` that produces a messages when hovered.

// This file is manifestly a copy-paste of the iced_native::widget::container source code
//
//    https://docs.rs/iced_native/0.9.1/src/iced_native/widget/container.rs.html

use iced_native::alignment::{self, Alignment};
use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::widget::{self};
use iced_native::{
    Background, Clipboard, Color, Element, Layout, Length, Padding, Point, Rectangle, Shell, Widget,
};

pub use iced_style::container::{Appearance, StyleSheet};

use std::u32;

/// The local state of an [`HoverableContainer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_hovered: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}
/// NOTE: Consider to delete state sto follow the “stateless” paradigm of iced.

/// An `iced_native::Container` that emits a message when hovered.
#[allow(missing_debug_implementations)]
pub struct HoverableContainer<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    padding: Padding,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    style: <Renderer::Theme as widget::container::StyleSheet>::Style,
    content: Element<'a, Message, Renderer>,
    on_hovered_in: Option<Message>,
    on_hovered_out: Option<Message>,
}

impl<'a, Message, Renderer> HoverableContainer<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates an empty [Container](iced::widget::container::Container).
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        HoverableContainer {
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            style: Default::default(),
            content: content.into(),
            on_hovered_in: None,
            on_hovered_out: None,
        }
    }

    /// Sets the [`Padding`] of the [Container](iced::widget::container::Container).
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the width of the [Container](iced::widget::container::Container).
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [Container](iced::widget::container::Container).
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [Container](iced::widget::container::Container).
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [Container](iced::widget::container::Container) in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the content alignment for the horizontal axis of the [Container](iced::widget::container::Container).
    pub fn align_x(mut self, alignment: alignment::Horizontal) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [Container](iced::widget::container::Container).
    pub fn align_y(mut self, alignment: alignment::Vertical) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Centers the contents in the horizontal axis of the [Container](iced::widget::container::Container).
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = alignment::Horizontal::Center;
        self
    }

    /// Centers the contents in the vertical axis of the [Container](iced::widget::container::Container).
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = alignment::Vertical::Center;
        self
    }

    /// Set the appearance of the [Container](iced::widget::container::Container).
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as iced_native::widget::container::StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    pub fn on_hovered_in(mut self, message: Message) -> Self {
        self.on_hovered_in = Some(message);
        self
    }

    pub fn on_hovered_out(mut self, message: Message) -> Self {
        self.on_hovered_out = Some(message);
        self
    }
}

/// Computes the layout of a [Container](iced::widget::container::Container).
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    padding: Padding,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.loose().width(width).height(height).pad(padding);

    let mut content = layout_content(renderer, &limits.loose());
    let size = limits.resolve(content.size());

    content.move_to(Point::new(padding.left.into(), padding.top.into()));
    content.align(
        Alignment::from(horizontal_alignment),
        Alignment::from(vertical_alignment),
        size,
    );

    layout::Node::with_children(size.pad(padding), vec![content])
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for HoverableContainer<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::Renderer,
    Renderer::Theme: iced_native::widget::container::StyleSheet + StyleSheet,
{
    fn state(&self) -> widget::tree::State {
        widget::tree::State::Some(Box::new(State::default()))
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            self.horizontal_alignment,
            self.vertical_alignment,
            |renderer, limits| self.content.as_widget().layout(renderer, limits),
        )
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            tree,
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            event::Status::Captured
        } else {
            if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
                let state = tree.state.downcast_mut::<State>();
                let bounds = layout.bounds();
                if bounds.contains(cursor_position) {
                    if !state.is_hovered {
                        if let Some(on_hovered_in) = self.on_hovered_in.clone() {
                            shell.publish(on_hovered_in)
                        }
                        state.is_hovered = true;
                    }
                } else {
                    if state.is_hovered {
                        if let Some(on_hovered_out) = self.on_hovered_out.clone() {
                            shell.publish(on_hovered_out)
                        }
                        state.is_hovered = false;
                    }
                }
            }
            event::Status::Ignored
        }
    }

    fn mouse_interaction(
        &self,
        state: &widget::Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            state,
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let style = theme.appearance(&self.style);

        draw_background(renderer, &style, layout.bounds());

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color.unwrap_or(renderer_style.text_color),
            },
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(state, layout.children().next().unwrap(), renderer)
    }
}

/// Draws the background of a
/// [Container](iced::widget::container::Container) given its [Appearance] and its `bounds`.
pub fn draw_background<Renderer>(renderer: &mut Renderer, style: &Appearance, bounds: Rectangle)
where
    Renderer: iced_native::Renderer,
{
    if style.background.is_some() || style.border_width > 0.0 {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: style.border_radius.into(),
                border_width: style.border_width,
                border_color: style.border_color,
            },
            style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
        );
    }
}

impl<'a, Message, Renderer> From<HoverableContainer<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Message: Clone,
    Renderer: 'a + iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(column: HoverableContainer<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
