use cosmic_text::{FontSystem, SwashCache};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::render::utils::texture::Texture;

pub struct RendererGlobal {
    pub device: Device,
    pub config: SurfaceConfiguration,
    pub queue: Queue,
    pub depth_texture: Texture,
    pub font_system: FontSystem,
    pub text_swash_cache: SwashCache,
}
