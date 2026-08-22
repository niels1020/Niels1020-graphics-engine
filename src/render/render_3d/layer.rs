use std::any::Any;

use bytemuck::{NoUninit, cast_slice};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, BufferBindingType, BufferUsages, Device, FragmentState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderStages, SurfaceConfiguration, VertexState, hal::auxil::db::qualcomm, include_wgsl, util::{BufferInitDescriptor, DeviceExt}, wgc::device::queue,
};

use crate::{
    common::Vertex,
    render::{
        render_3d::camera::{Camera3D, Camera3DUniform},
        render_layers::RenderLayer,
        utils::{atlas::AtlasTexture, global::RendererGlobal, texture::Texture},
    },
};

#[allow(dead_code)]
pub struct RenderLayer3D {
    to_render: Vec<RenderObject3DContainer>,
    render_pipeline: Option<RenderPipeline>,
    shader: ShaderModuleDescriptor<'static>,
    name: String,
    number_of_children_changed: bool,

    atlas_bind: Option<BindGroup>,
    atlas_texture: AtlasTexture,
    atlas_rebuilt: bool,

    pub camera: Camera3D,
    camera_bind: Option<BindGroup>,
    camera_buffer: Option<Buffer>,
    camera_uniform: Camera3DUniform,

    transform_layout: Option<BindGroupLayout>,
}

pub trait RenderObject3D {
    fn have_vertices_changed(&self) -> bool;
    ///gets called when have_vertices_changed of any object returns true or when the atlas has been rebuild
    fn get_vertices(&mut self, global: &mut RenderLayer3DGlobal) -> Vec<Vertex>;
    fn get_name(&self) -> String;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_transform(&self) -> Transform;
    fn has_transform_changed(&self) -> bool;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, NoUninit)]
pub struct Transform {
    pub position: [f32; 3],
    _pad1: f32,
    pub rotation: [f32; 3], //degrees
    _pad2: f32,
}

impl Transform {
    pub fn new(position: [f32; 3], rotation /*degrees */: [f32; 3]) -> Self {
        Self {
            position,
            _pad1: 0.0,
            rotation,
            _pad2: 0.0,
        }
    }
}

pub fn object3d_as_type_mut<T: RenderObject3D + 'static>(
    object2d: &mut Box<dyn RenderObject3D>,
) -> Option<&mut T> {
    object2d.as_any_mut().downcast_mut::<T>()
}

impl RenderLayer for RenderLayer3D {
    fn render(&mut self, global: &mut RendererGlobal, render_pass: &mut RenderPass) {
        self.camera_uniform.update_view_proj(&self.camera);

        if self.render_pipeline.is_none() {
            let cam_bind_group_layout = camera_bind_group_layout(&global.device);
            self.camera_buffer = Some(global.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[self.camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            ));

            self.transform_layout = Some(transform_bind_group_layout(&global.device));

            self.render_pipeline = Some(create_pipeline(
                &global.device,
                self.shader.clone(),
                &global.config,
                &self.name,
                &cam_bind_group_layout,
                &self.atlas_texture.create_layout(&global.device),
                self.transform_layout.as_ref().unwrap(),
            ));

            self.camera_bind = Some(create_camera_bind_group(
                &global.device,
                self.camera_buffer.as_ref().unwrap(),
                &cam_bind_group_layout,
            ));
            self.atlas_texture.build(&global.queue, &global.device);
            self.atlas_bind = Some(self.atlas_texture.bind(&global.device))
        }

        self.camera_uniform.update_view_proj(&self.camera);
        global.queue.write_buffer(self.camera_buffer.as_ref().unwrap(), 0, cast_slice(&[self.camera_uniform]));

        render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
        render_pass.set_bind_group(0, self.camera_bind.as_ref().unwrap(), &[]);

        for container in self.to_render.iter_mut() {
            if container.vertex_buffer.is_none() || container.object().have_vertices_changed() {
                let vertices = container
                    .object_mut()
                    .get_vertices(&mut RenderLayer3DGlobal {
                        renderer_global: global,
                        atlas_texture: &mut self.atlas_texture,
                    });

                container.vertices_len = vertices.len();

                if container.vertices_len == 0 {
                    continue;
                }

                container.vertex_buffer =
                    Some(global.device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("Vertex beffer for an object"),
                        contents: cast_slice(&vertices),
                        usage: BufferUsages::VERTEX,
                    }));
            }

