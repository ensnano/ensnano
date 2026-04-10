use crate::{
    CameraPtr,
    data::{
        helix::Helix,
        strand::{FreeEnd, Strand},
    },
    flat_types::FlatNucl,
};
use std::rc::Rc;
use wgpu::{Buffer, Device, Queue, RenderPass};

pub(super) struct HelixView {
    vertex_buffer: DynamicBuffer,
    index_buffer: DynamicBuffer,
    num_instance: u32,
    background: bool,
}

impl HelixView {
    pub(super) fn new(device: Rc<Device>, queue: Rc<Queue>, background: bool) -> Self {
        Self {
            vertex_buffer: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::VERTEX,
                "helix vertex buffer",
            ),
            index_buffer: DynamicBuffer::new(
                device,
                queue,
                wgpu::BufferUsages::INDEX,
                "helix index buffer",
            ),
            num_instance: 0,
            background,
        }
    }

    pub(super) fn update(&mut self, helix: &Helix) {
        let vertices = if self.background {
            helix.background_vertices()
        } else {
            helix.to_vertices()
        };
        self.vertex_buffer.update(vertices.vertices.as_slice());
        self.index_buffer.update(vertices.indices.as_slice());
        self.num_instance = vertices.indices.len() as u32;
    }

    pub(super) fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        if self.index_buffer.length == 0 || self.num_instance == 0 || self.vertex_buffer.length == 0
        {
            println!(
                "[[Bug in ensnano_flatscene::view::helix_view::HelixView::draw]]: should not be empty\n\tindex_buffer={} num_instance={} vertex_buffer={}",
                self.index_buffer.length, self.num_instance, self.index_buffer.length
            );
            return;
        }
        render_pass.set_index_buffer(self.index_buffer.get_slice(), wgpu::IndexFormat::Uint16);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.get_slice());
        render_pass.draw_indexed(0..self.num_instance, 0, 0..1);
    }
}

pub(super) struct StrandView {
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
}

