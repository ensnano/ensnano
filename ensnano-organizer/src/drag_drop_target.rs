//! Allow your users to drag and drop widgets.
use iced::{overlay, Padding};
use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::renderer;
use iced_native::widget::tree::{self, Tree};
use iced_native::{Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Widget};

use super::OrganizerMessage;

/// Identifier for drag-drop widgets.
#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub enum DragIdentifier<K, AutoGroup> {
    Group { id: super::NodeId<AutoGroup> },
    Section { key: K },
}

/// An widget that can be dragged.
pub struct DragDropTarget<'a, Message, Renderer, K, E> {
    padding: Padding,
    content: Element<'a, Message, Renderer>,
    identifier: DragIdentifier<K, E>,
}

impl<'a, Message, Renderer, K, E> DragDropTarget<'a, Message, Renderer, K, E> {
    const WIDTH: Length = Length::Shrink;
    const HEIGHT: Length = Length::Shrink;

    /// Creates a new [`DragDropTarget`] with the given content and identifier.
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        identifier: DragIdentifier<K, E>,
    ) -> Self {
        Self {
            padding: Padding::ZERO,
            content: content.into(),
            identifier,
        }
    }

    /// Sets the [`Padding`] of the content.
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }
}

impl<'a, E, Renderer> Widget<OrganizerMessage<E>, Renderer>
    for DragDropTarget<'a, OrganizerMessage<E>, Renderer, E::Key, E::AutoGroup>
where
    E: super::OrganizerElement,
    Renderer: iced_native::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn width(&self) -> Length {
        Self::WIDTH
    }

    fn height(&self) -> Length {
        Self::HEIGHT
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits
            .width(Self::WIDTH)
            .height(Self::HEIGHT)
            .pad(self.padding);

        let mut content_layout = self.content.as_widget().layout(renderer, &limits);
        content_layout.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        let size = limits.resolve(content_layout.size()).pad(self.padding);

        layout::Node::with_children(size, vec![content_layout])
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<OrganizerMessage<E>>,
    ) -> event::Status {
        use iced::mouse;
        use iced::mouse::Event as MouseEvent;
        let status = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        );
        match event {
            Event::Mouse(MouseEvent::ButtonReleased(mouse::Button::Left)) => {
                if layout.bounds().contains(cursor_position) {
                    shell.publish(OrganizerMessage::drag_dropped(self.identifier.clone()))
                }
            }
            Event::Mouse(MouseEvent::ButtonPressed(mouse::Button::Left)) => {
                if layout.bounds().contains(cursor_position) {
                    shell.publish(OrganizerMessage::dragging(self.identifier.clone()))
                }
                return event::Status::Captured;
            }
            _ => (),
        };
        status
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, OrganizerMessage<E>, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, E, Renderer> From<DragDropTarget<'a, OrganizerMessage<E>, Renderer, E::Key, E::AutoGroup>>
    for Element<'a, OrganizerMessage<E>, Renderer>
where
    E: super::OrganizerElement,
    Renderer: 'a + iced_native::Renderer,
{
    fn from(
        value: DragDropTarget<'a, OrganizerMessage<E>, Renderer, E::Key, E::AutoGroup>,
    ) -> Element<'a, OrganizerMessage<E>, Renderer> {
        Element::new(value)
    }
}
