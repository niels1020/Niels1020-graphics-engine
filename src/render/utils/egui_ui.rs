use std::sync::Arc;

use egui::{ClippedPrimitive, Context, FullOutput};
use egui_wgpu::{Renderer, RendererOptions};
use wgpu::{CommandEncoder, RenderPass};
use winit::window::Window;

use crate::render::utils::{global::RendererGlobal, texture::Texture};

pub struct UI {
    pub(crate) ctx: Context,
    pub(crate) output: Option<FullOutput>,
    renderer: Renderer,
    primitives: Vec<ClippedPrimitive>,
    pixels_per_point: f32,
}

impl UI {
    pub fn new(window: Arc<Window>, globals: &RendererGlobal, ctx: Context) -> Self {
        Self {
            renderer: Renderer::new(
                &globals.device,
                globals.config.format,
                RendererOptions {
                    msaa_samples: 0,
                    depth_stencil_format: Some(Texture::DEPTH_FORMAT),
                    dithering: true,
                    predictable_texture_filtering: false,
                },
            ),
            ctx,
            primitives: vec![],
            output: None,
            pixels_per_point: window.scale_factor() as f32,
        }
    }

    pub fn pre_render(&mut self, globals: &RendererGlobal, encoder: &mut CommandEncoder) {
        if self.output.is_none() {
            return;
        }
        let mut output = self.output.take().unwrap();
        self.primitives = self
            .ctx
            .tessellate(output.shapes, output.pixels_per_point);

        for (id, image_deltas) in &output.textures_delta.set {
            for image_delta in image_deltas {
                self.renderer
                    .update_texture(&globals.device, &globals.queue, *id, image_delta);
            }
        }

        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        output.textures_delta.clear();

        self.renderer.update_buffers(
            &globals.device,
            &globals.queue,
            encoder,
            &self.primitives,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [globals.config.width, globals.config.height],
                pixels_per_point: self.pixels_per_point,
            },
        );
    }

    pub fn render_pass(&mut self, globals: &RendererGlobal, render_pass: &mut RenderPass<'static>) {
        self.renderer.render(
            render_pass,
            &self.primitives,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [globals.config.width, globals.config.height],
                pixels_per_point: self.pixels_per_point,
            },
        );
    }
}
