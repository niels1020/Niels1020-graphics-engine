use cosmic_text::{FontSystem, SwashCache};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::render::utils::texture::Texture;

pub struct RendererGlobal {
    pub device: Option<Device>,
    pub config: Option<SurfaceConfiguration>,
    pub queue: Option<Queue>,
    pub depth_texture: Option<Texture>,
    pub font_system: FontSystem,
    pub text_swash_cache: SwashCache,
    pub is_headless: bool,
}
