use iced::Element;
use iced_native::{
    event, layout, overlay, renderer::Style, widget, Alignment, Clipboard, Event, Layout, Length,
    Point, Rectangle, Shell, Widget,
};

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Ord)]
pub(super) enum Identifier<K, AutoGroup> {
    Group { id: super::NodeId<AutoGroup> },
    Section { key: K },
}

pub(super) struct DragDropTarget<'a, Message, K, E> {
    padding: u16,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    horizontal_alignment: Alignment,
    vertical_alignment: Alignment,
    content: Element<'a, Message>,
    identifier: Identifier<K, E>,
}

impl<'a, Message, K, E> DragDropTarget<'a, Message, K, E> {
    /// Creates an empty [`DragDropTarget`].
    pub fn new(content: impl Into<Element<'a, Message>>, identifier: Identifier<K, E>) -> Self {
        Self {
            padding: 0,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: f32::MAX,
            max_height: f32::MAX,
            horizontal_alignment: Alignment::Start,
            vertical_alignment: Alignment::Start,
            content: content.into(),
            identifier,
        }
    }
}

use super::OrganizerMessage;
use iced::Theme;
use iced_graphics::Renderer;
use iced_wgpu::Backend;

impl<'a, E: super::OrganizerElement> Widget<OrganizerMessage<E>, Renderer<Backend, Theme>>
    for DragDropTarget<'a, OrganizerMessage<E>, E::Key, E::AutoGroup>
{
    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::tree::Tree::new(&self.content)]
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer<Backend, Theme>, limits: &layout::Limits) -> layout::Node {
        let padding = iced_native::Padding::from(self.padding);

        let limits = limits
            .loose()
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let mut content = self.content.as_widget().layout(renderer, &limits.loose());
        let size = limits.resolve(content.size());

        content.move_to(Point::new(self.padding as f32, self.padding as f32));
        content.align(self.horizontal_alignment, self.vertical_alignment, size);

        layout::Node::with_children(size.pad(padding), vec![content])
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer<Backend, Theme>,
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
        tree: &widget::Tree,
        renderer: &mut Renderer<Backend, Theme>,
        theme: &Theme,
        style: &Style,
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
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer<Backend, Theme>,
    ) -> Option<overlay::Element<'_, OrganizerMessage<E>, Renderer<Backend, Theme>>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, E: super::OrganizerElement>
    From<DragDropTarget<'a, OrganizerMessage<E>, E::Key, E::AutoGroup>>
    for Element<'a, OrganizerMessage<E>, Renderer<Backend, Theme>>
{
    fn from(
        value: DragDropTarget<'a, OrganizerMessage<E>, E::Key, E::AutoGroup>,
    ) -> Element<'a, OrganizerMessage<E>, Renderer<Backend, Theme>> {
        Element::new(value)
    }
}
