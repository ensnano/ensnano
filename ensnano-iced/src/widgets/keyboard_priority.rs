/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
//! Gives text_input widgets priority to handle keyboard event.
use iced::{
    Element, Length, Rectangle, Size, Vector,
    advanced::{
        layout::{self, Layout},
        renderer,
        widget::{self, Widget, operation::Focusable as _},
        {Clipboard, Shell, mouse},
    },
    event, overlay,
    widget::text_input,
};
use iced_graphics::text::Paragraph;

/// A container that should contain a [text_input::TextInput].
///
/// Trigger `on_priority` and `on_unpriority` when the text_input is focused or unfocused.
pub struct KeyboardPriority<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    id: Option<Id>,
    width: Length,
    height: Length,
    content: iced::Element<'a, Message, Theme, Renderer>,
    on_priority: Option<Message>,
    on_unpriority: Option<Message>,
}

/// A container that gives keyboard priority to it's [text_input::TextInput] content.
pub fn keyboard_priority<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> KeyboardPriority<'a, Message> {
    KeyboardPriority::new(content)
}

impl<'a, Message, Theme, Renderer> KeyboardPriority<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    /// Creates a new [`HoverableContainer`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();
        KeyboardPriority {
            id: None,
            width: size.width.fluid(),
            height: size.height.fluid(),
            content,
            on_priority: None,
            on_unpriority: None,
        }
    }

    /// Sets the [`Id`] of the [`KeyboardPriority`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the width of the [`KeyboardPriority`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`KeyboardPriority`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the message that will be produced when the content is hovered.
    pub fn on_priority(mut self, message: Message) -> Self {
        self.on_priority = Some(message);
        self
    }

    /// Sets the message that will be produced when the content is unhovered.
    pub fn on_unpriority(mut self, message: Message) -> Self {
        self.on_unpriority = Some(message);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for KeyboardPriority<'a, Message, Theme, Renderer>
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

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        iced::widget::container::layout(
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
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        operation.container(
            self.id.as_ref().map(|id| &id.0),
            layout.bounds(),
            &mut |operation| {
                self.content.as_widget().operate(
                    &mut tree.children[0],
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            },
        );
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
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
        if let Some(tree) = tree.children.get(0) {
            // TODO: Make the downcast more robust.
            let text_input_state = tree.state.downcast_ref::<text_input::State<Paragraph>>();
            // Send message if the state has changed.
            if text_input_state.is_focused() & !state.is_focused() {
                state.focus();
                if let Some(on_priority) = &self.on_priority {
                    shell.publish(on_priority.clone())
                }
                event::Status::Captured
            } else if !text_input_state.is_focused() & state.is_focused() {
                state.unfocus();
                if let Some(on_unpriority) = &self.on_unpriority {
                    shell.publish(on_unpriority.clone())
                }
                event::Status::Captured
            } else {
                status
            }
        } else {
            status
        }
    }

    // NOTE: Needed to transmit mouse intercation to child [`TextInput`].
    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout,
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

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout,
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

impl<'a, Message, Theme, Renderer> From<KeyboardPriority<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(value: KeyboardPriority<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

/// The identifier of a [`KeyboardPriority`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// The local state of an [`KeyboardPriority`].
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Store the last known state of the underlying [text_input].
    is_focused: bool,
}

impl widget::operation::Focusable for State {
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
