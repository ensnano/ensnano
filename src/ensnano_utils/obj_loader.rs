use std::{fs::File, io::BufReader, path::Path};
use ultraviolet::Vec3;

const OBJ_VERTEX_ARRAY: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4];

const DEFAULT_STL_COLOR: [f32; 4] = [0., 0.5, 1., 0.9];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl ModelVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &OBJ_VERTEX_ARRAY,
        }
    }
}

pub struct GltfFile {
    pub meshes: Vec<GltfMesh>,
}

pub struct GltfMesh {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

fn read_mesh(mesh_data: &gltf::Mesh, data: &[gltf::buffer::Data]) -> Result<GltfMesh, ErrGltf> {
    let primitive = mesh_data.primitives().next().ok_or(ErrGltf::NoPrimitive)?;
    let reader = primitive.reader(|b| Some(&data.get(b.index())?.0[..b.length()]));

    let vertex_positions = { reader.read_positions().ok_or(ErrGltf::NoPosition)? };
    let vertex_normals = { reader.read_normals().ok_or(ErrGltf::NoNormal)? };
    let vertex_colors = {
        let color_iter = reader.read_colors(0).ok_or(ErrGltf::NoColor)?;
        color_iter.into_rgba_u8().map(|v| {
            [
                v[0] as f32 / 255.,
                v[1] as f32 / 255.,
                v[2] as f32 / 255.,
                v[3] as f32 / 255.,
            ]
        })
    };
    let indices = reader.read_indices().unwrap().into_u32().collect();

    let vertices: Vec<ModelVertex> = vertex_positions
        .zip(vertex_normals.zip(vertex_colors))
        .map(|(position, (normal, color))| ModelVertex {
            position,
            // position: [5.0*position[0], 5.0*position[1], 5.0*position[2], ], // UGLY HARDCODING OF STL UPSCALING BY 5.0
            normal,
            color,
        })
        .collect();

    Ok(GltfMesh { vertices, indices })
}

pub fn load_gltf<P: AsRef<Path>>(path: P) -> Result<GltfFile, ErrGltf> {
    let (doc, data, _) = gltf::import(path).ok().ok_or(ErrGltf::CannotReadFile)?;
    let mesh_data = doc.meshes();
    let mut meshes = Vec::new();
    for m in mesh_data {
        let mesh = read_mesh(&m, &data)?;
        meshes.push(mesh);
    }
    Ok(GltfFile { meshes })
}

#[derive(Debug)]
pub enum ErrGltf {
    CannotReadFile,
    NoPrimitive,
    NoColor,
    NoNormal,
    NoPosition,
}

pub struct StlMesh {
    pub vertices: Vec<ModelVertex>,
}

pub fn load_stl<P: AsRef<Path>>(path: P) -> Result<StlMesh, ErrStl> {
    let color = DEFAULT_STL_COLOR; //[0.55, 0.20, 0.25, 1.];
    let file = File::open(path).map_err(ErrStl::FileErr)?;
    let mut stl_buff = BufReader::new(&file);
    let mesh = nom_stl::parse_stl(&mut stl_buff).map_err(ErrStl::StlParseErr)?;
    let mut vertices = Vec::new();
    for t in mesh.triangles() {
        let normal = (Vec3::from(t.vertices()[0]) - Vec3::from(t.vertices()[1]))
            .cross(Vec3::from(t.vertices()[1]) - Vec3::from(t.vertices()[2]));
        log::trace!("normal: {normal:?}");
        for v in t.vertices() {
            vertices.push(ModelVertex {
                color,
                position: [v[0] / 10., v[1] / 10., v[2] / 10.], // scale by 10 Å = 1 nm
                normal: normal.into(),
            });
        }
    }
    Ok(StlMesh { vertices })
}

#[derive(Debug)]
pub enum ErrStl {
    FileErr(#[expect(unused)] std::io::Error),
    StlParseErr(#[expect(unused)] nom_stl::Error),
}
