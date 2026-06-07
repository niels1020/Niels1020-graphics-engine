use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration};

use crate::{
    logic::game_window::{GameInfo, SceneTree},
    render::{render_info::RenderInfo, texture::Texture},
};

pub struct Renderer {
    pub depth_texture: Texture,
    pub window_res: [u32;2]
}

impl Renderer {
    pub fn new(device: &Device, _queue: &Queue, config: &SurfaceConfiguration) -> Self {
        Self {
            depth_texture: Texture::create_depth_texture(device, config, "depth_texture"),
            window_res: [0,0],
        }
    }

    pub fn render(&mut self, game_info: &mut GameInfo, render_pass: &mut RenderPass) {

    }
}
