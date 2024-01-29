//! Allow widgets to emit messages when hovered
//!
//! A [`HoverableContainer`] is an widget that produces a messages when hovered or unhovered.
//!
//! This widget is greatly inspired by
//!
//!    https://giesch.dev/iced-hoverable/

use iced::{overlay, Padding};
use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::renderer;
use iced_native::widget::tree::{self, Tree};
use iced_native::{Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Widget};

/// A widget that emits a message when hovered.
pub struct HoverableContainer<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    padding: Padding,
    content: Element<'a, Message, Renderer>,
    on_hover: Option<Message>,
    on_unhover: Option<Message>,
}

impl<'a, Message, Renderer> HoverableContainer<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    const WIDTH: Length = Length::Shrink;
    const HEIGHT: Length = Length::Shrink;

    /// Creates a new [HoverableContainer] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
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

impl<'a, Message, Renderer> Widget<Message, Renderer> for HoverableContainer<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: iced_native::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }
        let mut state = tree.state.downcast_mut::<State>();
        let was_hovered = state.is_hovered;
        let now_hovered = layout.bounds().contains(cursor_position);
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

        //if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
        //    let bounds = layout.bounds();
        //    if bounds.contains(cursor_position) {
        //        if !state.is_hovered {
        //            if let Some(on_hovered_in) = self.on_hovered_in.clone() {
        //                shell.publish(on_hovered_in)
        //            }
        //            state.is_hovered = true;
        //        }
        //    } else {
        //        if state.is_hovered {
        //            if let Some(on_hovered_out) = self.on_hovered_out.clone() {
        //                shell.publish(on_hovered_out)
        //            }
        //            state.is_hovered = false;
        //        }
        //    }
        //}
        event::Status::Ignored
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

    fn width(&self) -> Length {
        Self::WIDTH
    }

    fn height(&self) -> Length {
        Self::HEIGHT
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        self.content.as_widget().draw(
            &state.children[0],
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
        state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> iced_native::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout.children().next().unwrap(),
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<HoverableContainer<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_native::Renderer,
{
    fn from(value: HoverableContainer<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(value)
    }
}
