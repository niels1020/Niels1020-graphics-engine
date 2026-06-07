use wgpu::{Device, Queue, SurfaceConfiguration};

pub struct Renderer {
}

impl Renderer {
    pub fn new(_device: &Device, _queue: &Queue, _config: &SurfaceConfiguration) -> Self {
        Self {  }
    }
}