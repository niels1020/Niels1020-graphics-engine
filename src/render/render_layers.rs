use std::any::Any;

use wgpu::RenderPass;

use crate::render::utils::global::RendererGlobal;

pub trait RenderLayer {
    fn render(
        &mut self,
        global: &mut RendererGlobal,
        render_pass: &mut RenderPass,
    );

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub fn layer_as_type_mut<T: RenderLayer + 'static>(layer: &mut Box<dyn RenderLayer + Send>) -> Option<&mut T> {
    layer.as_any_mut().downcast_mut::<T>()
}
