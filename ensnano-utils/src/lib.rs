pub mod colors;
pub mod instance;

use serde::{Deserialize, Serialize};
use winit::dpi::{PhysicalPosition, PhysicalSize, Pixel};

#[derive(Serialize, Deserialize, Debug)]
pub struct StrandNucleotidesPositions {
    pub is_cyclic: bool,
    pub positions: Vec<[f32; 3]>,
    pub curvatures: Vec<f64>,
    pub torsions: Vec<f64>,
}

/// This struct handle the alignment of row in WGPU buffers.
pub struct BufferDimensions {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
}

impl BufferDimensions {
    pub fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let block_size = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padding = (block_size - unpadded_bytes_per_row % block_size) % block_size;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;
        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
    pub fn buffer_size(&self) -> usize {
        self.padded_bytes_per_row * self.height
    }
}

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
