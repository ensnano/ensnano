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
//! A handmade color picker for ENSnano.
//!
//! This module is an example of how to embed mini iced applications. It is inspired by the
//! [`Component`](https://docs.rs/iced/0.12.1/iced/widget/trait.Component.html) trait, but it does
//! not use it as this trait is deprecated since v0.13…
//!
//!
use super::widgets::{ColorSquare, HueRow, LightSatSquare};
use color_space::{Hsv, Rgb};
use iced::{
    Color, Length,
    widget::{Column, Row, column, row},
};
use std::collections::VecDeque;

// TODO: Adjust to tab height.

const DEFAULT_SIZE: f32 = 30.0;
// Ratio between Hue column height and width.
// Gives a number of memory cells: 2(FACTOR-2).
// Choose between: [3,4,5,6]
const FACTOR: usize = 6;
// Gap between squares.
const GAP: f32 = 3.0;

/// Messages from ColorPicker
#[derive(Debug, Clone, Copy)]
pub enum ColorPickerMessage {
    HueChanged(f64),
    HsvSatValueChanged(f64, f64),
    ColorPicked(Color),
    FinishChangingColor,
}
// Local alias
type Message = ColorPickerMessage;

fn hsv_to_color(hsv: Hsv) -> Color {
    let Rgb { r, g, b } = Rgb::from(hsv);
    Color::from_rgb(r as f32 / 255., g as f32 / 255., b as f32 / 255.)
}

fn color_to_hsv(color: Color) -> Hsv {
    let Color { r, g, b, a: _ } = color;
    Hsv::from(Rgb {
        r: r as f64 * 255.,
        g: g as f64 * 255.,
        b: b as f64 * 255.,
    })
}

/// Helper function to create color squares.
fn color_square<'a>(color: Color) -> ColorSquare<'a, Message> {
    ColorSquare::new(color)
        .on_click(Message::ColorPicked)
        .on_release(Message::FinishChangingColor)
}

pub struct ColorPicker {
    size: f32,
    current_color: Hsv,
    color_history: VecDeque<Color>,
    // TODO: Evaluate if using bounded-vec-deque crate is advantageous
    //       https://docs.rs/bounded-vec-deque/0.1.0/bounded_vec_deque/index.html
}

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            size: DEFAULT_SIZE,
            current_color: Hsv {
                h: 0.,
                s: 1.,
                v: 1.,
            },
            color_history: VecDeque::new(),
        }
    }

    fn history_size(&self) -> usize {
        2 * (FACTOR - 2)
    }

    pub fn current_color(&self) -> Color {
        hsv_to_color(self.current_color)
    }

    pub fn current_hue(&self) -> f64 {
        self.current_color.h
    }

    fn add_color_to_history(&mut self, color: Color) {
        if !self.color_history.contains(&color) {
            self.color_history.push_front(color);
            self.color_history.truncate(self.history_size());
        }
    }

    pub fn update(&mut self, message: Message) {
        // TODO: Managed color_square internally
        match message {
            // HueColumn message
            Message::HueChanged(hue) => {
                self.current_color = Hsv {
                    h: hue,
                    ..self.current_color
                };
                //self.update_color();
            }
            // HsvSat square message
            Message::HsvSatValueChanged(saturation, value) => {
                self.current_color = Hsv {
                    s: saturation,
                    v: value,
                    ..self.current_color
                };
                //self.update_color();
            }
            // Color square message
            Message::ColorPicked(color) => {
                self.current_color = color_to_hsv(color);
            }
            Message::FinishChangingColor => {
                self.add_color_to_history(self.current_color());
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        column![
            HueRow::new()
                .on_slide(Message::HueChanged)
                .height(Length::Fixed(self.size))
                .width(Length::Fixed(FACTOR as f32 * self.size)),
            LightSatSquare::new(self.current_hue())
                .on_slide(Message::HsvSatValueChanged)
                .on_finish(Message::FinishChangingColor)
                .height(Length::Fixed(FACTOR as f32 * self.size))
                .width(Length::Fixed(FACTOR as f32 * self.size)),
            row![
                color_square(self.current_color())
                    .height(Length::Fixed(2.0 * self.size - GAP))
                    .width(Length::Fixed(2.0 * self.size - GAP)),
                self.view_color_history(),
            ]
            .spacing(GAP),
        ]
        .spacing(GAP)
        .into()
    }

    fn view_color_history(&self) -> iced::Element<'_, Message> {
        let mut color_squares = self
            .color_history
            .iter()
            .map(|c| {
                color_square(c.to_owned())
                    .height(Length::Fixed(self.size - GAP))
                    .width(Length::Fixed(self.size - GAP))
            })
            .into_iter();
        let mut row = Vec::new();
        loop {
            let first_square = color_squares.next();
            let second_square = color_squares.next();
            match (first_square, second_square) {
                (Some(sq1), Some(sq2)) => {
                    row.push(column![sq1, sq2].spacing(GAP));
                }
                (Some(sq1), None) => {
                    row.push(column![sq1].spacing(GAP));
                    break;
                }
                (None, Some(sq2)) => {
                    log::error!("Buggy situation in color_picker history colors");
                    row.push(column![sq2].spacing(GAP));
                }
                (None, None) => break,
            }
        }
        let row = row.into_iter().map(Column::into);
        Row::from_vec(row.collect()).spacing(GAP).into()
    }
}