impl StrandView {
    pub(super) fn new(device: Rc<Device>, queue: Rc<Queue>) -> Self {
        Self {
            vertex_buffer_top: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::VERTEX,
                "vertex buffer top",
            ),
            index_buffer_top: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::INDEX,
                "index buffer top",
            ),
            split_vbo_top: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::VERTEX,
                "split vbo top",
            ),
            split_ibo_top: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::INDEX,
                "split ibo top",
            ),
            split_vbo_bottom: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::VERTEX,
                "split vbo bottom",
            ),
            split_ibo_bottom: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::INDEX,
                "split ibo bottom",
            ),
            vertex_buffer_bottom: DynamicBuffer::new(
                device.clone(),
                queue.clone(),
                wgpu::BufferUsages::VERTEX,
                "vertex buffer bottom",
            ),
            index_buffer_bottom: DynamicBuffer::new(
                device,
                queue,
                wgpu::BufferUsages::INDEX,
                "index buffer bottom",
            ),
            num_instance_top: 0,
            num_instance_bottom: 0,
            num_instance_split_top: 0,
            num_instance_split_bottom: 0,
        }
    }

    pub(super) fn update(
        &mut self,
        strand: &Strand,
        helices: &[Helix],
        free_end: Option<&FreeEnd>,
        top_cam: &CameraPtr,
        bottom_cam: &CameraPtr,
    ) {
        // TODO: check if update is needed

        let (vertices_top, split_vertices_top) =
            strand.to_vertices(helices, free_end, top_cam, bottom_cam);
        self.vertex_buffer_top
            .update(vertices_top.vertices.as_slice());
        self.index_buffer_top
            .update(vertices_top.indices.as_slice());
        self.num_instance_top = vertices_top.indices.len() as u32;
        self.split_vbo_top
            .update(split_vertices_top.vertices.as_slice());
        self.split_ibo_top
            .update(split_vertices_top.indices.as_slice());
        self.num_instance_split_top = split_vertices_top.indices.len() as u32;
        let (vertices_bottom, split_vertices_bottom) =
            strand.to_vertices(helices, free_end, bottom_cam, top_cam);
        self.vertex_buffer_bottom
            .update(vertices_bottom.vertices.as_slice());
        self.index_buffer_bottom
            .update(vertices_bottom.indices.as_slice());
        self.num_instance_bottom = vertices_bottom.indices.len() as u32;
        self.split_vbo_bottom
            .update(split_vertices_bottom.vertices.as_slice());
        self.split_ibo_bottom
            .update(split_vertices_bottom.indices.as_slice());
        self.num_instance_split_bottom = split_vertices_bottom.indices.len() as u32;
    }

    pub(super) fn set_indication(&mut self, nucl1: FlatNucl, nucl2: FlatNucl, helices: &[Helix]) {
        let vertices = Strand::indication(nucl1, nucl2, helices);
        self.vertex_buffer_top.update(vertices.vertices.as_slice());
        self.index_buffer_top.update(vertices.indices.as_slice());
        self.num_instance_top = vertices.indices.len() as u32;
        self.vertex_buffer_bottom
            .update(vertices.vertices.as_slice());
        self.index_buffer_bottom.update(vertices.indices.as_slice());
        self.num_instance_bottom = vertices.indices.len() as u32;
    }

    pub(super) fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, bottom: bool) {
        if bottom {
            render_pass.set_index_buffer(
                self.index_buffer_bottom.get_slice(),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.set_vertex_buffer(0, self.vertex_buffer_bottom.get_slice());
            render_pass.draw_indexed(0..self.num_instance_bottom, 0, 0..1);
        } else {
            render_pass
                .set_index_buffer(self.index_buffer_top.get_slice(), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer_top.get_slice());
            render_pass.draw_indexed(0..self.num_instance_top, 0, 0..1);
        }
    }

    pub(super) fn draw_split<'a>(&'a self, render_pass: &mut RenderPass<'a>, bottom: bool) {
        if bottom {
            if self.num_instance_split_bottom > 0 {
                render_pass
                    .set_index_buffer(self.split_ibo_bottom.get_slice(), wgpu::IndexFormat::Uint16);
                render_pass.set_vertex_buffer(0, self.split_vbo_bottom.get_slice());
                render_pass.draw_indexed(0..self.num_instance_split_bottom, 0, 0..1);
            }
        } else if self.num_instance_split_top > 0 {
            render_pass.set_index_buffer(self.split_ibo_top.get_slice(), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.split_vbo_top.get_slice());
            render_pass.draw_indexed(0..self.num_instance_split_top, 0, 0..1);
        }
    }
}

struct DynamicBuffer {
    buffer: Buffer,
    capacity: usize,
    length: u64,
    device: Rc<Device>,
    queue: Rc<Queue>,
    usage: wgpu::BufferUsages,
}

impl DynamicBuffer {
    pub(crate) fn new(
        device: Rc<Device>,
        queue: Rc<Queue>,
        usage: wgpu::BufferUsages,
        label: &str,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: 0,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let capacity = 0;
        let length = 0;

        Self {
            buffer,
            capacity,
            length,
            device,
            queue,
            usage,
        }
    }

    /// Replace the data of the associated buffer.
    pub(crate) fn update<I: bytemuck::Pod>(&mut self, data: &[I]) {
        let mut bytes: Vec<u8> = bytemuck::cast_slice(data).into();
        let length = bytes.len();
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        if self.capacity < bytes.len() {
            self.length = length as u64;
            self.buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("capacity = {}", 2 * bytes.len())),
                size: 2 * bytes.len() as u64,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.capacity = 2 * bytes.len();
        } else if self.length != length as u64 {
            self.length = length as u64;
        }
        self.queue.write_buffer(&self.buffer, 0, bytes.as_slice());
    }

    pub(crate) fn get_slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..self.length)
    }
}
