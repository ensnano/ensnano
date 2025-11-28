//! This modules contains structure that manipulate bind groups and their associated buffers.

use crate::ensnano_utils::create_buffer_with_data;
use std::rc::Rc;
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferDescriptor, Device, Queue};

/// A bind group with an associated buffer whose size may vary
pub struct DynamicBindGroup {
    layout: BindGroupLayout,
    buffer: Buffer,
    capacity: usize,
    length: u64,
    bind_group: BindGroup,
    device: Rc<Device>,
    queue: Rc<Queue>,
}

const INITIAL_CAPACITY: u64 = 1024;

impl DynamicBindGroup {
    pub fn new(device: Rc<Device>, queue: Rc<Queue>, label: &str) -> Self {
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
            buffer,
            capacity,
            length,
            bind_group,
            device,
            queue,
        }
    }

    /// Replace the data of the associated buffer.
    pub fn update<I: bytemuck::Pod>(&mut self, data: &[I]) {
        let bytes = bytemuck::cast_slice(data);
        if self.capacity < bytes.len() {
            self.length = bytes.len() as u64;
            self.buffer = self.device.create_buffer(&BufferDescriptor {
                label: Some(&format!("capacity = {}", 2 * bytes.len())),
                size: 2 * bytes.len() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.capacity = 2 * bytes.len();
            self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.buffer,
                        size: wgpu::BufferSize::new(self.length),
                        offset: 0,
                    }),
                }],
                label: None,
            });
        } else if self.length != bytes.len() as u64 {
            self.length = bytes.len() as u64;
            self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.buffer,
                        size: wgpu::BufferSize::new(self.length),
                        offset: 0,
                    }),
                }],
                label: None,
            });
        }
        self.queue.write_buffer(&self.buffer, 0, bytes);
    }

    pub fn get_bindgroup(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}

/// A structure that manages a bind group associated to a uniform buffer
pub struct UniformBindGroup {
    layout: BindGroupLayout,
    buffer: Buffer,
    bind_group: BindGroup,
    queue: Rc<Queue>,
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
    pub fn new<I: bytemuck::Pod>(
        device: Rc<Device>,
        queue: Rc<Queue>,
        viewer_data: &I,
        label: &str,
    ) -> Self {
        let buffer = create_buffer_with_data(
            &device,
            bytemuck::cast_slice(&[*viewer_data]),
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
            buffer,
            bind_group,
            queue,
        }
    }

    pub fn update<I: bytemuck::Pod>(&self, new_data: &I) {
        self.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[*new_data]));
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
