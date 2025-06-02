//! A widget to select Lightness and Saturation values.
use std::marker::PhantomData;

use iced::{
    advanced::{
        layout, mouse, renderer::Style, widget, Clipboard, Layout, Renderer as RendererTrait,
        Shell, Widget,
    },
    event,
    mouse::Cursor,
    Length, Point, Rectangle, Size, Vector,
};
use iced_graphics::{
    color::pack,
    mesh::{Indexed, Mesh, SolidVertex2D},
    Primitive,
};
use iced_wgpu as wgpu;

use color_space::{Hsv, Rgb};

/// The internal state of a [LightSatSquare].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LightSatState {
    is_dragging: bool,
}

fn hsv_to_linear(hue: f64, sat: f64, light: f64) -> [f32; 4] {
    let hsv = Hsv::new(hue, sat, light);
    let rgb = Rgb::from(hsv);
    [
        rgb.r as f32 / 255.,
        rgb.g as f32 / 255.,
        rgb.b as f32 / 255.,
        1.,
    ]
}

/// A Lightness-Saturation square Widget.
pub struct LightSatSquare<'a, Message, Theme = crate::Theme, Renderer = iced_wgpu::Renderer> {
    hue: f64,
    on_slide: Option<Box<dyn Fn(f64, f64) -> Message + 'a>>,
    on_finish: Option<Message>,
    _theme: PhantomData<Theme>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme> LightSatSquare<'a, Message, Theme, iced_wgpu::Renderer> {
    pub fn new(hue: f64) -> Self {
        Self {
            hue,
            on_slide: None,
            on_finish: None,
            _theme: Default::default(),
            _renderer: Default::default(),
        }
    }

    pub fn on_slide<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(f64, f64) -> Message,
    {
        self.on_slide = Some(Box::new(f));
        self
    }

    pub fn on_finish(mut self, message: Message) -> Self {
        self.on_finish = Some(message);
        self
    }
}

impl<'a, Message, Theme> Widget<Message, Theme, iced_wgpu::Renderer>
    for LightSatSquare<'a, Message, Theme, iced_wgpu::Renderer>
where
    Message: Clone,
{
    fn state(&self) -> widget::tree::State {
        widget::tree::State::Some(Box::new(LightSatState::default()))
    }
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::FillPortion(4),
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &iced_wgpu::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);

        layout::Node::new(Size::new(size.width, size.width))
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut iced_wgpu::Renderer,
        _theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();

        let x_max = b.width;
        let y_max = b.height;

        let nb_row = 100;
        let nb_column = 100;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for i in 0..nb_row {
            let value = 1. - (i as f64 / nb_row as f64);
            for j in 0..nb_column {
                let sat = 1. - (j as f64 / nb_column as f64);
                let color = pack(hsv_to_linear(self.hue, sat, value));
                vertices.push(SolidVertex2D {
                    position: [
                        x_max * (j as f32 / nb_column as f32),
                        y_max * (i as f32 / nb_row as f32),
                    ],
                    color,
                });
                if i > 0 && j > 0 {
                    indices.push(nb_row * (i - 1) + j - 1);
                    indices.push(nb_row * i + j);
                    indices.push(nb_row * i + j - 1);
                    indices.push(nb_row * (i - 1) + j - 1);
                    indices.push(nb_row * i + j);
                    indices.push(nb_row * (i - 1) + j);
                }
            }
        }

        let mesh = wgpu::primitive::Custom::Mesh(Mesh::Solid {
            size: b.size(),
            buffers: Indexed { vertices, indices },
        });

        renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
            renderer.draw_primitive(Primitive::Custom(mesh))
        });
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: event::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &iced_wgpu::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let mut change = |Point { x, y }| {
            let bounds = layout.bounds();
            let percent_x = if x <= bounds.x {
                0.
            } else if x >= bounds.x + bounds.width {
                1.
            } else {
                f64::from(x - bounds.x) / f64::from(bounds.width)
            };

            let percent_y = if y <= bounds.y {
                0.
            } else if y >= bounds.y + bounds.height {
                1.
            } else {
                f64::from(y - bounds.y) / f64::from(bounds.height)
            };

            let saturation = 1. - percent_x;
            let value = 1. - percent_y;
            if let Some(on_slide) = &self.on_slide {
                shell.publish(on_slide(saturation, value));
            }
        };

        if let event::Event::Mouse(mouse_event) = event {
            let state = tree.state.downcast_mut::<LightSatState>();
            match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(position) = cursor.position() {
                        change(position);
                        state.is_dragging = true;
                        event::Status::Captured
                    } else {
                        event::Status::Ignored
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.is_dragging {
                        state.is_dragging = false;
                    }
                    if let Some(on_finish) = self.on_finish.clone() {
                        shell.publish(on_finish);
                    }
                    event::Status::Captured
                }
                mouse::Event::CursorMoved { position } => {
                    if state.is_dragging {
                        change(position);
                    }
                    event::Status::Captured
                }
                _ => event::Status::Ignored,
            }
        } else {
            event::Status::Ignored
        }
    }
}

impl<'a, Message, Theme> From<LightSatSquare<'a, Message, Theme, iced_wgpu::Renderer>>
    for crate::Element<'a, Message, Theme, iced_wgpu::Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
{
    fn from(value: LightSatSquare<'a, Message, Theme, iced_wgpu::Renderer>) -> Self {
        Self::new(value)
    }
}
