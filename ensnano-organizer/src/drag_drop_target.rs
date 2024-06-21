//! Allow your users to drag and drop widgets.
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::advanced::{mouse, Clipboard, Shell};
use iced::{event, overlay, Element, Length, Padding, Point, Rectangle, Size, Vector};

use super::OrganizerMessage;

/// Identifier for drag-drop widgets.
#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub enum DragIdentifier<K, AutoGroup> {
    Group { id: super::NodeId<AutoGroup> },
    Section { key: K },
}

/// An widget that can be dragged.
pub struct DragDropTarget<'a, Message, Theme, Renderer, K, E> {
    padding: Padding,
    content: Element<'a, Message, Theme, Renderer>,
    identifier: DragIdentifier<K, E>,
}

impl<'a, Message, Theme, Renderer, K, E> DragDropTarget<'a, Message, Theme, Renderer, K, E> {
    /// Creates a new [`DragDropTarget`] with the given content and identifier.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
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

impl<'a, E, Theme, Renderer> Widget<OrganizerMessage<E>, Theme, Renderer>
    for DragDropTarget<'a, OrganizerMessage<E>, Theme, Renderer, E::Key, E::AutoGroup>
where
    E: super::OrganizerElement,
    Renderer: renderer::Renderer,
{
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
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
            .layout(tree, renderer, &limits)
            .move_to(Point::new(
                self.padding.left.into(),
                self.padding.top.into(),
            ));

        let size = limits
            .resolve(width, height, content_layout.size())
            .expand(self.padding);

        layout::Node::with_children(size, vec![content_layout])
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<OrganizerMessage<E>>,
        viewport: &Rectangle,
    ) -> event::Status {
        let status = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        match event {
            event::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if cursor_position.is_over(layout.bounds()) {
                    shell.publish(OrganizerMessage::drag_dropped(self.identifier.clone()))
                }
            }
            event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor_position.is_over(layout.bounds()) {
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
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor_position: mouse::Cursor,
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
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'_, OrganizerMessage<E>, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a, E, Theme, Renderer>
    From<DragDropTarget<'a, OrganizerMessage<E>, Theme, Renderer, E::Key, E::AutoGroup>>
    for Element<'a, OrganizerMessage<E>, Theme, Renderer>
where
    E: super::OrganizerElement,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(
        value: DragDropTarget<'a, OrganizerMessage<E>, Theme, Renderer, E::Key, E::AutoGroup>,
    ) -> Element<'a, OrganizerMessage<E>, Theme, Renderer> {
        Element::new(value)
    }
}
