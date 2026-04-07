use crate::{
    CameraPtr,
    data::{
        helix::{GpuVertices, Helix},
        strand::{FreeEnd, Strand, StrandVertices},
    },
    flat_types::FlatNucl,
};
use wgpu::{Buffer, Device, Queue, RenderPass};

#[expect(clippy::large_enum_variant)]
pub(super) enum HelixView {
    Unprepared {
        vertices: GpuVertices,
        background: bool,
    },
    Prepared {
        vertex_buffer: DynamicBuffer,
        index_buffer: DynamicBuffer,
        num_instance: u32,
        background: bool,
    },
}

impl HelixView {
    pub(super) fn from_helix(helix: &Helix, background: bool) -> Self {
        Self::Unprepared {
            vertices: if background {
                helix.background_vertices()
            } else {
                helix.to_vertices()
            },
            background,
        }
    }

    pub(super) fn update(&mut self, helix: &Helix) {
        match self {
            Self::Unprepared {
                vertices,
                background,
            } => {
                *vertices = if *background {
                    helix.to_vertices()
                } else {
                    helix.background_vertices()
                };
            }
            Self::Prepared {
                vertex_buffer,
                index_buffer,
                num_instance,
                background,
            } => {
                let vertices = if *background {
                    helix.to_vertices()
                } else {
                    helix.background_vertices()
                };
                vertex_buffer.update(vertices.vertices.as_slice());
                index_buffer.update(vertices.indices.as_slice());
                *num_instance = vertices.indices.len() as u32;
            }
        }
    }

    pub(super) fn prepare(&mut self, device: &Device, queue: &Queue) {
        if let Self::Unprepared {
            vertices,
            background,
        } = self
        {
            let vertices = vertices.clone();
            *self = Self::Prepared {
                vertex_buffer: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::VERTEX,
                    "helix vertex buffer",
                ),
                index_buffer: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::INDEX,
                    "helix index buffer",
                ),
                num_instance: 0,
                background: *background,
            };

            if let Self::Prepared {
                vertex_buffer,
                index_buffer,
                num_instance,
                ..
            } = self
            {
                vertex_buffer.update(vertices.vertices.as_slice());
                index_buffer.update(vertices.indices.as_slice());
                *num_instance = vertices.indices.len() as u32;
            }
        }

        if let Self::Prepared {
            vertex_buffer,
            index_buffer,
            ..
        } = self
        {
            vertex_buffer.prepare(device, queue);
            index_buffer.prepare(device, queue);
        }
    }

    pub(super) fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        if let Self::Prepared {
            vertex_buffer,
            index_buffer,
            num_instance,
            ..
        } = self
        {
            if index_buffer.length == 0 || *num_instance == 0 || vertex_buffer.length == 0 {
                println!(
                    "[[Bug in ensnano_flatscene::view::helix_view::HelixView::draw]]: should not be empty\n\tindex_buffer={} num_instance={} vertex_buffer={}",
                    index_buffer.length, num_instance, index_buffer.length
                );
                return;
            }
            render_pass.set_index_buffer(index_buffer.get_slice(), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, vertex_buffer.get_slice());
            render_pass.draw_indexed(0..*num_instance, 0, 0..1);
        }
    }
}

#[expect(clippy::large_enum_variant)]
pub(super) enum StrandView {
    UnPrepared {
        top: StrandVertices,
        split_top: StrandVertices,
        bottom: StrandVertices,
        split_bottom: StrandVertices,
    },
    Prepared {
        vertex_buffer_top: DynamicBuffer,
        index_buffer_top: DynamicBuffer,
        num_instance_top: u32,
        vertex_buffer_bottom: DynamicBuffer,
        index_buffer_bottom: DynamicBuffer,
        num_instance_bottom: u32,

        split_vbo_top: DynamicBuffer,
        split_ibo_top: DynamicBuffer,
        num_instance_split_top: u32,
        split_vbo_bottom: DynamicBuffer,
        split_ibo_bottom: DynamicBuffer,
        num_instance_split_bottom: u32,
    },
}

impl StrandView {
    pub(super) fn from_strand(
        strand: &Strand,
        helices: &[Helix],
        free_end: Option<&FreeEnd>,
        top_cam: &CameraPtr,
        bottom_cam: &CameraPtr,
    ) -> Self {
        let (top, split_top) = strand.to_vertices(helices, free_end, top_cam, bottom_cam);
        let (bottom, split_bottom) = strand.to_vertices(helices, free_end, bottom_cam, top_cam);
        Self::UnPrepared {
            top,
            split_top,
            bottom,
            split_bottom,
        }
    }

    pub(super) fn from_indication(nucl1: FlatNucl, nucl2: FlatNucl, helices: &[Helix]) -> Self {
        let vertices = Strand::indication(nucl1, nucl2, helices);
        Self::UnPrepared {
            top: vertices.clone(),
            split_top: vertices.clone(),
            bottom: vertices.clone(),
            split_bottom: vertices,
        }
    }

