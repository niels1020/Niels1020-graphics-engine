use std::sync::Arc;

use cosmic_text::{FontSystem, SwashCache};
use wgpu::{
    CurrentSurfaceTexture, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits,
    Queue, RenderPass, Surface, TextureUsages, TextureViewDescriptor,
    wgt::{CommandEncoderDescriptor, DeviceDescriptor, SurfaceConfiguration},
};
use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    common::{CLEAR_COLOR, DEPTH_CLEAR_VALUE, MAX_FRAME_LATENCY},
    logic::game_window::{GameInfo, SceneTree},
    render::utils::{global::RendererGlobal, texture::Texture},
};

pub struct Renderer {
    global: RendererGlobal,
    is_surface_configured: bool,
    pub window: Arc<Window>,
    window_res: [u32; 2],
    surface: Surface<'static>,
}

impl Renderer {
    pub async fn new(event_loop: &ActiveEventLoop, window_attributes: &WindowAttributes) -> Self {
        let window = Arc::new(event_loop.create_window(window_attributes.clone()).unwrap());

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
                label: Some("wgpu_game_engine"),
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

        Self {
            window: window,
            global: RendererGlobal {
                depth_texture: Texture::create_depth_texture(&device, &config, "depth texture"),
                device,
                config,
                queue,
                font_system: FontSystem::new(),
                text_swash_cache: SwashCache::new(),
            },
            is_surface_configured: true,
            window_res: [size.width, size.height],
            surface,
        }
    }

    pub fn render(&mut self, game_info: &mut GameInfo) {
        if !self.is_surface_configured {
            println!("surface not configured");
            return;
        }
        if let CurrentSurfaceTexture::Success(output) = self.surface.get_current_texture() {
            let view = output
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut encoder =
                self.global
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("wgpu_game_engine render encoder"),
                    });

            //render pass
            {
                let mut render_pass: wgpu::RenderPass<'_> =
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
                            view: &self.global.depth_texture.view,
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

                self.render_tree(&mut game_info.tree, &mut render_pass);
            }

            // submit will accept anything that implements IntoIter
            self.global.queue.submit(std::iter::once(encoder.finish()));
            self.global.queue.present(output);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.global.config.width = width;
            self.global.config.height = height;
            self.surface
                .configure(&self.global.device, &self.global.config);
            self.is_surface_configured = true;
            self.global.depth_texture = Texture::create_depth_texture(
                &self.global.device,
                &self.global.config,
                "depth_texture",
            );
            self.window_res = [width, height];
        }
    }

    pub fn render_tree(&mut self, tree: &mut SceneTree, render_pass: &mut RenderPass) {
        for object in tree.root.iter_mut() {
            object.render(&mut self.global, render_pass);
        }
    }
}
