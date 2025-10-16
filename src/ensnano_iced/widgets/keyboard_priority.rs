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
    Element, Length, Padding, Point, Rectangle, Size, Vector,
    advanced::{
        layout::{self, Layout},
        renderer,
        widget::{self, Widget},
        {Clipboard, Shell, mouse},
    },
    event, keyboard, overlay,
    widget::text_input,
};
use iced_graphics::text::Paragraph;

/// A container that should contain a [text_input::TextInput].
///
/// Trigger `on_priority` and `on_unpriority` when the text_input is focused or unfocused.
pub struct KeyboardPriority<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    padding: Padding,
    width: Length,
    height: Length,
    content: iced::Element<'a, Message, Theme, Renderer>,
    on_priority: Option<Message>,
    on_unpriority: Option<Message>,
}

/// A container that gives keyboard priority to it's [text_input::TextInput] content.
pub fn keyboard_priority<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> KeyboardPriority<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
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
            padding: Padding::ZERO,
            width: size.width.fluid(),
            height: size.height.fluid(),
            content,
            on_priority: None,
            on_unpriority: None,
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

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

/// The local state of an [`HoverableContainer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_focused: bool,
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

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout,
        cursor_position: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        // Here are the three actions we can do.
        enum Action {
            Activate,
            Deactivate,
            None,
        }
        // First, we get the current focus state.
        let state = tree.state.downcast_mut::<State>();

        // Figure out whether the underlying widget is a [`text_input`].
        let is_child_a_text_input = if let Some(child_widget) = tree.children.get(0)
            && let widget::tree::State::Some(child_state) = &child_widget.state
        {
            child_state.downcast_ref::<text_input::State<Paragraph>>()
        } else {
            None
        };

        let action = match is_child_a_text_input {
            Some(text_input_state) => {
                let was_focused = state.is_focused;
                // Figure out whether the underlying widget is focused.
                let now_focused = text_input_state.is_focused();
                // Update state
                state.is_focused = now_focused;
                // We also need to intercept if the key Enter has been hit.
                let enter_key_hit = match &event {
                    event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                        match key.as_ref() {
                            keyboard::Key::Named(keyboard::key::Named::Enter) => true,
                            _ => false,
                        }
                    }
                    _ => false,
                };
                if enter_key_hit {
                    // I.e, user requested to stop text edition.
                    Action::Deactivate
                } else {
                    if was_focused == now_focused {
                        // Situation has not changed, do nothing.
                        Action::None
                    } else {
                        if now_focused {
                            Action::Activate
                        } else {
                            Action::Deactivate
                        }
                    }
                }
            }
            None => {
                // If the child is not a [`text_input`] ensure keyboard_priority is off and stop
                if state.is_focused {
                    state.is_focused = false;
                    Action::Deactivate
                } else {
                    Action::None
                }
            }
        };

        // Act.
        match action {
            Action::Activate => {
                if let Some(on_hover) = &self.on_priority {
                    shell.publish(on_hover.clone());
                }
            }
            Action::Deactivate => {
                if let Some(on_unhover) = &self.on_unpriority {
                    shell.publish(on_unhover.clone());
                }
            }
            Action::None => {}
        }

        // Finally process the event of child.
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // container::layout(
        //     limits,
        //     self.width,
        //     self.height,
        //     f32::INFINITY,
        //     f32::INFINITY,
        //     self.padding,
        //     alignment::Horizontal::Left,
        //     alignment::Vertical::Top,
        //     |limits| self.content.as_widget().layout(tree, renderer, limits),
        // )
        // NOTE: I tried to use the layout defined by container. I will try again later to make it
        // work.
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
            width: self.width,
            height: self.height,
        }
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
    fn from(
        value: KeyboardPriority<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(value)
    }
}
