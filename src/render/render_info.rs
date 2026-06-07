use std::sync::Arc;

use wgpu::{ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits, Queue, TextureUsages, wgt::{DeviceDescriptor, SurfaceConfiguration}};
use winit::{event_loop::ActiveEventLoop, window::{Window, WindowId}};

use crate::{logic::game_window::MAX_FRAME_LATENCY, render::renderer::Renderer};

pub struct RenderInfo {
    pub renderer: Renderer,
    pub id: WindowId,
    pub queue: Queue,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub window: Arc<Window>,
}

impl RenderInfo {
    pub async fn new(event_loop: &ActiveEventLoop) -> Self {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("wgpu_game"),
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
            };

        let renderer = Renderer::new(&device, &queue, &config);

        Self {
            id: window.id(),
            window: window,
            renderer,
            queue,
            surface,
            device,
            config,
            is_surface_configured: false,
        }
    }
}