use wgpu::{
    BindGroup, BindGroupLayout, Buffer, Device, FragmentState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, SurfaceConfiguration, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    common::{CAMERA_BINDING, TEXTURE_BINDING, Vertex},
    render::{
        render_2d::camera::Camera2D,
        render_layers::RenderLayer,
        utils::{atlas::AtlasTexture, texture::Texture},
    },
};

#[allow(dead_code)]
pub struct RenderLayer2D {
    to_render: Vec<Box<dyn RenderObject2D + Send>>,
    render_pipeline: Option<RenderPipeline>,
    shader: ShaderModuleDescriptor<'static>,
    name: String,
    vertex_buffer: Option<Buffer>,
    vertices_len: usize,
    number_of_children_changed: bool,
    atlas_bind: Option<BindGroup>,

    atlas_texture: AtlasTexture,
    pub camera: Camera2D,
}

pub trait RenderObject2D {
    fn have_vertices_changed(&mut self) -> bool;
    fn get_vertices(&mut self, atlas: &mut AtlasTexture) -> Vec<Vertex>;
    fn get_name(&self) -> String;
}

impl RenderLayer for RenderLayer2D {
    fn render(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &Queue,
        _depth_texture: &Texture,
        render_pass: &mut RenderPass,
    ) {
        self.camera.update(device, queue);

        if self.render_pipeline.is_none() {
            self.render_pipeline = Some(create_pipeline(
                device,
                self.shader.clone(),
                config,
                &self.name,
                self.camera.layout.as_ref().unwrap(),
                &self.atlas_texture.create_layout(device),
            ));
            self.atlas_texture.build(queue, device);
        }

        let mut buffer_need_update = self.vertex_buffer.is_none() | self.number_of_children_changed;
        self.number_of_children_changed = false;
        for i in self.to_render.iter_mut() {
            if i.have_vertices_changed() {
                buffer_need_update = true;
            }
        }

        if buffer_need_update {
            let mut vertices = vec![];
            for i in self.to_render.iter_mut() {
                vertices.extend(i.get_vertices(&mut self.atlas_texture));
            }

            if vertices.is_empty() {
                self.vertex_buffer = None;
            } else if self.vertex_buffer.is_none() {
                self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor { label: Some("test buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, /*COPY_DST means that you can upload to it later*/}));
            } else {
                queue.write_buffer(
                    self.vertex_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&vertices),
                );
            }

            self.vertices_len = vertices.len();
        }

        let atlas_rebuilt = self.atlas_texture.build_if_needed(queue, device);
        if atlas_rebuilt {
            self.atlas_bind = Some(self.atlas_texture.bind(device));
        }

        if self.vertex_buffer.is_some() {
            render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
            render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_bind_group(CAMERA_BINDING, self.camera.bind.as_ref().unwrap(), &[]);
            render_pass.set_bind_group(TEXTURE_BINDING, self.atlas_bind.as_ref().unwrap(), &[]);
            render_pass.draw(0..self.vertices_len as u32, 0..1);
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl RenderLayer2D {
    pub fn new(
        shader: ShaderModuleDescriptor<'static>,
        name: String,
        camera: Camera2D,
    ) -> Box<Self> {
        Box::new(Self {
            to_render: vec![],
            render_pipeline: None,
            shader,
            name,
            vertex_buffer: None,
            number_of_children_changed: false,
            camera,
            atlas_texture: AtlasTexture::new(),
            vertices_len: 0,
            atlas_bind: None,
        })
    }

    pub fn get_child(&mut self, name: String) -> Option<&Box<dyn RenderObject2D + Send>> {
        self.to_render.iter().find(|a| a.get_name() == name)
    }

    pub fn get_mut_child(&mut self, name: String) -> Option<&mut Box<dyn RenderObject2D + Send>> {
        self.to_render.iter_mut().find(|a| a.get_name() == name)
    }

    pub fn remove_child(&mut self, name: String) {
        self.number_of_children_changed = true;
        let pos = self.to_render.iter().position(|a| a.get_name() == name);
        if let Some(pos) = pos {
            self.to_render.remove(pos);
        }
    }

    pub fn add_child(&mut self, child: Box<dyn RenderObject2D + Send>) {
        self.to_render.push(child);
        self.number_of_children_changed = true;
    }
}

fn create_pipeline(
    device: &Device,
    shader: ShaderModuleDescriptor<'static>,
    config: &SurfaceConfiguration,
    name: &str,
    camera_layout: &BindGroupLayout,
    atlas_layout: &BindGroupLayout,
) -> RenderPipeline {
    let module = device.create_shader_module(shader);

    let layout_desc = PipelineLayoutDescriptor {
        label: Some(&format!("({}) layout", name)),
        bind_group_layouts: &[Some(camera_layout), Some(atlas_layout)],
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
                blend: Some(wgpu::BlendState::REPLACE),
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
