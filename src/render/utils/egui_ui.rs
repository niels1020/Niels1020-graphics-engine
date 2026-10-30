use std::sync::Arc;

use egui::{ClippedPrimitive, Context, FullOutput};
use egui_wgpu::{Renderer, RendererOptions};
use log::info;
use wgpu::{CommandEncoder, RenderPass};
use winit::window::Window;

use crate::render::utils::{global::RendererGlobal, texture::Texture};

pub struct UI {
    pub(crate) ctx: Context,
    pub(crate) output: Option<FullOutput>,
    renderer: Option<Renderer>,
    primitives: Vec<ClippedPrimitive>,
    pixels_per_point: f32,
    is_headless: bool,
}

impl UI {
    pub fn new(window: Option<Arc<Window>>, globals: &RendererGlobal, ctx: Context) -> Self {
        if window.is_none() {
            info!("GUI detected headles");
            return Self {
                ctx,
                output: None,
                renderer: None,
                primitives: vec![],
                pixels_per_point: 0.0,
                is_headless: true,
            };
        }
        Self {
            renderer: Some(Renderer::new(
                globals.device.as_ref().unwrap(),
                globals.config.as_ref().unwrap().format,
                RendererOptions {
                    msaa_samples: 0,
                    depth_stencil_format: Some(Texture::DEPTH_FORMAT),
                    dithering: true,
                    predictable_texture_filtering: false,
                },
            )),
            ctx,
            primitives: vec![],
            output: None,
            pixels_per_point: window.unwrap().scale_factor() as f32,
            is_headless: false,
        }
    }

    pub fn pre_render(&mut self, globals: &RendererGlobal, encoder: &mut CommandEncoder) {
        if self.output.is_none() {
            return;
        }

        let mut output = self.output.take().unwrap();

        if self.is_headless {
            output.drop_without_applying_deltas();
            return;
        }
        self.primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);

        for (id, image_deltas) in &output.textures_delta.set {
            for image_delta in image_deltas {
                self.renderer.as_mut().unwrap().update_texture(
                    globals.device.as_ref().unwrap(),
                    globals.queue.as_ref().unwrap(),
                    *id,
                    image_delta,
                );
            }
        }

        for id in &output.textures_delta.free {
            self.renderer.as_mut().unwrap().free_texture(id);
        }

        output.textures_delta.clear();

        self.renderer.as_mut().unwrap().update_buffers(
            globals.device.as_ref().unwrap(),
            globals.queue.as_ref().unwrap(),
            encoder,
            &self.primitives,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [
                    globals.config.as_ref().unwrap().width,
                    globals.config.as_ref().unwrap().height,
                ],
                pixels_per_point: self.pixels_per_point,
            },
        );
    }

    pub fn render_pass(&mut self, globals: &RendererGlobal, render_pass: &mut RenderPass<'static>) {
        if self.is_headless {
            return;
        }
        self.renderer.as_mut().unwrap().render(
            render_pass,
            &self.primitives,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [
                    globals.config.as_ref().unwrap().width,
                    globals.config.as_ref().unwrap().height,
                ],
                pixels_per_point: self.pixels_per_point,
            },
        );
    }
}
