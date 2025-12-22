pub mod colors;
pub mod instance;

use winit::dpi::{PhysicalPosition, PhysicalSize, Pixel};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Ndc {
    pub x: f32,
    pub y: f32,
}

impl Ndc {
    pub fn from_physical<S: Pixel, T: Pixel>(
        position: PhysicalPosition<S>,
        window_size: PhysicalSize<T>,
    ) -> Self {
        let position = position.cast::<f32>();
        let size = window_size.cast::<f32>();
        Self {
            x: position.x / size.width * 2. - 1.,
            y: position.y / size.height * -2. + 1.,
        }
    }
}
