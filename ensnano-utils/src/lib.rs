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
pub use iced_wgpu;
pub use iced_wgpu::wgpu;
pub use iced_winit;
pub use iced_winit::winit;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
pub use winit::dpi::{PhysicalPosition, PhysicalSize, Pixel};

pub mod bindgroup_manager;
pub mod camera2d;
pub mod chars2d;
pub mod circles2d;
pub mod full_isometry;
pub mod id_generator;
pub mod instance;
pub mod light;
pub mod mesh;
pub mod obj_loader;
pub mod text;
pub mod texture;

pub mod clic_counter;

pub mod colors;

pub mod filename;

pub type PhySize = PhysicalSize<u32>;
pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub fn create_buffer_with_data(
    device: &wgpu::Device,
    data: &[u8],
    usage: wgpu::BufferUsages,
    label: &str,
) -> wgpu::Buffer {
    let descriptor = BufferInitDescriptor {
        label: Some(label),
        contents: data,
        usage,
    };
    device.create_buffer_init(&descriptor)
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
        let bytes_per_pixel = std::mem::size_of::<u32>();
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
