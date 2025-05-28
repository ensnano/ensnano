//! Allow widgets to emit messages when hovered
//!
//! A [`HoverableContainer`] is an widget that produces a messages when hovered or unhovered.
//!
//! This widget is greatly inspired by
//!
//!    https://giesch.dev/iced-hoverable/
//!
//! see also
//!
//!    https://docs.rs/iced_widget/0.12.1/src/iced_widget/container.rs.html

use ensnano_iced::iced::{
    advanced::{
        layout::{self, Layout},
        mouse, renderer,
        widget::{self, Widget},
        Clipboard, Shell,
    },
    event, overlay, Element, Length, Padding, Point, Rectangle, Size, Vector,
};

/// A widget that emits a message when hovered.
pub struct HoverableContainer<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    padding: Padding,
    content: Element<'a, Message, Theme, Renderer>,
    on_hover: Option<Message>,
    on_unhover: Option<Message>,
}

impl<'a, Message, Theme, Renderer> HoverableContainer<'a, Message, Theme, Renderer> {
    /// Creates a new [HoverableContainer] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        HoverableContainer {
            padding: Padding::ZERO,
            content: content.into(),
            on_hover: None,
            on_unhover: None,
        }
    }

    /// Sets the [`Padding`] of the content.
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the content is hovered.
    pub fn on_hover(mut self, message: Message) -> Self {
        self.on_hover = Some(message);
        self
    }

    /// Sets the message that will be produced when the content is unhovered.
    pub fn on_unhover(mut self, message: Message) -> Self {
        self.on_unhover = Some(message);
        self
    }
}

/// The local state of an [`HoverableContainer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_hovered: bool,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for HoverableContainer<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }
        let state = tree.state.downcast_mut::<State>();
        let was_hovered = state.is_hovered;
        let now_hovered = cursor_position.is_over(layout.bounds());
        match (was_hovered, now_hovered) {
            (true, true) => {}
            (false, false) => {}
            (true, false) => {
                // exited hover
                state.is_hovered = now_hovered;
                if let Some(on_unhover) = &self.on_unhover {
                    shell.publish(on_unhover.clone());
                }
            }
            (false, true) => {
                // entered hover
                state.is_hovered = now_hovered;
                if let Some(on_hover) = &self.on_hover {
                    shell.publish(on_hover.clone());
                }
            }
        }
        event::Status::Ignored
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } = self.size();
        let limits = limits.width(width).height(height).shrink(self.padding);

        let content_layout = self
            .content
            .as_widget()
            .layout(&mut tree.children[0], renderer, &limits)
            .move_to(Point::new(
                self.padding.left.into(),
                self.padding.top.into(),
            ));

        let size = limits
            .resolve(width, height, content_layout.size())
            .expand(self.padding);

        layout::Node::with_children(size, vec![content_layout])
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            content_layout,
            cursor_position,
            &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<HoverableContainer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(
        value: HoverableContainer<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(value)
    }
}
