use std::{any::Any, fmt::Debug};

use wgpu::RenderPass;

use crate::render::utils::global::RendererGlobal;

pub trait RenderLayer: Send + Debug {
    fn render(&mut self, global: &mut RendererGlobal, render_pass: &mut RenderPass);

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn RenderLayer>;
}

impl Clone for Box<dyn RenderLayer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
pub fn layer_as_type_mut<T: RenderLayer + 'static>(
    layer: &mut Box<dyn RenderLayer>,
) -> Option<&mut T> {
    layer.as_any_mut().downcast_mut::<T>()
}
