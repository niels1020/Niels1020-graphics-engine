//TODO finish
use wgpu::{RenderPass, RenderPipeline};

use crate::render::{render_objects::RenderObject, texture::Texture};

#[allow(dead_code)]
pub struct Camera2D {
    to_render: Vec<Box<dyn RenderObject2D>>,
    render_pipeline: Option<RenderPipeline>,
    render_shader_name: String,
    name: String,
}

pub trait RenderObject2D {}

impl RenderObject for Camera2D {
    fn render(
        &mut self,
        _device: &wgpu::Device,
        _config: &wgpu::SurfaceConfiguration,
        _depth_texture: &Texture,
        _render_pass: &mut RenderPass,
    ) {
        //TODO: actual rendering
        todo!("actual rendering")
    }
}

impl Camera2D {
    pub fn new(shader: String, name: String) -> Box<Self> {
        Box::new(Self {
            to_render: vec![],
            render_pipeline: None,
            render_shader_name: shader,
            name,
        })
    }
}
