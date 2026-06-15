use wgpu::{Device, RenderPass, SurfaceConfiguration};

use crate::render::texture::Texture;

pub trait RenderObject {
    fn render(
        &mut self,
        device: &Device,
        config: &SurfaceConfiguration,
        depth_texture: &Texture,
        render_pass: &mut RenderPass,
    );
}
