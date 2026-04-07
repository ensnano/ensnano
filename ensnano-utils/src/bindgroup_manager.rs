//! This modules contains structure that manipulate bind groups and their associated buffers.

use crate::create_buffer_with_data;
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferDescriptor, Device, Queue};

/// A bind group with an associated buffer whose size may vary.
pub struct DynamicBindGroup {
    layout: BindGroupLayout,
    bytes: Vec<u8>,
    buffer: Buffer,
    length: usize,
    capacity: usize,
    bind_group: BindGroup,
}

const INITIAL_CAPACITY: u64 = 1024;

impl DynamicBindGroup {
    pub fn new(device: &Device, label: &str) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: INITIAL_CAPACITY,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let capacity = INITIAL_CAPACITY as usize;
        let length = 0;

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    // We don't plan on changing the size of this buffer
                    has_dynamic_offset: false,
                    // The shader is not allowed to modify it's contents
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    size: None,
                    offset: 0,
                }),
            }],
            label: Some("instance_bind_group"),
        });

        Self {
            layout,
            bytes: vec![],
            buffer,
            capacity,
            length,
            bind_group,
        }
    }

    /// Replace the data of the associated buffer.
    pub fn update<I: bytemuck::Pod>(&mut self, data: &[I]) {
        let bytes = bytemuck::cast_slice::<_, u8>(data);
        self.bytes = bytes.to_vec();
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        // We lack capacity; we reallocate a buffer of size 2 x the required size.
        if self.capacity < self.bytes.len() {
            self.capacity = self.bytes.len() * 2;
            self.buffer = device.create_buffer(&BufferDescriptor {
                label: Some(&format!("capacity = {}", self.capacity)),
                size: self.capacity as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.buffer, 0, &self.bytes);
        self.length = self.bytes.len();

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.buffer,
                    size: wgpu::BufferSize::new(self.length as u64),
                    offset: 0,
                }),
            }],
            label: None,
        });
    }

    pub fn get_bindgroup(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}

/// A structure that manages a bind group associated to a uniform buffer.
pub struct UniformBindGroup {
    layout: BindGroupLayout,
    bytes: Vec<u8>,
    buffer: Buffer,
    bind_group: BindGroup,
}

static UNIFORM_BG_ENTRY: &[wgpu::BindGroupLayoutEntry] = &[wgpu::BindGroupLayoutEntry {
    binding: 0,
    visibility: wgpu::ShaderStages::from_bits_truncate(
        wgpu::ShaderStages::VERTEX.bits() | wgpu::ShaderStages::FRAGMENT.bits(),
    ),
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
}];

impl UniformBindGroup {
    pub fn new<I: bytemuck::Pod>(device: &Device, viewer_data: &I, label: &str) -> Self {
        let bytes = bytemuck::cast_slice(&[*viewer_data]).to_vec();
        let buffer = create_buffer_with_data(
            device,
            &bytes,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label,
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: UNIFORM_BG_ENTRY,
            label: Some("uniform_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                // perspective and view
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        size: None,
                        offset: 0,
                    }),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        Self {
            layout,
            bytes,
            buffer,
            bind_group,
        }
    }

    pub fn update<I: bytemuck::Pod>(&mut self, new_data: &I) {
        self.bytes = bytemuck::cast_slice::<I, u8>(&[*new_data]).to_vec();
    }

    pub fn prepare(&self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, &self.bytes);
    }

    pub fn get_bindgroup(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    pub fn get_layout_desc(&self) -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: UNIFORM_BG_ENTRY,
            label: Some("uniform_bind_group"),
        }
    }
}
