use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    logic::{
        commands::Commands,
        threaded::{SharedInfo, start_logic_thread},
    },
    render::{render_layers::RenderLayer, renderer::Renderer},
};

//get removed after init
pub struct InitOnly {
    window_attributes: WindowAttributes,
}

pub struct GameWindow {
    //gets removed after init
    input_handler: Option<Box<dyn InputHandler + Send>>,
    pub(crate) renderer: Option<Renderer>,
    init_only: Option<InitOnly>,
    pub(crate) shared_info: Option<Arc<Mutex<SharedInfo>>>,
    last_render: Instant,
}

pub struct SceneTree {
    pub root: Vec<Box<dyn RenderLayer + Send>>,
}

impl GameWindow {
    pub fn new(
        input_handler: Box<dyn InputHandler + Send>,
        window_attributes: WindowAttributes,
    ) -> Self {
        Self {
            input_handler: Some(input_handler),
            renderer: None,
            init_only: Some(InitOnly { window_attributes }),
            shared_info: None,
            last_render: Instant::now(),
        }
    }

    pub fn start(&mut self, _commands: &mut Commands, event_loop: &ActiveEventLoop) {
        let init_only = self.init_only.take().unwrap();

        let window = Arc::new(
            event_loop
                .create_window(init_only.window_attributes.clone())
                .unwrap(),
        );

        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.renderer = Some(renderer);

        let shared_info = start_logic_thread(window, self.input_handler.take().unwrap());

        self.shared_info = Some(shared_info);
    }

    pub fn window_event(
        &mut self,
        commands: &mut Commands,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                commands.close_window(window_id);
                return;
            }
            WindowEvent::Destroyed => {
                commands.close_window(window_id);
                return;
            }
            _ => {}
        }

        if let Some(renderer) = self.renderer.as_mut() {
            if renderer.window_id == window_id {
                match event {
                    WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
                    WindowEvent::RedrawRequested => {
                        let mut shared = self.shared_info.as_ref().unwrap().lock().unwrap();
                        let now = Instant::now();
                        if (now - self.last_render)
                            >= Duration::from_secs_f64(1.0 / shared.refresh_rate as f64)
                        {
                            self.last_render = now;
                            renderer.render(&mut shared.game_info);

                            //append main command buffer very frame
                            commands.append(&mut shared.commands);
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Ok(mut shared) = self.shared_info.as_ref().unwrap().try_lock() {
            shared.window_events.push((event, window_id));
        }
    }

    pub fn device_event(&mut self, event: DeviceEvent, device_id: DeviceId) {
        if let Ok(mut shared) = self.shared_info.as_ref().unwrap().try_lock() {
            shared.device_events.push((event, device_id));
        }
    }
}

impl SceneTree {
    pub fn new() -> Self {
        Self { root: vec![] }
    }
}

pub trait InputHandler {
    fn window_event(
        &mut self,
        commands: &mut Commands,
        game_info: &mut GameInfo,
        event: WindowEvent,
    );
    fn other_window_event(
        &mut self,
        _commands: &mut Commands,
        _game_info: &mut GameInfo,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
    fn update(&mut self, commands: &mut Commands, game_info: &mut GameInfo, delta: f64);

    fn start(&mut self, commands: &mut Commands, game_info: &mut GameInfo);

    fn exit(&mut self, game_info: &mut GameInfo);

    fn device_event(
        &mut self,
        _commands: &mut Commands,
        _game_info: &mut GameInfo,
        _event: DeviceEvent,
        _device_id: DeviceId,
    ) {
    }
}

pub struct GameInfo {
    pub tree: SceneTree,
    pub refresh_rate: u64,
    pub window: Arc<Window>,
}

impl GameInfo {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        Self {
            tree: SceneTree::new(),
            refresh_rate: 144,
            window,
        }
    }
}
