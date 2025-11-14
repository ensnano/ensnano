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

use super::ColorMessage;
use crate::ensnano_iced::helpers::*;
use hue_column::HueColumn;
use iced::Color;
use light_sat_square::LightSatSquare;

pub struct ColorPicker {
    hue: f64,
}

impl ColorPicker {
    pub fn new() -> Self {
        Self { hue: 0. }
    }

    pub fn change_hue(&mut self, hue: f64) {
        self.hue = hue;
    }

    pub fn new_view(&self) -> crate::ensnano_iced::Element<'_, ColorMessage> {
        row![
            HueColumn::new(ColorMessage::HueChanged,),
            LightSatSquare::new(
                self.hue,
                ColorMessage::HsvSatValueChanged,
                ColorMessage::FinishChangingColor,
            ),
        ]
        .spacing(10)
        .into()
    }
}

/// A Iced Widget to select Hue.
mod hue_column {
    use color_space::{Hsv, Rgb};
    use iced::{
        Length, Point, Rectangle, Renderer, Size, Vector,
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

    /// The internal state of a [HueColumnState].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct HueColumnState {
        is_dragging: bool,
    }

    /// A HueColumn Widget.
    pub struct HueColumn<'a, Message> {
        on_slide: Box<dyn Fn(f64) -> Message + 'a>,
        // TODO: Mimic iced like in Checkbox: Option<Box<…>>, then a method to set something else
        //       than None.
    }

    impl<Message> HueColumn<'_, Message> {
        pub fn new<F>(on_slide: F) -> Self
        where
            F: 'static + Fn(f64) -> Message,
        {
            Self {
                on_slide: Box::new(on_slide),
            }
        }
    }

    impl<Message> Widget<Message, crate::ensnano_iced::Theme, iced::Renderer>
        for HueColumn<'_, Message>
    {
        fn state(&self) -> widget::tree::State {
            widget::tree::State::Some(Box::new(HueColumnState::default()))
        }
        fn size(&self) -> Size<Length> {
            Size {
                width: Length::FillPortion(1),
                height: Length::Shrink,
            }
        }

        fn layout(
            &self,
            _tree: &mut widget::Tree,
            _renderer: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);

            layout::Node::new(Size::new(size.width, 4. * size.width))
        }

        fn draw(
            &self,
            _tree: &widget::Tree,
            renderer: &mut iced::Renderer,
            _theme: &crate::ensnano_iced::Theme,
            _style: &Style,
            layout: Layout,
            _cursor: Cursor,
            _viewport: &Rectangle,
        ) {
            let b = layout.bounds();

            let x_max = b.width;
            let y_max = b.height;

            let nb_row = 10;

            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            for i in 0..=nb_row {
                let hsv = Hsv::new(i as f64 / nb_row as f64 * 360., 1., 1.);
                let rgb = Rgb::from(hsv);
                let color = pack([
                    rgb.r as f32 / 255.,
                    rgb.g as f32 / 255.,
                    rgb.b as f32 / 255.,
                    1.,
                ]);
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

            let mesh = Custom::Mesh(Mesh::Solid {
                buffers: Indexed { vertices, indices },
                size: b.size(),
            });

            match renderer {
                Renderer::Wgpu(wgpu_renderer) => {
                    wgpu_renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
                        renderer.draw_primitive(Primitive::Custom(mesh));
                    });
                }
                _ => panic!("Unhandled renderer"),
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
            let mut change = |Point { x: _, y }| {
                let bounds = layout.bounds();
                if y <= bounds.y {
                    shell.publish((self.on_slide)(0.));
                } else if y >= bounds.y + bounds.height {
                    shell.publish((self.on_slide)(360.));
                } else {
                    let percent = (y - bounds.y) / bounds.height;
                    let value: f32 = percent * 360.;
                    shell.publish((self.on_slide)(value.into()));
                }
            };

            if let event::Event::Mouse(mouse_event) = event {
                let state = tree.state.downcast_mut::<HueColumnState>();
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if let Some(position) = cursor.position() {
                            change(position);
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
                    mouse::Event::CursorMoved { position } => {
                        if state.is_dragging {
                            change(position);
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

    impl<'a, Message> From<HueColumn<'a, Message>>
        for crate::ensnano_iced::Element<'a, Message, crate::ensnano_iced::Theme, iced::Renderer>
    where
        Message: 'a + Clone,
    {
        fn from(hue_column: HueColumn<'a, Message>) -> Self {
            Self::new(hue_column)
        }
    }
}

/// A widget to select Lightness and Saturation values.
mod light_sat_square {
    use color_space::{Hsv, Rgb};
    use iced::{
        Length, Point, Rectangle, Renderer, Size, Vector,
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
    pub struct LightSatSquare<'a, Message: Clone> {
        hue: f64,
        on_slide: Box<dyn Fn(f64, f64) -> Message + 'a>,
        // TODO: Mimic iced like in Checkbox: Option<Box<…>>, then a method to set something else
        //       than None.
        on_finish: Message,
        // TODO: Mimic iced like in Button: Option<…>, then a method to set something else than None.
    }

    impl<'a, Message: Clone> LightSatSquare<'a, Message> {
        pub fn new<F>(hue: f64, on_slide: F, on_finish: Message) -> Self
        where
            F: 'static + Fn(f64, f64) -> Message + 'a,
        {
            Self {
                hue,
                on_slide: Box::new(on_slide),
                on_finish,
            }
        }
    }

    impl<'a, Message> Widget<Message, crate::ensnano_iced::Theme, iced::Renderer>
        for LightSatSquare<'a, Message>
    where
        Message: Clone + 'a,
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
            _renderer: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);

            layout::Node::new(Size::new(size.width, size.width))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut iced::Renderer,
            _theme: &crate::ensnano_iced::Theme,
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

            let mesh = Custom::Mesh(Mesh::Solid {
                size: b.size(),
                buffers: Indexed { vertices, indices },
            });

            match renderer {
                Renderer::Wgpu(wgpu_renderer) => {
                    wgpu_renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
                        renderer.draw_primitive(Primitive::Custom(mesh));
                    });
                }
                _ => panic!("Unhandled renderer"),
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
                shell.publish((self.on_slide)(saturation, value));
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
                        shell.publish(self.on_finish.clone());
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

    impl<'a, Message> From<LightSatSquare<'a, Message>>
        for crate::ensnano_iced::Element<'a, Message, crate::ensnano_iced::Theme, iced::Renderer>
    where
        Message: Clone + 'a,
    {
        fn from(value: LightSatSquare<'a, Message>) -> Self {
            Self::new(value)
        }
    }
}

/// A widget to Visualize selected color.
mod color_square {
    use super::Color;
    use iced::{
        Length, Rectangle, Size, Vector,
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

    /// The State of a [ColorSquare]
    #[derive(Default, Clone, Eq, PartialEq)]
    pub struct ColorSquareState {
        clicked: bool,
    }

    /// A ColorSquare Widget
    pub struct ColorSquare<'a, Message>
    where
        Message: Clone,
    {
        //state: &'a mut State,
        color: Color,
        on_click: Box<dyn Fn(Color) -> Message + 'a>,
        // TODO: Mimic iced like in Checkbox: Option<Box<…>>, then a method to set something else
        //       than None.
        on_release: Message,
        // TODO: Mimic iced like in Button: Option<…>, then a method to set something else than None.
    }

    impl<'a, Message> ColorSquare<'a, Message>
    where
        Message: Clone,
    {
        pub fn new<F>(color: Color, on_click: F, on_release: Message) -> Self
        where
            F: 'static + Fn(Color) -> Message + 'a,
        {
            Self {
                //state,
                color,
                on_click: Box::new(on_click),
                on_release,
            }
        }
    }

    impl<'a, Message> Widget<Message, crate::ensnano_iced::Theme, iced::Renderer>
        for ColorSquare<'a, Message>
    where
        Message: Clone + 'a,
    {
        fn state(&self) -> widget::tree::State {
            widget::tree::State::Some(Box::new(ColorSquareState::default()))
        }
        fn size(&self) -> Size<Length> {
            Size {
                width: Length::FillPortion(1),
                height: Length::FillPortion(1),
            }
        }

        fn layout(
            &self,
            _tree: &mut widget::Tree,
            _renderer: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);

            layout::Node::new(Size::new(size.width, size.width))
        }

        fn draw(
            &self,
            _tree: &widget::Tree,
            renderer: &mut iced::Renderer,
            _theme: &crate::ensnano_iced::Theme,
            _style: &Style,
            layout: Layout,
            _cursor: Cursor,
            _viewport: &Rectangle,
        ) {
            let b = layout.bounds();
            let x_max = b.width;
            let y_max = b.height;
            let dummy_color = pack([1.0, 0.0, 0.0, 1.0]);
            // TODO: Find an appropriate color.
            // The primitive API changed. It now ask for
            // some color. I do not now which one to choose now.
            let vertices = vec![
                SolidVertex2D {
                    position: [0., 0.],
                    color: dummy_color,
                },
                SolidVertex2D {
                    position: [0., y_max],
                    color: dummy_color,
                },
                SolidVertex2D {
                    position: [x_max, 0.],
                    color: dummy_color,
                },
                SolidVertex2D {
                    position: [x_max, y_max],
                    color: dummy_color,
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
                    })
                }
                _ => panic!("Unhandled renderer."),
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
            if let event::Event::Mouse(mouse_event) = event {
                let state = tree.state.downcast_mut::<ColorSquareState>();
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if cursor.is_over(layout.bounds()) {
                            state.clicked = true;
                            shell.publish((self.on_click)(self.color));
                            event::Status::Captured
                        } else {
                            event::Status::Ignored
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) if state.clicked => {
                        if cursor.is_over(layout.bounds()) {
                            state.clicked = false;
                            shell.publish(self.on_release.clone());
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
                            shell.publish(self.on_release.clone());
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

    impl<'a, Message> From<ColorSquare<'a, Message>>
        for crate::ensnano_iced::Element<'a, Message, crate::ensnano_iced::Theme, iced::Renderer>
    where
        Message: Clone + 'a,
    {
        fn from(color_square: ColorSquare<'a, Message>) -> Self {
            Self::new(color_square)
        }
    }
}
