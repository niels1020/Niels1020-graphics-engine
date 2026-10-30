use std::sync::Arc;

use cosmic_text::{FontSystem, SwashCache};
use egui::Context;
use log::{info, warn};
use wgpu::{
    CurrentSurfaceTexture, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits,
    RenderPass, Surface, TextureUsages, TextureViewDescriptor,
    wgt::{CommandEncoderDescriptor, DeviceDescriptor, SurfaceConfiguration},
};
use winit::window::{Window, WindowId};

use crate::{
    common::{CLEAR_COLOR, DEPTH_CLEAR_VALUE, MAX_FRAME_LATENCY},
    logic::game_window::SceneTree,
    render::utils::{egui_ui::UI, global::RendererGlobal, texture::Texture},
};

pub struct Renderer {
    pub(crate) global: RendererGlobal,
    is_surface_configured: bool,
    pub(crate) window_res: Option<[u32; 2]>,
    surface: Option<Surface<'static>>,
    pub(crate) window_id: Option<WindowId>,
    pub ui: UI,
    pub is_headless: bool,
}

impl Renderer {
    pub async fn new_headless(egui_ctx: Context) -> Self {
        let globals = RendererGlobal {
            font_system: FontSystem::new(),
            text_swash_cache: SwashCache::new(),
            device: None,
            config: None,
            queue: None,
            depth_texture: None,
            is_headless: true,
        };
        Self {
            ui: UI::new(None, &globals, egui_ctx),
            global: globals,
            is_surface_configured: false,
            is_headless: true,
            window_res: None,
            surface: None,
            window_id: None,
        }
    }

    pub async fn new(window: Arc<Window>, egui_ctx: Context) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("niels1020_graphics_engine"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // Configure the surface with appropriate settings for the display
        let config: wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>> =
            SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                // Use Fifo for vsync (synchronized with display refresh rate)
                present_mode: surface_caps
                    .present_modes
                    .iter()
                    .find(|f| **f == wgpu::PresentMode::Fifo)
                    .copied()
                    .unwrap_or(surface_caps.present_modes[0]),
                desired_maximum_frame_latency: MAX_FRAME_LATENCY,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                color_space: wgpu::SurfaceColorSpace::Auto,
            };

        surface.configure(&device, &config);

        info!("done renderer init");

        let global = RendererGlobal {
            depth_texture: Some(Texture::create_depth_texture(
                &device,
                &config,
                "depth texture",
            )),
            device: Some(device),
            config: Some(config),
            queue: Some(queue),
            font_system: FontSystem::new(),
            text_swash_cache: SwashCache::new(),
            is_headless: false,
        };

        Self {
            window_id: Some(window.id()),
            ui: UI::new(Some(window), &global, egui_ctx),
            global,
            is_surface_configured: true,
            window_res: Some([size.width, size.height]),
            surface: Some(surface),
            is_headless: false,
        }
    }

    pub fn render(&mut self, scene_tree: &mut SceneTree) {
        if !self.is_surface_configured {
            info!("surface not configured");
            return;
        }
        if self.is_headless {
            warn!("running in headless mode: no graphics");
            return;
        }
        if let CurrentSurfaceTexture::Success(output) =
            self.surface.as_ref().unwrap().get_current_texture()
        {
            let view = output
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut encoder = self.global.device.as_ref().unwrap().create_command_encoder(
                &CommandEncoderDescriptor {
                    label: Some("wgpu_game_engine render encoder"),
                },
            );

            self.ui.pre_render(&self.global, &mut encoder);
            //render pass
            {
                let render_pass: wgpu::RenderPass<'_> =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.global.depth_texture.as_ref().unwrap().view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(DEPTH_CLEAR_VALUE),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        occlusion_query_set: None,
                        timestamp_writes: None,
                        multiview_mask: None,
                    });

                let mut render_pass = render_pass.forget_lifetime();
                self.render_tree(scene_tree, &mut render_pass);
                self.ui.render_pass(&self.global, &mut render_pass);
            }

            // submit will accept anything that implements IntoIter
            self.global
                .queue
                .as_ref()
                .unwrap()
                .submit(std::iter::once(encoder.finish()));
            self.global.queue.as_ref().unwrap().present(output);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.is_headless {
            warn!("running in headless mode: no graphics");
            return;
        }
        if width > 0 && height > 0 {
            self.global.config.as_mut().unwrap().width = width;
            self.global.config.as_mut().unwrap().height = height;
            self.surface.as_ref().unwrap().configure(
                &self.global.device.as_ref().unwrap(),
                &self.global.config.as_ref().unwrap(),
            );
            self.is_surface_configured = true;
            self.global.depth_texture = Some(Texture::create_depth_texture(
                &self.global.device.as_ref().unwrap(),
                &self.global.config.as_ref().unwrap(),
                "depth_texture",
            ));
            self.window_res = Some([width, height]);
        }
    }

    pub fn render_tree(&mut self, tree: &mut SceneTree, render_pass: &mut RenderPass) {
        for object in tree.root.iter_mut() {
            object.render(&mut self.global, render_pass);
        }
    }
}
