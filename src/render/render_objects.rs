use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};

use crate::render::texture::Texture;

pub trait RenderObject {
    fn render(
        &mut self,
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
        depth_texture: &Texture,
        render_pass: &mut RenderPass,
    );
}
