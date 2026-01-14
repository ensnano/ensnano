use ensnano_design::external_3d_objects::{External3DObject, External3DObjectId};
use ensnano_utils::{
    TEXTURE_FORMAT,
    consts::SAMPLE_COUNT,
    create_buffer_with_data,
    obj_loader::{GltfMesh, ModelVertex, load_gltf, load_stl},
    surfaces::UnrootedRevolutionSurfaceDescriptor,
    texture::Texture,
};
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    rc::Rc,
};
use wgpu::{BindGroupLayoutDescriptor, Device};

struct DesiredRevolutionShapeDrawer {
    shape: UnrootedRevolutionSurfaceDescriptor,
    drawer: GltfDrawer,
}

pub struct Object3DDrawer {
    gltf_drawers: BTreeMap<External3DObjectId, GltfDrawer>,
    stl_drawers: BTreeMap<External3DObjectId, StlDrawer>,
    device: Rc<Device>,
    desired_revolution_shape_drawer: Option<DesiredRevolutionShapeDrawer>,
}

impl Object3DDrawer {
    pub fn new(device: Rc<Device>) -> Self {
        Self {
            gltf_drawers: Default::default(),
            stl_drawers: Default::default(),
            device,
            desired_revolution_shape_drawer: None,
        }
    }

    pub fn draw<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        viewer_bind_group: &'a wgpu::BindGroup,
    ) {
        for d in self.gltf_drawers.values_mut() {
            d.draw(render_pass, viewer_bind_group);
        }
        for d in self.stl_drawers.values_mut() {
            d.draw(render_pass, viewer_bind_group);
        }
        if let Some(ref mut d) = self.desired_revolution_shape_drawer {
            d.drawer.draw(render_pass, viewer_bind_group);
        }
    }

    pub fn update_objects(
        &mut self,
        objects: ExternalObjects,
        bg_desc: &BindGroupLayoutDescriptor,
    ) {
        for (obj_id, object) in objects.objects {
            if !self.stl_drawers.contains_key(&obj_id) && !self.gltf_drawers.contains_key(&obj_id) {
                self.add_object(obj_id, object, &objects.path_base, bg_desc);
            }
        }
    }

    fn add_object(
        &mut self,
        id: External3DObjectId,
        object: External3DObject,
        base_path: &PathBuf,
        bg_desc: &BindGroupLayoutDescriptor,
    ) {
        let path = object.get_path_to_source_file(base_path);
        println!("{}", path.display());
        if path.extension() == Some(OsStr::new("stl")) {
            let mut drawer = StlDrawer::new(self.device.as_ref(), bg_desc);
            drawer.add_stl(self.device.as_ref(), path);
            self.stl_drawers.insert(id, drawer);
        } else if path.extension() == Some(OsStr::new("gltf")) {
            let mut drawer = GltfDrawer::new(self.device.as_ref(), bg_desc);
            drawer.add_gltf(self.device.as_ref(), path);
            self.gltf_drawers.insert(id, drawer);
        }
    }

    pub fn update_desired_revolution_shape(
        &mut self,
        shape: Option<UnrootedRevolutionSurfaceDescriptor>,
        device: &Device,
        view_bg_layout_desc: &BindGroupLayoutDescriptor,
    ) -> bool {
        if self
            .desired_revolution_shape_drawer
            .as_ref()
            .map(|d| &d.shape)
            == shape.as_ref()
        {
            false
        } else {
            let new_drawer = shape.map(|shape| {
                let mut drawer = GltfDrawer::new(device, view_bg_layout_desc);
                let meshes = shape.meshes();
                drawer.set_meshes(device, meshes);
                DesiredRevolutionShapeDrawer { shape, drawer }
            });
            self.desired_revolution_shape_drawer = new_drawer;
            true
        }
    }

    pub fn clear(&mut self) {
        self.gltf_drawers = Default::default();
        self.stl_drawers = Default::default();
        self.desired_revolution_shape_drawer = None;
    }
}

#[derive(Debug)]
pub struct ExternalObjects {
    pub path_base: PathBuf,
    pub objects: Vec<(External3DObjectId, External3DObject)>,
}

pub struct GltfDrawer {
    vbos: Vec<wgpu::Buffer>,
    ibos: Vec<wgpu::Buffer>,
    nb_idx: Vec<u32>,
    render_pipeline: wgpu::RenderPipeline,
}

