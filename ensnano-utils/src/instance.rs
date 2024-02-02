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
use ultraviolet::{Mat4, Rotor3, Vec3, Vec4};
#[derive(Debug, Copy, Clone)]
/// The instantiation of an object
pub struct Instance {
    /// The position in space
    pub position: Vec3,
    /// The rotation of the instance
    pub rotor: Rotor3,
    pub color: Vec4,
    pub id: u32,
    pub scale: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    /// The model matrix of the instance
    pub model: Mat4,
    pub color: Vec4,
    pub id: Vec4,
}

impl Instance {
    pub fn grey_u32_color_from_f32(grey: f32) -> u32 {
        let g = (grey * 255.).round() as u32;
        return g << 16 | g << 8 | g;
    }

    pub fn grey_au32_color_from_f32(grey: f32, alpha: f32) -> u32 {
        let a = (alpha * 255.).round() as u32;
        return Self::grey_u32_color_from_f32(grey) | a << 24;
    }

    pub fn color_from_u32(color: u32) -> Vec4 {
        let red = (color & 0xFF0000) >> 16;
        let green = (color & 0x00FF00) >> 8;
        let blue = color & 0x0000FF;
        Vec4::new(
            red as f32 / 255.,
            green as f32 / 255.,
            blue as f32 / 255.,
            1.,
        )
    }

    pub fn add_alpha_to_clear_color_u32(color: u32) -> u32 {
        let alpha = (color & 0xFF_00_00_00) >> 24;
        if alpha == 0 {
            color | 0xFF_00_00_00
        } else {
            color
        }
    }

    pub fn color_au32_with_alpha_scaled_by(color: u32, alpha_scale: f32) -> u32 {
        let alpha = (color & 0xFF_00_00_00) >> 24;
        let alpha = (alpha as f32 * alpha_scale).round().max(0.).min(255.) as u32;
        return (color & 0xFF_FF_FF) | (alpha << 24);
    }

    pub fn unclear_color_from_u32(color: u32) -> Vec4 {
        let red = (color & 0xFF0000) >> 16;
        let green = (color & 0x00FF00) >> 8;
        let blue = color & 0x0000FF;
        let alpha = (color & 0xFF000000) >> 24;
        Vec4::new(
            red as f32 / 255.,
            green as f32 / 255.,
            blue as f32 / 255.,
            if alpha == 0 { 1. } else { alpha as f32 / 255. },
        )
    }

    pub fn color_from_au32(color: u32) -> Vec4 {
        let red = (color & 0xFF0000) >> 16;
        let green = (color & 0x00FF00) >> 8;
        let blue = color & 0x0000FF;
        let alpha = (color & 0xFF000000) >> 24;
        Vec4::new(
            red as f32 / 255.,
            green as f32 / 255.,
            blue as f32 / 255.,
            alpha as f32 / 255.,
        )
    }

    #[allow(dead_code)]
    pub fn id_from_u32(id: u32) -> Vec4 {
        let a = (id & 0xFF000000) >> 24;
        let r = (id & 0x00FF0000) >> 16;
        let g = (id & 0x0000FF00) >> 8;
        let b = id & 0x000000FF;
        Vec4::new(r as f32 / 255., g as f32 / 255., b as f32 / 255., a as f32)
    }
}