    pub(super) fn update_strand(
        &mut self,
        strand: &Strand,
        helices: &[Helix],
        free_end: Option<&FreeEnd>,
        top_cam: &CameraPtr,
        bottom_cam: &CameraPtr,
    ) {
        match self {
            StrandView::UnPrepared {
                top,
                split_top,
                bottom,
                split_bottom,
            } => {
                (*top, *split_top) = strand.to_vertices(helices, free_end, top_cam, bottom_cam);
                (*bottom, *split_bottom) =
                    strand.to_vertices(helices, free_end, bottom_cam, top_cam);
            }
            StrandView::Prepared {
                vertex_buffer_top,
                index_buffer_top,
                num_instance_top,
                vertex_buffer_bottom,
                index_buffer_bottom,
                num_instance_bottom,
                split_vbo_top,
                split_ibo_top,
                num_instance_split_top,
                split_vbo_bottom,
                split_ibo_bottom,
                num_instance_split_bottom,
            } => {
                let (top, split_top) = strand.to_vertices(helices, free_end, top_cam, bottom_cam);
                let (bottom, split_bottom) =
                    strand.to_vertices(helices, free_end, bottom_cam, top_cam);

                vertex_buffer_top.update(top.vertices.as_slice());
                index_buffer_top.update(top.indices.as_slice());
                *num_instance_top = top.indices.len() as u32;
                split_vbo_top.update(split_top.vertices.as_slice());
                split_ibo_top.update(split_top.indices.as_slice());
                *num_instance_split_top = split_top.indices.len() as u32;
                vertex_buffer_bottom.update(bottom.vertices.as_slice());
                index_buffer_bottom.update(bottom.indices.as_slice());
                *num_instance_bottom = bottom.indices.len() as u32;
                split_vbo_bottom.update(split_bottom.vertices.as_slice());
                split_ibo_bottom.update(split_bottom.indices.as_slice());
                *num_instance_split_bottom = split_bottom.indices.len() as u32;
            }
        }
    }

    // pub(super) fn new(device: &Device) -> Self {
    //     Self {
    //         vertex_buffer_top: DynamicBuffer::new(
    //             device,
    //             wgpu::BufferUsages::VERTEX,
    //             "vertex buffer top",
    //         ),
    //         index_buffer_top: DynamicBuffer::new(
    //             device,
    //             wgpu::BufferUsages::INDEX,
    //             "index buffer top",
    //         ),
    //         split_vbo_top: DynamicBuffer::new(device, wgpu::BufferUsages::VERTEX, "split vbo top"),
    //         split_ibo_top: DynamicBuffer::new(device, wgpu::BufferUsages::INDEX, "split ibo top"),
    //         split_vbo_bottom: DynamicBuffer::new(
    //             device,
    //             wgpu::BufferUsages::VERTEX,
    //             "split vbo bottom",
    //         ),
    //         split_ibo_bottom: DynamicBuffer::new(
    //             device,
    //             wgpu::BufferUsages::INDEX,
    //             "split ibo bottom",
    //         ),
    //         vertex_buffer_bottom: DynamicBuffer::new(
    //             device,
    //             wgpu::BufferUsages::VERTEX,
    //             "vertex buffer bottom",
    //         ),
    //         index_buffer_bottom: DynamicBuffer::new(
    //             device,
    //             wgpu::BufferUsages::INDEX,
    //             "index buffer bottom",
    //         ),
    //         num_instance_top: 0,
    //         num_instance_bottom: 0,
    //         num_instance_split_top: 0,
    //         num_instance_split_bottom: 0,
    //     }
    // }

