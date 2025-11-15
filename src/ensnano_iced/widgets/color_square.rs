//! A widget to Visualize selected color.

use iced::{
    Color, Length, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Renderer as RendererTrait, Shell, Widget, layout, mouse,
        renderer::Style, widget,
    },
    event,
    mouse::Cursor,
};
use iced_graphics::{
    Primitive,
    color::pack,
    mesh::{Indexed, Mesh, SolidVertex2D},
};
use iced_wgpu::primitive::Custom;
use std::marker::PhantomData;

const DEFAULT_SIZE: f32 = 90.0;

/// The State of a [ColorSquare]
#[derive(Default, Clone, Eq, PartialEq)]
pub struct ColorSquareState {
    clicked: bool,
}

/// A ColorSquare Widget
pub struct ColorSquare<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    width: Length,
    height: Length,
    color: Color,
    on_click: Option<Box<dyn Fn(Color) -> Message + 'a>>,
    on_release: Option<Message>,
    _theme: PhantomData<Theme>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme> ColorSquare<'a, Message, Theme, iced::Renderer> {
    pub fn new(color: Color) -> Self {
        Self {
            width: Length::Fixed(DEFAULT_SIZE),
            height: Length::Fixed(DEFAULT_SIZE),
            color,
            on_click: None,
            on_release: None,
            _theme: Default::default(),
            _renderer: Default::default(),
        }
    }

    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(Color) -> Message,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer>
    for ColorSquare<'_, Message, Theme, iced::Renderer>
where
    Message: Clone,
{
    fn state(&self) -> widget::tree::State {
        widget::tree::State::Some(Box::new(ColorSquareState::default()))
    }
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &Style,
        layout: Layout,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let x_max = b.width;
        let y_max = b.height;
        let color = pack(self.color.into_linear());
        let vertices = vec![
            SolidVertex2D {
                position: [0., 0.],
                color,
            },
            SolidVertex2D {
                position: [0., y_max],
                color,
            },
            SolidVertex2D {
                position: [x_max, 0.],
                color,
            },
            SolidVertex2D {
                position: [x_max, y_max],
                color,
            },
        ];
        let indices = vec![0, 1, 2, 1, 2, 3];

        let mesh = Custom::Mesh(Mesh::Solid {
            buffers: Indexed { vertices, indices },
            size: b.size(),
        });

        match renderer {
            iced::Renderer::Wgpu(wgpu_renderer) => {
                wgpu_renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
                    renderer.draw_primitive(Primitive::Custom(mesh));
                });
            }
            iced::Renderer::TinySkia(_) => unreachable!(),
        }
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout,
        cursor: Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        if let event::Event::Mouse(mouse_event) = event {
            let state = tree.state.downcast_mut::<ColorSquareState>();
            match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if cursor.is_over(layout.bounds()) {
                        state.clicked = true;
                        if let Some(on_click) = &self.on_click {
                            shell.publish(on_click(self.color));
                        }
                        event::Status::Captured
                    } else {
                        event::Status::Ignored
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) if state.clicked => {
                    if cursor.is_over(layout.bounds()) {
                        state.clicked = false;
                        if let Some(on_release) = self.on_release.clone() {
                            shell.publish(on_release);
                        }
                        event::Status::Captured
                    } else {
                        event::Status::Ignored
                    }
                }
                mouse::Event::CursorMoved { .. } if state.clicked => {
                    if cursor.is_over(layout.bounds()) {
                        event::Status::Ignored
                    } else {
                        state.clicked = false;
                        if let Some(on_release) = self.on_release.clone() {
                            shell.publish(on_release);
                        }
                        event::Status::Captured
                    }
                }
                _ => event::Status::Ignored,
            }
        } else {
            event::Status::Ignored
        }
    }
}

impl<'a, Message, Theme> From<ColorSquare<'a, Message, Theme, iced::Renderer>>
    for iced::Element<'a, Message, Theme, iced::Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
{
    fn from(color_square: ColorSquare<'a, Message, Theme, iced::Renderer>) -> Self {
        Self::new(color_square)
    }
}
