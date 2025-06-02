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
use iced::{widget::row, Color};
use iced_winit::winit::dpi::LogicalSize;

use crate::widgets::{ColorSquare, HueColumn, LightSatSquare};

/// Messages from ColorPicker
#[derive(Debug, Clone, Copy)]
pub enum ColorPickerMessage {
    HueChanged(f64),
    HsvSatValueChanged(f64, f64),
    ColorPicked(Color),
    FinishChangingColor,
    Resized(LogicalSize<f64>),
}
type Message = ColorPickerMessage; // Local alias

pub struct ColorPicker {
    color: Color,
    hue: f64,
    saturation: f64,
    hsv_value: f64,
}

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

    pub fn view(&self) -> crate::Element<'_, Message, crate::Theme, iced_wgpu::Renderer> {
        row![
            HueColumn::new().on_slide(Message::HueChanged),
            LightSatSquare::new(self.hue as f64)
                .on_slide(Message::HsvSatValueChanged)
                .on_finish(Message::FinishChangingColor),
        ]
        .spacing(10)
        .into()
    }

    pub fn color_square(&self) -> ColorSquare<'_, Message> {
        ColorSquare::new(self.color)
            .on_click(Message::ColorPicked)
            .on_release(Message::FinishChangingColor)
    }
}
