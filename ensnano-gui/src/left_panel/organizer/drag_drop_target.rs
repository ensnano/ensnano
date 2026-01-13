//! Allow your users to drag and drop widgets.

use super::{OrganizerNodeId, message::OrganizerMessage};
use ensnano_design::elements::DesignElementKey;
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
pub enum DragIdentifier {
    Group { id: OrganizerNodeId },
    Section { key: DesignElementKey },
}

/// An widget that can be dragged.
///
/// There is no [Padding], [Size] for this widget. It sticks around its content.
pub struct DragDropTarget<'a, Message> {
    content: Element<'a, Message>,
    identifier: DragIdentifier,
}

impl<'a, Message> DragDropTarget<'a, Message> {
    /// Creates a new [`DragDropTarget`] with the given content and identifier.
    pub fn new(content: impl Into<Element<'a, Message>>, identifier: DragIdentifier) -> Self {
        Self {
            content: content.into(),
            identifier,
        }
    }
}

impl Widget<OrganizerMessage, iced::Theme, iced::Renderer>
    for DragDropTarget<'_, OrganizerMessage>
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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<OrganizerMessage>,
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
        }
        status
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            tree,
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout,
        renderer: &iced::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, OrganizerMessage, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout.children().next().unwrap(),
            renderer,
            translation,
        )
    }
}

impl<'a> From<DragDropTarget<'a, OrganizerMessage>> for Element<'a, OrganizerMessage> {
    fn from(value: DragDropTarget<'a, OrganizerMessage>) -> Self {
        Element::new(value)
    }
}