            if container.transform_buffer.is_none() {
                container.transform_buffer = Some(global.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("A Transform Buffer"),
                        contents: bytemuck::cast_slice(&[container.object_mut().get_transform()]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, //COPY_DST means that you can upload to it later
                    },
                ));
            }

            if container.transform_bind.is_none() {
                container.transform_bind = Some(
                    global.device.create_bind_group(&BindGroupDescriptor {
                        label: Some("transform bind"),
                        layout: self.transform_layout.as_ref().unwrap(),
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: container
                                .transform_buffer
                                .as_ref()
                                .unwrap()
                                .as_entire_binding(),
                        }],
                    }),
                )
            }

            if container.object().has_transform_changed() {
                let transform = container.object_mut().get_transform();
                global.queue.write_buffer(
                    container.transform_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&[transform]),
                );
            }

            self.atlas_rebuilt = self
                .atlas_texture
                .build_if_needed(&global.queue, &global.device);
            if self.atlas_rebuilt {
                self.atlas_bind = Some(self.atlas_texture.bind(&global.device));
            }

            render_pass.set_bind_group(1, self.atlas_bind.as_ref().unwrap(), &[]);
            render_pass.set_bind_group(2, container.transform_bind.as_ref().unwrap(), &[]);
            render_pass.set_vertex_buffer(0, container.vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.draw(0..container.vertices_len as u32, 0..1);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl RenderLayer3D {
    ///if shader == None it wil use a default shader
    pub fn new(
        shader: Option<ShaderModuleDescriptor<'static>>,
        name: String,
        camera: Camera3D,
    ) -> Box<Self> {
        let atlas_texture = AtlasTexture::new();

        let shader = match shader {
            Some(s) => s,
            None => include_wgsl!("../../../assets/test/3d.wgsl"),
        };

        Box::new(Self {
            to_render: vec![],
            render_pipeline: None,
            shader,
            name,
            number_of_children_changed: false,
            camera,
            atlas_texture,
            atlas_bind: None,
            atlas_rebuilt: false,
            camera_bind: None,
            camera_buffer: None,
            camera_uniform: Camera3DUniform::new(),
            transform_layout: None,
        })
    }

    pub fn get_child(&mut self, name: String) -> Option<&Box<dyn RenderObject3D + Send>> {
        match self
            .to_render
            .iter()
            .find(|a| a.object().get_name() == name)
        {
            Some(s) => Some(s.object()),
            None => None,
        }
    }

    pub fn get_mut_child(&mut self, name: String) -> Option<&mut Box<dyn RenderObject3D + Send>> {
        match self
            .to_render
            .iter_mut()
            .find(|a| a.object().get_name() == name)
        {
            Some(s) => Some(s.object_mut()),
            None => None,
        }
    }

    pub fn remove_child(&mut self, name: String) {
        self.number_of_children_changed = true;
        let pos = self
            .to_render
            .iter()
            .position(|a| a.object().get_name() == name);
        if let Some(pos) = pos {
            self.to_render.remove(pos);
        }
    }

    pub fn add_child(&mut self, child: Box<dyn RenderObject3D + Send>) {
        self.to_render.push(child.into());
        self.number_of_children_changed = true;
    }
}

fn camera_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("camera_bind_group_layout"),
    })
}

fn transform_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("transform layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_camera_bind_group(
    device: &Device,
    camera_buffer: &wgpu::Buffer,
    layout: &BindGroupLayout,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    })
}

fn create_pipeline(
    device: &Device,
    shader: ShaderModuleDescriptor<'static>,
    config: &SurfaceConfiguration,
    name: &str,
    camera_layout: &BindGroupLayout,
    atlas_layout: &BindGroupLayout,
    transform_layout: &BindGroupLayout,
) -> RenderPipeline {
    let module = device.create_shader_module(shader);

    let layout_desc = PipelineLayoutDescriptor {
        label: Some(&format!("({}) layout", name)),
        bind_group_layouts: &[
            Some(camera_layout),
            Some(atlas_layout),
            Some(transform_layout),
        ],
        immediate_size: 0,
    };

    let layout = device.create_pipeline_layout(&layout_desc);

    let desc: RenderPipelineDescriptor<'_> = RenderPipelineDescriptor {
        label: Some(&format!("({}) render pipeline", name)),
        layout: Some(&layout),
        vertex: VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[Some(Vertex::desc())],
        },
        fragment: Some(FragmentState {
            module: &module,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Front),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    };

    device.create_render_pipeline(&desc)
}

pub struct RenderLayer3DGlobal<'a> {
    pub renderer_global: &'a mut RendererGlobal,
    pub atlas_texture: &'a mut AtlasTexture,
}

pub struct RenderObject3DContainer {
    object: Box<dyn RenderObject3D + Send>,
    transform_bind: Option<BindGroup>,
    vertex_buffer: Option<Buffer>,
    transform_buffer: Option<Buffer>,
    vertices_len: usize,
}

impl From<Box<dyn RenderObject3D + Send>> for RenderObject3DContainer {
    fn from(value: Box<dyn RenderObject3D + Send>) -> Self {
        Self {
            object: value,
            transform_bind: None,
            vertex_buffer: None,
            transform_buffer: None,
            vertices_len: 0,
        }
    }
}

impl RenderObject3DContainer {
    pub fn object_mut(&mut self) -> &mut Box<dyn RenderObject3D + Send> {
        &mut self.object
    }

    pub fn object(&self) -> &Box<dyn RenderObject3D + Send> {
        &self.object
    }
}
