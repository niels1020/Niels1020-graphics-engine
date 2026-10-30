use std::sync::Arc;

use egui::{Context, FontData, FontFamily, FontInsert, FontPriority, InsertFontFamily};
use log::{info, warn};
use winit::{
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    logic::{
        commands::{GlobalCommands, Request},
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
    pub(crate) window_id: Option<WindowId>,
    is_headless: bool,
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
            window_id: None,
            is_headless: false,
        }
    }

    fn init(&mut self, commands: GlobalCommands, event_loop: &ActiveEventLoop) {
        let init_only = self.init_only.take().unwrap();

        let tmp = event_loop.create_window(init_only.window_attributes.clone());
        if tmp.is_err() {
            self.init_headless(commands);
            warn!("failed to open window interpreting as doctest and running headless");
            warn!("actuall error: {}", tmp.err().unwrap());
            return;
        }

        let window = Arc::new(tmp.unwrap());

        let egui_ctx = Context::default();

        egui_ctx.add_font(FontInsert::new(
            "OxProta",
            FontData::from_static(include_bytes!("../../assets/0xProtoNerdFont-Regular.ttf")),
            vec![InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Highest,
            }],
        ));

        let (shared_logic_info, shared_render_info) = start_logic_thread(
            commands.clone(),
            window.clone(),
            self.input_handler.take().unwrap(),
            egui_ctx.clone(),
        );

        self.window_id = Some(window.id());

        start_render_thread(
            commands,
            pollster::block_on(Renderer::new(window, egui_ctx)),
            shared_render_info,
        );

        self.shared_logic_info = Some(shared_logic_info);

        info!("window started")
    }

    fn init_headless(&mut self, commands: GlobalCommands) {
        self.is_headless = true;
        //exit immeaditly as there is no logic after this
        commands.exit();
    }

    pub fn start(&mut self, commands: GlobalCommands, event_loop: &ActiveEventLoop) {
        if cfg!(test) {
            self.init_headless(commands);
        } else {
            self.init(commands, event_loop);
        }
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
        commands: GlobalCommands,
        game_info: &mut GameInfo,
        event: WindowEvent,
        consumed: bool,
    );
    fn other_window_event(
        &mut self,
        _commands: GlobalCommands,
        _game_info: &mut GameInfo,
        _window_id: WindowId,
        _event: WindowEvent,
        _consumed: bool,
    ) {
    }
    fn update(&mut self, commands: GlobalCommands, game_info: &mut GameInfo, delta: f64);

    fn start(&mut self, commands: GlobalCommands, game_info: &mut GameInfo);

    fn exit(&mut self, commands: GlobalCommands, game_info: &mut GameInfo);

    fn receive_request(
        &mut self,
        commands: GlobalCommands,
        game_info: &mut GameInfo,
        request: Request,
    );

    fn device_event(
        &mut self,
        _commands: GlobalCommands,
        _game_info: &mut GameInfo,
        _event: DeviceEvent,
        _device_id: DeviceId,
    ) {
    }

    fn gui(&mut self, commands: GlobalCommands, game_info: &mut GameInfo, ctx: egui::Context);
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
