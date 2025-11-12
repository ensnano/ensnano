//! A widget to select Lightness and Saturation values.
use std::marker::PhantomData;

use iced::{
    Length, Point, Rectangle, Size, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer::Style, widget,
    },
    event,
    mouse::Cursor,
};
use iced_graphics::{
    Primitive,
    color::pack,
    mesh::{Indexed, Mesh, SolidVertex2D},
};
use iced_wgpu as wgpu;

use color_space::{Hsv, Rgb};

const DEFAULT_SIZE: f32 = 360.0;

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
pub struct LightSatSquare<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    width: Length,
    height: Length,
    hue: f64,
    on_slide: Option<Box<dyn Fn(f64, f64) -> Message + 'a>>,
    on_finish: Option<Message>,
    _theme: PhantomData<Theme>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme> LightSatSquare<'a, Message, Theme, iced::Renderer> {
    pub fn new(hue: f64) -> Self {
        Self {
            width: Length::Fixed(DEFAULT_SIZE),
            height: Length::Fixed(DEFAULT_SIZE),
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

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, Message, Theme> Widget<Message, Theme, iced::Renderer>
    for LightSatSquare<'a, Message, Theme, iced::Renderer>
where
    Message: Clone,
{
    fn state(&self) -> widget::tree::State {
        widget::tree::State::Some(Box::new(LightSatState::default()))
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
        _state: &widget::Tree,
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

        let nb_row = 100;
        let nb_column = 100;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for i in 0..=nb_row {
            let value = 1. - (i as f64 / nb_row as f64);
            for j in 0..=nb_column {
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

        match renderer {
            iced::Renderer::Wgpu(wgpu_renderer) => wgpu_renderer
                .with_translation(Vector::new(b.x, b.y), |renderer| {
                    renderer.draw_primitive(Primitive::Custom(mesh))
                }),
            _ => panic!("Unhandled renderer"),
        };
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
        // A closure that takes an absolute position and send Message.
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
            let position = cursor.position_over(layout.bounds());
            match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(pos) = position {
                        change(pos);
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
                mouse::Event::CursorMoved { .. } => {
                    // NOTE: Using "position" attribute from mouse::Event::CursorMoved doesn't work because
                    //       it is not the good coordinates.
                    if state.is_dragging
                        && let Some(pos) = position
                    {
                        change(pos);
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

impl<'a, Message, Theme> From<LightSatSquare<'a, Message, Theme, iced::Renderer>>
    for iced::Element<'a, Message, Theme, iced::Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
{
    fn from(value: LightSatSquare<'a, Message, Theme, iced::Renderer>) -> Self {
        Self::new(value)
    }
}
