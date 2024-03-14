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
use super::{AppState, ColorMessage, Message};
use iced_native::Color;

pub struct ColorPicker {
    color: Color,
    hue: f64,
    saturation: f64,
    hsv_value: f64,
}

pub use color_square::ColorSquare;
use hue_column::HueColumn;
use light_sat_square::LightSatSquare;

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            color: Color::BLACK,
            hue: 0.,
            saturation: 1.,
            hsv_value: 1.,
        }
    }

    pub fn update_color(&mut self) -> Color {
        use color_space::{Hsv, Rgb};
        let hsv = Hsv::new(self.hue, self.saturation, self.hsv_value);
        let rgb = Rgb::from(hsv);
        let color: Color = [
            rgb.r as f32 / 255.,
            rgb.g as f32 / 255.,
            rgb.b as f32 / 255.,
            1.,
        ]
        .into();
        self.color = color;
        color
    }

    pub fn change_hue(&mut self, hue: f64) {
        self.hue = hue
    }

    pub fn set_saturation(&mut self, saturation: f64) {
        self.saturation = saturation
    }

    pub fn set_hsv_value(&mut self, hsv_value: f64) {
        self.hsv_value = hsv_value
    }

    pub fn view<S>(&self) -> iced::Element<Message<S>>
    where
        S: AppState,
    {
        iced_native::row![
            HueColumn::new(Message::HueChanged),
            LightSatSquare::new(
                self.hue as f64,
                Message::HsvSatValueChanged,
                Message::FinishChangingColor,
            ),
        ]
        .spacing(10)
        .into()
    }

    pub fn color_square<'a, S: AppState>(&self) -> ColorSquare<'a, Message<S>> {
        ColorSquare::new(
            self.color,
            Message::ColorPicked,
            Message::FinishChangingColor,
        )
    }

    pub fn new_view(&self) -> iced::Element<ColorMessage> {
        iced_native::row![
            HueColumn::new(ColorMessage::HueChanged,),
            LightSatSquare::new(
                self.hue as f64,
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
    use iced_graphics::{
        renderer::{Renderer, Style},
        triangle::{ColoredVertex2D, Mesh2D},
        Primitive, Rectangle,
    };
    use iced_native::{
        layout, mouse, widget, Clipboard, Event, Layout, Length, Point, Renderer as RendererTrait,
        Shell, Size, Vector, Widget,
    };

    use color_space::{Hsv, Rgb};

    /// The internal state of a [HueColumnState].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct HueColumnState {
        is_dragging: bool,
    }

    /// A HueColumn Widget.
    pub struct HueColumn<'a, Message> {
        on_slide: Box<dyn Fn(f64) -> Message + 'a>,
    }

    impl<'a, Message> HueColumn<'a, Message> {
        pub fn new<F>(on_slide: F) -> Self
        where
            F: 'static + Fn(f64) -> Message,
        {
            Self {
                on_slide: Box::new(on_slide),
            }
        }
    }

    impl<'a, Message, Backend, Theme> Widget<Message, Renderer<Backend, Theme>>
        for HueColumn<'a, Message>
    where
        Backend: iced_graphics::Backend,
    {
        fn state(&self) -> widget::tree::State {
            widget::tree::State::Some(Box::new(HueColumnState::default()))
        }
        fn width(&self) -> Length {
            Length::FillPortion(1)
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer<Backend, Theme>,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits
                .width(Length::Fill)
                .height(Length::Fill)
                .resolve(Size::ZERO);

            layout::Node::new(Size::new(size.width, 4. * size.width))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut Renderer<Backend, Theme>,
            _theme: &Theme,
            _style: &Style,
            layout: Layout<'_>,
            _cursor_position: Point,
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
                let color = [
                    rgb.r as f32 / 255.,
                    rgb.g as f32 / 255.,
                    rgb.b as f32 / 255.,
                    1.,
                ];
                vertices.push(ColoredVertex2D {
                    position: [0., y_max * (i as f32 / nb_row as f32)],
                    color,
                });
                vertices.push(ColoredVertex2D {
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

            renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
                renderer.draw_primitive(Primitive::SolidMesh {
                    size: b.size(),
                    buffers: Mesh2D { vertices, indices },
                })
            });
        }

        fn on_event(
            &mut self,
            tree: &mut widget::Tree,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
            _renderer: &Renderer<Backend, Theme>,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> iced_native::event::Status {
            let mut change = || {
                let bounds = layout.bounds();
                if cursor_position.y <= bounds.y {
                    shell.publish((self.on_slide)(0.));
                } else if cursor_position.y >= bounds.y + bounds.height {
                    shell.publish((self.on_slide)(360.));
                } else {
                    let percent = (cursor_position.y - bounds.y) / bounds.height;
                    let value = percent * 360.;
                    shell.publish((self.on_slide)(value.into()));
                }
            };

            if let Event::Mouse(mouse_event) = event {
                let state = tree.state.downcast_mut::<HueColumnState>();
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if layout.bounds().contains(cursor_position) {
                            change();
                            state.is_dragging = true;
                        }
                        iced_native::event::Status::Captured
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        if state.is_dragging {
                            state.is_dragging = false;
                        }
                        iced_native::event::Status::Captured
                    }
                    mouse::Event::CursorMoved { .. } => {
                        if state.is_dragging {
                            change();
                            iced_native::event::Status::Captured
                        } else {
                            iced_native::event::Status::Ignored
                        }
                    }
                    _ => iced_native::event::Status::Ignored,
                }
            } else {
                iced_native::event::Status::Ignored
            }
        }
    }

    impl<'a, Message> From<HueColumn<'a, Message>> for iced::Element<'a, Message>
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
    use iced_graphics::{
        renderer::{Renderer, Style},
        triangle::{ColoredVertex2D, Mesh2D},
        Primitive, Rectangle,
    };
    use iced_native::{
        layout, mouse, widget, Clipboard, Event, Layout, Length, Point, Renderer as RendererTrait,
        Shell, Size, Vector, Widget,
    };

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
    pub struct LightSatSquare<'a, Message: Clone> {
        hue: f64,
        on_slide: Box<dyn Fn(f64, f64) -> Message + 'a>,
        on_finish: Message,
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

    impl<'a, Message, Backend, Theme> Widget<Message, Renderer<Backend, Theme>>
        for LightSatSquare<'a, Message>
    where
        Message: Clone + 'a,
        Backend: iced_graphics::Backend,
    {
        fn state(&self) -> widget::tree::State {
            widget::tree::State::Some(Box::new(LightSatState::default()))
        }
        fn width(&self) -> Length {
            Length::FillPortion(4)
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(
            &self,
            _renderer: &Renderer<Backend, Theme>,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits
                .width(Length::Fill)
                .height(Length::Fill)
                .resolve(Size::ZERO);

            layout::Node::new(Size::new(size.width, size.width))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut Renderer<Backend, Theme>,
            _theme: &Theme,
            _style: &Style,
            layout: Layout<'_>,
            _cursor_position: Point,
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
                    let color = hsv_to_linear(self.hue, sat, value);
                    vertices.push(ColoredVertex2D {
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

            renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
                renderer.draw_primitive(Primitive::SolidMesh {
                    size: b.size(),
                    buffers: Mesh2D { vertices, indices },
                })
            });
        }

        fn on_event(
            &mut self,
            tree: &mut widget::Tree,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
            _renderer: &Renderer<Backend, Theme>,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> iced_native::event::Status {
            let mut change = || {
                let bounds = layout.bounds();
                let percent_x = if cursor_position.x <= bounds.x {
                    0.
                } else if cursor_position.x >= bounds.x + bounds.width {
                    1.
                } else {
                    f64::from(cursor_position.x - bounds.x) / f64::from(bounds.width)
                };

                let percent_y = if cursor_position.y <= bounds.y {
                    0.
                } else if cursor_position.y >= bounds.y + bounds.height {
                    1.
                } else {
                    f64::from(cursor_position.y - bounds.y) / f64::from(bounds.height)
                };

                let saturation = 1. - percent_x;
                let value = 1. - percent_y;
                shell.publish((self.on_slide)(saturation, value));
            };

            if let Event::Mouse(mouse_event) = event {
                let state = tree.state.downcast_mut::<LightSatState>();
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if layout.bounds().contains(cursor_position) {
                            change();
                            state.is_dragging = true;
                            iced_native::event::Status::Captured
                        } else {
                            iced_native::event::Status::Ignored
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        if state.is_dragging {
                            state.is_dragging = false;
                        }
                        shell.publish(self.on_finish.clone());
                        iced_native::event::Status::Captured
                    }
                    mouse::Event::CursorMoved { .. } => {
                        if state.is_dragging {
                            change();
                        }
                        iced_native::event::Status::Captured
                    }
                    _ => iced_native::event::Status::Ignored,
                }
            } else {
                iced_native::event::Status::Ignored
            }
        }
    }

    impl<'a, Message> From<LightSatSquare<'a, Message>> for iced::Element<'a, Message>
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
    use iced_graphics::{
        renderer::{Renderer, Style},
        triangle::{ColoredVertex2D, Mesh2D},
        Primitive, Rectangle,
    };
    use iced_native::{
        layout, mouse, widget, Clipboard, Event, Layout, Length, Point, Renderer as RendererTrait,
        Shell, Size, Vector, Widget,
    };

    /// The State of a [ColorSquare]
    #[derive(Default, Clone, Eq, PartialEq)]
    pub struct ColorSquareState {
        clicked: bool,
    }

    /// A ColorSquare Widget
    pub struct ColorSquare<'a, Message: Clone> {
        //state: &'a mut State,
        color: Color,
        on_click: Box<dyn Fn(Color) -> Message + 'a>,
        on_release: Message,
    }

    impl<'a, Message: Clone> ColorSquare<'a, Message> {
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

    impl<'a, Message, Backend, Theme> Widget<Message, Renderer<Backend, Theme>>
        for ColorSquare<'a, Message>
    where
        Message: Clone + 'a,
        Backend: iced_graphics::Backend,
    {
        fn state(&self) -> widget::tree::State {
            widget::tree::State::Some(Box::new(ColorSquareState::default()))
        }
        fn width(&self) -> Length {
            Length::FillPortion(1)
        }

        fn height(&self) -> Length {
            Length::FillPortion(1)
        }

        fn layout(
            &self,
            _renderer: &Renderer<Backend, Theme>,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits
                .width(Length::Fill)
                .height(Length::Fill)
                .resolve(Size::ZERO);

            layout::Node::new(Size::new(size.width, size.width))
        }

        fn draw(
            &self,
            _state: &widget::Tree,
            renderer: &mut Renderer<Backend, Theme>,
            _theme: &Theme,
            _style: &Style,
            layout: Layout<'_>,
            _cursor_position: Point,
            _viewport: &Rectangle,
        ) {
            let b = layout.bounds();
            let x_max = b.width;
            let y_max = b.height;
            let dummy_color = [1.0, 0.0, 0.0, 1.0]; // TODO: Find an appropriate color.
                                                    // The primitiver API changed. It now ask for
                                                    // some color. I do not now which one to choose now.
            let vertices = vec![
                ColoredVertex2D {
                    position: [0., 0.],
                    color: dummy_color,
                },
                ColoredVertex2D {
                    position: [0., y_max],
                    color: dummy_color,
                },
                ColoredVertex2D {
                    position: [x_max, 0.],
                    color: dummy_color,
                },
                ColoredVertex2D {
                    position: [x_max, y_max],
                    color: dummy_color,
                },
            ];
            let indices = vec![0, 1, 2, 1, 2, 3];

            renderer.with_translation(Vector::new(b.x, b.y), |renderer| {
                renderer.draw_primitive(Primitive::SolidMesh {
                    buffers: Mesh2D { vertices, indices },
                    size: b.size(),
                })
            });
        }

        fn on_event(
            &mut self,
            tree: &mut widget::Tree,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
            _renderer: &Renderer<Backend, Theme>,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> iced_native::event::Status {
            if let Event::Mouse(mouse_event) = event {
                let state = tree.state.downcast_mut::<ColorSquareState>();
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if layout.bounds().contains(cursor_position) {
                            state.clicked = true;
                            shell.publish((self.on_click)(self.color));
                            iced_native::event::Status::Captured
                        } else {
                            iced_native::event::Status::Ignored
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) if state.clicked => {
                        if layout.bounds().contains(cursor_position) {
                            state.clicked = false;
                            shell.publish(self.on_release.clone());
                            iced_native::event::Status::Captured
                        } else {
                            iced_native::event::Status::Ignored
                        }
                    }
                    mouse::Event::CursorMoved { .. } if state.clicked => {
                        if layout.bounds().contains(cursor_position) {
                            iced_native::event::Status::Ignored
                        } else {
                            state.clicked = false;
                            shell.publish(self.on_release.clone());
                            iced_native::event::Status::Captured
                        }
                    }
                    _ => iced_native::event::Status::Ignored,
                }
            } else {
                iced_native::event::Status::Ignored
            }
        }
    }

    impl<'a, Message> From<ColorSquare<'a, Message>> for iced::Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        fn from(color_square: ColorSquare<'a, Message>) -> Self {
            Self::new(color_square)
        }
    }
}
