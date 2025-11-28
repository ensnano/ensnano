//! Gives text_input widgets priority to handle keyboard event.

use iced::{
    Element, Length, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Shell,
        layout::{self, Layout},
        mouse, renderer,
        widget::{self, Widget, operation::Focusable},
    },
    event, overlay,
    widget::{container, text_input},
};
use iced_graphics::text::Paragraph;
use std::borrow::Cow;

/// This is sent through messages to indicate
/// what keyboard priority widget is taking or giving
/// the priority. Being specific about the id allows
/// to prevent issues with race conditions in the
/// order of events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityRequest {
    pub id: KeyboardPriorityId,
    pub taking: bool,
}

/// A container that should contain a [text_input::TextInput].
///
/// Trigger `on_priority` and `on_unpriority` when the text_input is focused or unfocused.
pub struct KeyboardPriority<'a, Message> {
    id: KeyboardPriorityId,
    width: Length,
    height: Length,
    content: Element<'a, Message>,
    on_priority: Message,
    on_unpriority: Message,
}

/// A container that gives keyboard priority to it's [text_input::TextInput] content.
pub fn keyboard_priority<'a, Message>(
    id: impl Into<Cow<'static, str>>,
    message: impl Fn(PriorityRequest) -> Message,
    content: impl Into<Element<'a, Message>>,
) -> KeyboardPriority<'a, Message> {
    KeyboardPriority::new(id, message, content)
}

impl<'a, Message> KeyboardPriority<'a, Message> {
    /// Creates a new [`KeyboardPriority`] with the given content.
    pub fn new(
        id: impl Into<Cow<'static, str>>,
        message: impl Fn(PriorityRequest) -> Message,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        let id = KeyboardPriorityId::new(id);
        let content = content.into();
        let size = content.as_widget().size_hint();
        KeyboardPriority {
            id: id.clone(),
            width: size.width.fluid(),
            height: size.height.fluid(),
            content,
            on_priority: message(PriorityRequest {
                id: id.clone(),
                taking: true,
            }),
            on_unpriority: message(PriorityRequest { id, taking: false }),
        }
    }

    /// Sets the width of the [`KeyboardPriority`].
    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`KeyboardPriority`].
    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for KeyboardPriority<'a, Message>
where
    Message: 'a + Clone,
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

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
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
            self.width,
            self.height,
            f32::INFINITY,
            f32::INFINITY,
            iced::Padding::ZERO,
            iced::alignment::Horizontal::Left,
            iced::alignment::Vertical::Top,
            |limits| {
                self.content
                    .as_widget()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        operation.container(Some(&self.id.0), layout.bounds(), &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        // First, update the child [`TextInput`].
        let status = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        // Now, update self.
        let state = tree.state.downcast_mut::<State>();
        // Look if the underlying [`TextInput`] is focused.
        if let Some(tree) = tree.children.first() {
            // TODO: Make the downcast more robust.
            let text_input_state = tree.state.downcast_ref::<text_input::State<Paragraph>>();
            // Send message if the state has changed.
            if text_input_state.is_focused() & !state.is_focused() {
                state.focus();
                shell.publish(self.on_priority.clone());
                event::Status::Captured
            } else if !text_input_state.is_focused() & state.is_focused() {
                state.unfocus();
                shell.publish(self.on_unpriority.clone());
                event::Status::Captured
            } else {
                status
            }
        } else {
            status
        }
    }

    // NOTE: Needed to transmit mouse interaction to child [`TextInput`].
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

impl<'a, Message> From<KeyboardPriority<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: KeyboardPriority<'a, Message>) -> Self {
        Self::new(value)
    }
}

/// The identifier of a [`KeyboardPriority`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardPriorityId(widget::Id);

impl KeyboardPriorityId {
    /// Creates a custom [`KeyboardPriorityId`].
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`KeyboardPriorityId`].
    ///
    /// This function produces a different [`KeyboardPriorityId`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<KeyboardPriorityId> for widget::Id {
    fn from(id: KeyboardPriorityId) -> Self {
        id.0
    }
}

/// The local state of an [`KeyboardPriority`].
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Store the last known state of the underlying [`TextInput`](text_input::TextInput).
    is_focused: bool,
}

impl Focusable for State {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }
}
