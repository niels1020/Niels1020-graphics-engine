use std::{
    sync::Arc,
};

use egui::Context;
use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    logic::{
        commands::{GlobalComands, Request},
        threaded::{SharedLogicInfo, start_logic_thread},
    },
    render::{render_layers::RenderLayer, renderer::Renderer, threaded::start_render_thread},
};

//get removed after init
pub struct InitOnly {
    window_attributes: WindowAttributes,
}

pub struct GameWindow {
    //gets removed after init
    input_handler: Option<Box<dyn InputHandler + Send>>,
    init_only: Option<InitOnly>,
    pub(crate) shared_logic_info: Option<SharedLogicInfo>,
    pub(crate) window_id: WindowId,
}

#[derive(Clone)]
pub struct SceneTree {
    pub root: Vec<Box<dyn RenderLayer>>,
}

impl GameWindow {
    pub fn new(
        input_handler: Box<dyn InputHandler + Send>,
        window_attributes: WindowAttributes,
    ) -> Self {
        Self {
            input_handler: Some(input_handler),
            init_only: Some(InitOnly { window_attributes }),
            shared_logic_info: None,
            window_id: WindowId::dummy(),
        }
    }

    pub fn start(&mut self, commands: GlobalComands, event_loop: &ActiveEventLoop) {
        let init_only = self.init_only.take().unwrap();

        let window = Arc::new(
            event_loop
                .create_window(init_only.window_attributes.clone())
                .unwrap(),
        );

        let egui_ctx = Context::default();

        let (shared_logic_info, shared_render_info) = start_logic_thread(
            commands.clone(),
            window.clone(),
            self.input_handler.take().unwrap(),
            egui_ctx.clone(),
        );

        self.window_id = window.id();

        start_render_thread(
            commands,
            pollster::block_on(Renderer::new(window, egui_ctx)),
            shared_render_info,
        );

        self.shared_logic_info = Some(shared_logic_info);
    }

    pub fn window_event(&mut self, window_id: WindowId, event: WindowEvent) {
        if let Ok(mut shared) = self.shared_logic_info.as_ref().unwrap().lock() {
            shared.window_events.push_back((event, window_id));
        } else {
            panic!("could not lock window event queue")
        }
    }

    pub fn device_event(&mut self, event: DeviceEvent, device_id: DeviceId) {
        if let Ok(mut shared) = self.shared_logic_info.as_ref().unwrap().lock() {
            shared.device_events.push_back((event, device_id));
        } else {
            panic!("could not lock device event queue")
        }
    }
}

impl SceneTree {
    pub fn new() -> Self {
        Self { root: vec![] }
    }
}

pub trait InputHandler: Send {
    fn window_event(
        &mut self,
        commands: GlobalComands,
        game_info: &mut GameInfo,
        event: WindowEvent,
        consumed: bool,
    );
    fn other_window_event(
        &mut self,
        _commands: GlobalComands,
        _game_info: &mut GameInfo,
        _window_id: WindowId,
        _event: WindowEvent,
        _consumed: bool,
    ) {
    }
    fn update(&mut self, commands: GlobalComands, game_info: &mut GameInfo, delta: f64);

    fn start(&mut self, commands: GlobalComands, game_info: &mut GameInfo);

    fn exit(&mut self, commands: GlobalComands, game_info: &mut GameInfo);

    fn receive_request(
        &mut self,
        commands: GlobalComands,
        game_info: &mut GameInfo,
        request: Request,
    );

    fn device_event(
        &mut self,
        _commands: GlobalComands,
        _game_info: &mut GameInfo,
        _event: DeviceEvent,
        _device_id: DeviceId,
    ) {
    }

    fn gui(&mut self, commands: GlobalComands, game_info: &mut GameInfo, ctx: egui::Context);
}

pub struct GameInfo {
    pub window: Arc<Window>,
    pub window_id: WindowId,
    pub max_queue_size: usize,
}

impl GameInfo {
    pub(crate) fn new(window: Arc<Window>) -> Self {
        Self {
            window_id: window.id(),
            window,
            max_queue_size: 50,
        }
    }
}
