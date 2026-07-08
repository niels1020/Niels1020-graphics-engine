use std::any::Any;

use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};

use crate::render::texture::Texture;

pub trait RenderLayer {
    fn render(
        &mut self,
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
        depth_texture: &Texture,
        render_pass: &mut RenderPass,
    );

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
