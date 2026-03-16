use ultraviolet::{Mat4, Vec4};

#[derive(Debug, Copy, Clone)]
/// The instantiation of an object.
pub struct Instance;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    /// The model matrix of the instance.
    pub model: Mat4,
    pub color: Vec4,
    pub id: Vec4,
}

impl Instance {
    pub fn grey_u32_color_from_f32(grey: f32) -> u32 {
        let g = (grey * 255.).round() as u32;
        g << 16 | g << 8 | g
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
        let alpha = if alpha == 0 { 0xFF } else { alpha };
        let alpha = (alpha as f32 * alpha_scale).round().clamp(0., 255.) as u32;
        (color & 0xFF_FF_FF) | (alpha << 24)
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
}
