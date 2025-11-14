//! Allow your users to drag and drop widgets.

use super::OrganizerMessage;
use iced::{
    Element, Length, Padding, Rectangle, Size, Vector,
    advanced::{
        layout::{self, Layout},
        renderer,
        widget::{self, Widget},
        {Clipboard, Shell, mouse},
    },
    alignment, event, overlay,
    widget::container,
};

/// Identifier for drag-drop widgets.
#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub enum DragIdentifier<K, AutoGroup> {
    Group { id: super::NodeId<AutoGroup> },
    Section { key: K },
}

/// An widget that can be dragged.
///
/// There is no [Padding], [Size] for this widget. It sticks around its content.
pub struct DragDropTarget<'a, Message, Theme, Renderer, K, E> {
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
            content: content.into(),
            identifier,
        }
    }
}

impl<E, Theme, Renderer> Widget<OrganizerMessage<E>, Theme, Renderer>
    for DragDropTarget<'_, OrganizerMessage<E>, Theme, Renderer, E::Key, E::AutoGroup>
where
    E: super::OrganizerElement,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut widget::Tree) {
        self.content.as_widget().diff(tree);
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
        container::layout(
            limits,
            Length::Shrink,
            Length::Shrink,
            f32::INFINITY,
            f32::INFINITY,
            Padding::ZERO,
            alignment::Horizontal::Left,
            alignment::Vertical::Top,
            |limits| self.content.as_widget().layout(tree, renderer, limits),
        )
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<OrganizerMessage<E>>,
        viewport: &Rectangle,
    ) -> event::Status {
        let status = self.content.as_widget_mut().on_event(
            tree,
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
                    shell.publish(OrganizerMessage::drag_dropped(self.identifier.clone()));
                }
            }
            event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor_position.is_over(layout.bounds()) {
                    shell.publish(OrganizerMessage::dragging(self.identifier.clone()));
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
        layout: layout::Layout,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            tree,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, OrganizerMessage<E>, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
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
    ) -> Self {
        Element::new(value)
    }
}
