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
//! A Iced Widget to select Hue.
use std::marker::PhantomData;

use iced::{
    advanced::{
        layout, mouse, renderer::Style, widget, Clipboard, Layout, Renderer as _, Shell, Widget,
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
use iced_wgpu;

use color_space::{Hsv, Rgb};

const DEFAULT_SIZE: f32 = 90.0;

/// The internal state of a [HueColumn].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_dragging: bool,
}

/// A HueColumn Widget.
pub struct HueColumn<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    width: Length,
    height: Length,
    on_slide: Option<Box<dyn Fn(f64) -> Message + 'a>>,
    _theme: PhantomData<Theme>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme> HueColumn<'a, Message, Theme, crate::Renderer> {
    pub fn new() -> Self {
        Self {
            width: Length::Fixed(DEFAULT_SIZE),
            height: Length::Fixed(4.0 * DEFAULT_SIZE),
            on_slide: None,
            _theme: Default::default(),
            _renderer: Default::default(),
        }
    }

    pub fn on_slide<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(f64) -> Message,
    {
        self.on_slide = Some(Box::new(f));
        self
    }

    pub fn on_slide_maybe<F>(mut self, f: Option<F>) -> Self
    where
        F: 'a + Fn(f64) -> Message,
    {
        self.on_slide = f.map(|f| Box::new(f) as _);
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

impl<'a, Message, Theme> Widget<Message, Theme, crate::Renderer>
    for HueColumn<'a, Message, Theme, crate::Renderer>
{
    fn state(&self) -> widget::tree::State {
        widget::tree::State::Some(Box::new(State::default()))
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
        _renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut crate::Renderer,
        _theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let b = layout.bounds();

        let x_max = b.width;
        let y_max = b.height;

        let nb_row = u32::min(100, y_max.ceil() as u32);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for i in 0..=nb_row {
            let hsv = Hsv::new((360 * i) as f64 / nb_row as f64, 1., 1.);
            let Rgb { r, g, b } = Rgb::from(hsv);
            let color = pack([r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.]);

            vertices.push(SolidVertex2D {
                position: [0., y_max * (i as f32 / nb_row as f32)],
                color,
            });
            vertices.push(SolidVertex2D {
                position: [x_max, y_max * (i as f32 / nb_row as f32)],
                color,
            });
            if i > 0 {
                indices.push(2 * i - 2);
                indices.push(2 * i + 1);
                indices.push(2 * i);
                indices.push(2 * i - 2);
                indices.push(2 * i + 1);
                indices.push(2 * i - 1);
            }
        }

        let mesh = iced_wgpu::primitive::Custom::Mesh(Mesh::Solid {
            buffers: Indexed { vertices, indices },
            size: b.size(),
        });

        match renderer {
            crate::Renderer::Wgpu(wgpu_renderer) => wgpu_renderer
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
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &crate::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        // A closure that takes an absolute position and send Message.
        let mut change = |Point { x: _, y }| {
            let bounds = layout.bounds();
            if y <= bounds.y {
                if let Some(on_slide) = &self.on_slide {
                    shell.publish(on_slide(0.));
                }
            } else if y >= bounds.y + bounds.height {
                if let Some(on_slide) = &self.on_slide {
                    shell.publish(on_slide(360.));
                }
            } else {
                if let Some(on_slide) = &self.on_slide {
                    let percent = (y - bounds.y) / bounds.height;
                    let value: f32 = percent * 360.;
                    shell.publish(on_slide(value.into()));
                }
            }
        };

        if let event::Event::Mouse(mouse_event) = event {
            let state = tree.state.downcast_mut::<State>();
            let position = cursor.position_over(layout.bounds());
            match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(pos) = position {
                        change(pos);
                        state.is_dragging = true;
                    }
                    event::Status::Captured
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.is_dragging {
                        state.is_dragging = false;
                    }
                    event::Status::Captured
                }
                mouse::Event::CursorMoved { .. } => {
                    // NOTE: Using "position" attribute from mouse::Event::CursorMoved dosen't work because
                    //       it is not the good coordinates.
                    if state.is_dragging {
                        if let Some(pos) = position {
                            change(pos);
                        }
                        event::Status::Captured
                    } else {
                        event::Status::Ignored
                    }
                }
                _ => event::Status::Ignored,
            }
        } else {
            // Not a mouse event.
            event::Status::Ignored
        }
    }
}

impl<'a, Message, Theme> From<HueColumn<'a, Message, Theme, crate::Renderer>>
    for crate::Element<'a, Message, Theme, crate::Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
{
    fn from(hue_column: HueColumn<'a, Message, Theme, crate::Renderer>) -> Self {
        Self::new(hue_column)
    }
}
