//! Allow widgets to emit messages when hovered
//!
//! A [`HoverableContainer`] is an widget that produces a messages when hovered or unhovered.
//!
//! This widget is greatly inspired by
//!
//!    <https://giesch.dev/iced-hoverable/>
//!
//! see also
//!
//!    <https://docs.rs/iced_widget/0.12.1/src/iced_widget/container.rs.html>

use iced::{
    Element, Length, Padding, Point, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Shell,
        layout::{self, Layout},
        mouse, renderer,
        widget::{self, Widget},
    },
    event, overlay,
};

/// A widget that emits a message when hovered.
pub(super) struct HoverableContainer<'a, Message> {
    padding: Padding,
    content: Element<'a, Message>,
    on_hover: Option<Message>,
    on_unhover: Option<Message>,
}

impl<'a, Message> HoverableContainer<'a, Message> {
    /// Creates a new [HoverableContainer] with the given content.
    pub(super) fn new(content: impl Into<Element<'a, Message>>) -> Self {
        HoverableContainer {
            padding: Padding::ZERO,
            content: content.into(),
            on_hover: None,
            on_unhover: None,
        }
    }

    /// Sets the message that will be produced when the content is hovered.
    #[must_use]
    pub(super) fn on_hover(mut self, message: Message) -> Self {
        self.on_hover = Some(message);
        self
    }

    /// Sets the message that will be produced when the content is unhovered.
    #[must_use]
    pub(super) fn on_unhover(mut self, message: Message) -> Self {
        self.on_unhover = Some(message);
        self
    }
}

/// The local state of an [`HoverableContainer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct HoverableContainerState {
    is_hovered: bool,
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for HoverableContainer<'a, Message>
where
    Message: 'a + Clone,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<HoverableContainerState>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(HoverableContainerState::default())
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
        layout: Layout,
        cursor_position: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
            viewport,
        ) == event::Status::Captured
        {
            return event::Status::Captured;
        }

        let state = tree.state.downcast_mut::<HoverableContainerState>();
        let was_hovered = state.is_hovered;
        let now_hovered = cursor_position.is_over(layout.bounds());
        match (was_hovered, now_hovered) {
            (true, true) | (false, false) => {}
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
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } = self.size();
        let limits = limits.width(width).height(height).shrink(self.padding);

        let content_layout = self
            .content
            .as_widget()
            .layout(&mut tree.children[0], renderer, &limits)
            .move_to(Point::new(self.padding.left, self.padding.top));

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
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
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
            cursor,
            &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
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
        layout: Layout,
        renderer: &iced::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, Message> From<HoverableContainer<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: HoverableContainer<'a, Message>) -> Self {
        Element::new(value)
    }
}
