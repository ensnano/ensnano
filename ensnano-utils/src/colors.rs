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

use ensnano_interactor::consts::{
    RANDOM_COLOR_SHADE_HUE_RANGE, RANDOM_COLOR_SHADE_SATURATION_RANGE,
    RANDOM_COLOR_SHADE_VALUE_RANGE,
};
use rand::Rng;

pub fn hsv_color(hue: f64, saturation: f64, value: f64) -> u32 {
    let hsv = color_space::Hsv::new(hue, saturation, value);
    let rgb = color_space::Rgb::from(hsv);
    (0xFF << 24) | ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | (rgb.b as u32)
}

pub fn new_color(color_idx: &mut usize) -> u32 {
    // Fibonachi hue coloring scheme
    let color = {
        let hue = (*color_idx as f64 * (1. + 5f64.sqrt()) / 2.).fract() * 360.;
        let saturation = (*color_idx as f64 * 7. * (1. + 5f64.sqrt() / 2.)).fract() * 0.25 + 0.75;
        let value = (*color_idx as f64 * 11. * (1. + 5f64.sqrt() / 2.)).fract() * 0.5 + 0.5;
        hsv_color(hue, saturation, value)
    };
    *color_idx += 1;
    color
}

pub fn random_color_with_shade(shade: u32, hue_range: Option<f64>) -> u32 {
    // generate a random color around the shade
    let h_range = hue_range.unwrap_or(RANDOM_COLOR_SHADE_HUE_RANGE);
    let s_range = RANDOM_COLOR_SHADE_SATURATION_RANGE;
    let v_range = RANDOM_COLOR_SHADE_VALUE_RANGE;

    let (a, r, g, b) = (
        shade & 0xFF_00_00_00,
        (shade & 0xFF0000) >> 16,
        (shade & 0x00FF00) >> 8,
        shade & 0x0000FF,
    );
    let shade = color_space::Hsv::from(color_space::Rgb::new(r as f64, g as f64, b as f64));
    // randomly modify the shade
    let mut rng = rand::thread_rng();
    let hue = (shade.h / 360. + h_range * (2. * rng.gen::<f64>() - 1.)).fract() * 360.;
    let saturation = (shade.s.min(1. - s_range) + s_range * (2. * rng.gen::<f64>() - 1.))
        .min(1.)
        .max(0.);
    let value = (shade.v.min(1. - v_range) + v_range * (2. * rng.gen::<f64>() - 1.))
        .min(1.)
        .max(0.);

    let color = (hsv_color(hue, saturation, value) & 0xFF_FF_FF) | a;

    return color;
}