impl GltfDrawer {
    pub fn new(device: &Device, view_bg_layout_desc: &BindGroupLayoutDescriptor) -> Self {
        let primitive_topology = wgpu::PrimitiveTopology::TriangleStrip;
        let render_pipeline =
            build_render_pipeline(device, view_bg_layout_desc, primitive_topology);

        Self {
            render_pipeline,
            vbos: vec![],
            ibos: vec![],
            nb_idx: vec![],
        }
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        viewer_bind_group: &'a wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, viewer_bind_group, &[]);
        for i in 0..self.vbos.len() {
            render_pass.set_vertex_buffer(0, self.vbos[i].slice(..));
            render_pass.set_index_buffer(self.ibos[i].slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.nb_idx[i], 0, 0..1);
        }
    }

    pub fn add_gltf<P: AsRef<Path>>(&mut self, device: &Device, path: P) {
        match load_gltf(path) {
            Ok(file) => {
                self.set_meshes(device, file.meshes);
            }
            Err(err) => {
                log::error!("Could not read gltf file: {err:?}");
            }
        }
    }

    pub fn set_meshes(&mut self, device: &Device, meshes: Vec<GltfMesh>) {
        self.nb_idx.clear();
        self.vbos.clear();
        self.ibos.clear();
        for mesh in meshes {
            self.nb_idx.push(mesh.indices.len() as u32);
            self.vbos.push(create_buffer_with_data(
                device,
                bytemuck::cast_slice(mesh.vertices.as_slice()),
                wgpu::BufferUsages::VERTEX,
                "gltf vertex",
            ));
            self.ibos.push(create_buffer_with_data(
                device,
                bytemuck::cast_slice(mesh.indices.as_slice()),
                wgpu::BufferUsages::INDEX,
                "gltf index",
            ));
        }
    }
}

pub struct StlDrawer {
    vbos: Vec<wgpu::Buffer>,
    nb_idx: Vec<u32>,
    render_pipeline: wgpu::RenderPipeline,
}

impl StlDrawer {
    pub fn new(device: &Device, view_bg_layout_desc: &BindGroupLayoutDescriptor) -> Self {
        let primitive_topology = wgpu::PrimitiveTopology::TriangleList;
        let render_pipeline =
            build_render_pipeline(device, view_bg_layout_desc, primitive_topology);

        Self {
            render_pipeline,
            vbos: vec![],
            nb_idx: vec![],
        }
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        viewer_bind_group: &'a wgpu::BindGroup,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, viewer_bind_group, &[]);
        for i in 0..self.vbos.len() {
            render_pass.set_vertex_buffer(0, self.vbos[i].slice(..));
            render_pass.draw(0..self.nb_idx[i], 0..1);
        }
    }

    pub fn add_stl<P: AsRef<Path>>(&mut self, device: &Device, path: P) {
        match load_stl(path) {
            Ok(mesh) => {
                self.nb_idx.push(mesh.vertices.len() as u32);
                self.vbos.push(create_buffer_with_data(
                    device,
                    bytemuck::cast_slice(mesh.vertices.as_slice()),
                    wgpu::BufferUsages::VERTEX,
                    "std vertex",
                ));
            }
            Err(err) => {
                log::error!("Could not read stl file: {err:?}");
            }
        }
    }
}

fn build_render_pipeline(
    device: &Device,
    view_bg_layout_desc: &BindGroupLayoutDescriptor,
    primitive_topology: wgpu::PrimitiveTopology,
) -> wgpu::RenderPipeline {
    let viewer_bg_layout = device.create_bind_group_layout(view_bg_layout_desc);

    let vertex_module = device.create_shader_module(wgpu::include_spirv!("gltf_obj.vert.spv"));
    let fragment_module = device.create_shader_module(wgpu::include_spirv!("gltf_obj.frag.spv"));
    let format = TEXTURE_FORMAT;
    let blend_state = wgpu::BlendState::ALPHA_BLENDING;
    let sample_count = SAMPLE_COUNT;

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Gltf Drawer"),
        bind_group_layouts: &[&viewer_bg_layout],
        push_constant_ranges: &[],
    });

    let strip_index_format = match primitive_topology {
        wgpu::PrimitiveTopology::LineStrip | wgpu::PrimitiveTopology::TriangleStrip => {
            Some(wgpu::IndexFormat::Uint32)
        }
        _ => None,
    };

    let primitive = wgpu::PrimitiveState {
        topology: primitive_topology,
        strip_index_format,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        ..Default::default()
    };

    let targets = &[Some(wgpu::ColorTargetState {
        format,
        blend: Some(blend_state),
        write_mask: wgpu::ColorWrites::ALL,
    })];

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_module,
            entry_point: "main",
            buffers: &[ModelVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_module,
            entry_point: "main",
            targets,
        }),
        primitive,
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: true,
        },
        label: Some("Gltf drawer pipeline"),
        multiview: None,
    })
}
