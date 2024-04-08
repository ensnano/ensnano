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

// Give a color given a set of (t_i,h_i) where t_i is increasing from 0 to 1 such that for t_i ≤ t ≤ t_i+1, the hue is the linera interpolation of h_i and h_i+1 - h is considered modulo 360

pub const PURPLE_TO_BLUE_GRADIENT: [(f32, f32); 8] = [
    (0., -56.),  // purple
    (0.1, 0.),   // red
    (0.25, 27.), // orange
    (0.35, 54.), // yellow
    (0.5, 100.), // green
    (0.7, 176.), // cyan
    (0.8, 202.), // blue
    (1., 242.),  // dark blue
];

pub fn gradient_color(t: f32, t_hues: &[(f32, f32)]) -> u32 {
    assert!(t_hues.len() > 0, "/!\\ Empty gradient description");
    if t <= t_hues[0].0 {
        return hsv_color((t_hues[0].1 as f64).rem_euclid(360.), 1., 1.);
    }
    for ((t0, h0), (t1, h1)) in t_hues.iter().zip(t_hues.iter().skip(1)) {
        if *t0 <= t && t <= *t1 {
            let hue = (h0 + (h1 - h0) * (t - t0) / (t1 - t0)).rem_euclid(360.);
            return hsv_color(hue as f64, 1., 1.);
        }
    }
    return hsv_color((t_hues.last().unwrap().1 as f64).rem_euclid(360.), 1., 1.);
}

#[inline(always)]
pub fn purple_to_blue_gradient_color(t: f32) -> u32 {
    gradient_color(t, &PURPLE_TO_BLUE_GRADIENT)
}

#[inline(always)]
pub fn purple_to_blue_gradient_color_in_range(t: f32, t_min: f32, t_max: f32) -> u32 {
    purple_to_blue_gradient_color((t - t_min) / (t_max - t_min))
}