    pub(super) fn prepare(&mut self, device: &Device, queue: &Queue) {
        if let Self::UnPrepared {
            top,
            split_top,
            bottom,
            split_bottom,
        } = self
        {
            let top = top.clone();
            let split_top = split_top.clone();
            let bottom = bottom.clone();
            let split_bottom = split_bottom.clone();

            *self = Self::Prepared {
                vertex_buffer_top: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::VERTEX,
                    "vertex buffer top",
                ),
                index_buffer_top: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::INDEX,
                    "index buffer top",
                ),
                split_vbo_top: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::VERTEX,
                    "split vbo top",
                ),
                split_ibo_top: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::INDEX,
                    "split ibo top",
                ),
                split_vbo_bottom: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::VERTEX,
                    "split vbo bottom",
                ),
                split_ibo_bottom: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::INDEX,
                    "split ibo bottom",
                ),
                vertex_buffer_bottom: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::VERTEX,
                    "vertex buffer bottom",
                ),
                index_buffer_bottom: DynamicBuffer::new(
                    device,
                    wgpu::BufferUsages::INDEX,
                    "index buffer bottom",
                ),
                num_instance_top: 0,
                num_instance_bottom: 0,
                num_instance_split_top: 0,
                num_instance_split_bottom: 0,
            };

            if let Self::Prepared {
                vertex_buffer_top,
                index_buffer_top,
                num_instance_top,
                vertex_buffer_bottom,
                index_buffer_bottom,
                num_instance_bottom,
                split_vbo_top,
                split_ibo_top,
                num_instance_split_top,
                split_vbo_bottom,
                split_ibo_bottom,
                num_instance_split_bottom,
            } = self
            {
                vertex_buffer_top.update(top.vertices.as_slice());
                index_buffer_top.update(top.indices.as_slice());
                *num_instance_top = top.indices.len() as u32;
                split_vbo_top.update(split_top.vertices.as_slice());
                split_ibo_top.update(split_top.indices.as_slice());
                *num_instance_split_top = split_top.indices.len() as u32;
                vertex_buffer_bottom.update(bottom.vertices.as_slice());
                index_buffer_bottom.update(bottom.indices.as_slice());
                *num_instance_bottom = bottom.indices.len() as u32;
                split_vbo_bottom.update(split_bottom.vertices.as_slice());
                split_ibo_bottom.update(split_bottom.indices.as_slice());
                *num_instance_split_bottom = split_bottom.indices.len() as u32;
            }
        }

        if let Self::Prepared {
            vertex_buffer_top,
            index_buffer_top,
            vertex_buffer_bottom,
            index_buffer_bottom,
            split_vbo_top,
            split_ibo_top,
            split_vbo_bottom,
            split_ibo_bottom,
            ..
        } = self
        {
            vertex_buffer_top.prepare(device, queue);
            index_buffer_top.prepare(device, queue);
            vertex_buffer_bottom.prepare(device, queue);
            index_buffer_bottom.prepare(device, queue);
            split_vbo_top.prepare(device, queue);
            split_ibo_top.prepare(device, queue);
            split_vbo_bottom.prepare(device, queue);
            split_ibo_bottom.prepare(device, queue);
        }
    }

    pub(super) fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, bottom: bool) {
        if let Self::Prepared {
            vertex_buffer_top,
            index_buffer_top,
            num_instance_top,
            vertex_buffer_bottom,
            index_buffer_bottom,
            num_instance_bottom,
            ..
        } = self
        {
            if bottom {
                render_pass
                    .set_index_buffer(index_buffer_bottom.get_slice(), wgpu::IndexFormat::Uint16);
                render_pass.set_vertex_buffer(0, vertex_buffer_bottom.get_slice());
                render_pass.draw_indexed(0..*num_instance_bottom, 0, 0..1);
            } else {
                render_pass
                    .set_index_buffer(index_buffer_top.get_slice(), wgpu::IndexFormat::Uint16);
                render_pass.set_vertex_buffer(0, vertex_buffer_top.get_slice());
                render_pass.draw_indexed(0..*num_instance_top, 0, 0..1);
            }
        }
    }

    pub(super) fn draw_split<'a>(&'a self, render_pass: &mut RenderPass<'a>, bottom: bool) {
        if let Self::Prepared {
            split_vbo_top,
            split_ibo_top,
            num_instance_split_top,
            split_vbo_bottom,
            split_ibo_bottom,
            num_instance_split_bottom,
            ..
        } = self
        {
            if bottom {
                if *num_instance_split_bottom > 0 {
                    render_pass
                        .set_index_buffer(split_ibo_bottom.get_slice(), wgpu::IndexFormat::Uint16);
                    render_pass.set_vertex_buffer(0, split_vbo_bottom.get_slice());
                    render_pass.draw_indexed(0..*num_instance_split_bottom, 0, 0..1);
                }
            } else if *num_instance_split_top > 0 {
                render_pass.set_index_buffer(split_ibo_top.get_slice(), wgpu::IndexFormat::Uint16);
                render_pass.set_vertex_buffer(0, split_vbo_top.get_slice());
                render_pass.draw_indexed(0..*num_instance_split_top, 0, 0..1);
            }
        }
    }
}

pub(super) struct DynamicBuffer {
    bytes: Vec<u8>,
    buffer: Buffer,
    capacity: usize,
    length: usize,
    usage: wgpu::BufferUsages,
}

impl DynamicBuffer {
    pub(crate) fn new(device: &Device, usage: wgpu::BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: 0,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let capacity = 0;
        let length = 0;

        Self {
            bytes: vec![],
            buffer,
            capacity,
            length,
            usage,
        }
    }

    /// Uploads the buffer to the GPU.
    pub(crate) fn prepare(&mut self, device: &Device, queue: &Queue) {
        if self.capacity < self.bytes.len() {
            self.capacity = 2 * self.bytes.len();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("capacity = {}", self.capacity)),
                size: self.capacity as u64,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        self.length = self.bytes.len();
        queue.write_buffer(&self.buffer, 0, self.bytes.as_slice());
    }

    /// Replace the data of the associated buffer.
    pub(crate) fn update<I: bytemuck::Pod>(&mut self, data: &[I]) {
        let mut bytes: Vec<u8> = bytemuck::cast_slice(data).into();
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }

        self.bytes = bytes;
    }

    pub(crate) fn get_slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..self.length as u64)
    }
}
