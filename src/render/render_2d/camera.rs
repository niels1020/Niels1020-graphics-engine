use wgpu::{
    BufferDescriptor, Device, FragmentState, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, SurfaceConfiguration, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    common::Vertex,
    render::{render_objects::RenderObject, texture::Texture},
};

#[allow(dead_code)]
pub struct Camera2D<'a> {
    to_render: Vec<Box<dyn RenderObject2D>>,
    render_pipeline: Option<RenderPipeline>,
    shader: ShaderModuleDescriptor<'a>,
    name: String,
}

pub trait RenderObject2D {}

impl<'a> RenderObject for Camera2D<'a> {
    fn render(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        _depth_texture: &Texture,
        render_pass: &mut RenderPass,
    ) {
        if self.render_pipeline.is_none() {
            self.render_pipeline = Some(create_pipeline(
                device,
                self.shader.clone(),
                config,
                &self.name,
            ));
        }

        //TODO: render to_rendere instead of some random shit
        let vertices = vec![
            Vertex::new(1.0, 0.0, 0.0, 0.0, 0.0),
            Vertex::new(1.0, 1.0, 0.0, 0.0, 0.0),
            Vertex::new(0.0, 1.0, 0.0, 0.0, 0.0),
            Vertex::new(-1.0, 0.0, 0.0, 0.0, 0.0),
            Vertex::new(-1.0, -1.0, 0.0, 0.0, 0.0),
            Vertex::new(0.0, -1.0, 0.0, 0.0, 0.0),
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor { label: Some("test buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, /*COPY_DST means that you can upload to it later*/});

        render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}

impl<'a> Camera2D<'a> {
    pub fn new(shader: ShaderModuleDescriptor<'a>, name: String) -> Box<Self> {
        Box::new(Self {
            to_render: vec![],
            render_pipeline: None,
            shader,
            name,
        })
    }
}

fn create_pipeline(
    device: &Device,
    shader: ShaderModuleDescriptor,
    config: &SurfaceConfiguration,
    name: &str,
) -> RenderPipeline {
    let module = device.create_shader_module(shader);

    let layout_desc = PipelineLayoutDescriptor {
        label: Some(&format!("({}) layout", name)),
        bind_group_layouts: &[], //TODO: add layouts for cam pos and text
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
            buffers: &[Vertex::desc()],
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
