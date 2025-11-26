use crate::{
    ensnano_scene::view::instances_drawer::{Instantiable, ResourceProvider, Vertexable},
    ensnano_utils::text::{self, Letter},
};
use ultraviolet::{Vec2, Vec3, Vec4};
use wgpu::{Device, include_spirv};

#[derive(Debug, Clone)]
pub struct LetterInstance {
    pub position: Vec3,
    pub color: Vec4,
    pub design_id: u32,
    pub scale: f32,
    pub shift: Vec3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct RawLetter {
    pub position: Vec3,
    pub design_id: u32,
    pub color: Vec4,
    pub shift: Vec3,
    pub scale: f32,
}

impl ResourceProvider for Letter {
    fn resources_layout() -> &'static [wgpu::BindGroupLayoutEntry] {
        &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }

    /// This methods allows the resource tho provide the vertex buffer. If the return value is
    /// Some, it takes priority over the Instantiable's vertices.
    fn vertex_buffer_desc() -> Option<wgpu::VertexBufferLayout<'static>>
    where
        Self: Sized,
    {
        Some(text::Vertex::desc())
    }

    fn resources(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
        ]
    }

    fn vertex_buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.vertex_buffer)
    }

    fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.index_buffer)
    }
}

#[repr(C)]
#[derive(Clone, Debug, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LetterVertex {
    pub position: Vec2,
}

impl Vertexable for LetterVertex {
    type RawType = Self;

    fn to_raw(&self) -> Self {
        *self
    }

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        }
    }
}

impl Instantiable for LetterInstance {
    type Resource = Letter;
    type Vertex = LetterVertex;
    type RawInstance = RawLetter;

    fn to_raw_instance(&self) -> RawLetter {
        RawLetter {
            position: self.position,
            color: self.color,
            design_id: self.design_id,
            scale: self.scale,
            shift: self.shift,
        }
    }

    fn vertices() -> Vec<LetterVertex> {
        vec![
            LetterVertex {
                position: Vec2::new(0f32, 0f32),
            },
            LetterVertex {
                position: Vec2::new(0f32, 1f32),
            },
            LetterVertex {
                position: Vec2::new(1f32, 0f32),
            },
            LetterVertex {
                position: Vec2::new(1f32, 1f32),
            },
        ]
    }

    fn indices() -> Vec<u16> {
        vec![0, 1, 2, 3]
    }

    fn primitive_topology() -> wgpu::PrimitiveTopology {
        wgpu::PrimitiveTopology::TriangleStrip
    }

    fn vertex_module(device: &Device) -> wgpu::ShaderModule {
        device.create_shader_module(include_spirv!("letter.vert.spv"))
    }

    fn fragment_module(device: &Device) -> wgpu::ShaderModule {
        device.create_shader_module(include_spirv!("letter.frag.spv"))
    }
}
