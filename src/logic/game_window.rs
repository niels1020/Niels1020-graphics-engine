use std::{
    time::{Duration, Instant},
};

use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

use crate::{
    logic::commands::Commands,
    render::{render_objects::RenderLayer, renderer::Renderer},
};

//get removed after init
pub struct InitOnly {
    window_attributes: WindowAttributes,
}

pub struct GameWindow {
    pub game_info: GameInfo,
    input_handler: Box<dyn InputHandler>,
    pub renderer: Option<Renderer>,
    timing: Timing,
    init_only: Option<InitOnly>,
}

pub struct SceneTree {
    pub root: Vec<Box<dyn RenderLayer>>,
}

struct Timing {
    last_update: Instant,
    render_time: Duration,
    last_render: Instant,
}

impl Timing {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            render_time: Duration::from_secs(0),
            last_render: Instant::now(),
        }
    }
}

impl GameWindow {
    pub fn new(input_handler: Box<dyn InputHandler>, window_attributes: WindowAttributes) -> Self {
        Self {
            input_handler,
            renderer: None,
            game_info: GameInfo::new(),
            timing: Timing::new(),
            init_only: Some(InitOnly { window_attributes }),
        }
    }

    pub fn start(&mut self, commands: &mut Commands, event_loop: &ActiveEventLoop) {
        let info = pollster::block_on(Renderer::new(
            &event_loop,
            &self.init_only.as_ref().unwrap().window_attributes,
        ));
        self.game_info.window_id = Some(info.window.id());
        self.renderer = Some(info);
        self.renderer.as_ref().unwrap().window.request_redraw();
        self.input_handler.start(commands, &mut self.game_info);


        self.init_only = None;
    }

    pub fn window_event(
        &mut self,
        commands: &mut Commands,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(render_info) = self.renderer.as_mut() {
            if render_info.window.id() == window_id {
                match event {
                    WindowEvent::Resized(size) => render_info.resize(size.width, size.height),
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();

                        //logic
                        let delta = now - self.timing.last_update;
                        self.timing.last_update = now;
                        self.input_handler.update(
                            commands,
                            &mut self.game_info,
                            delta.as_secs_f64(),
                        );
                        //rendering at selected frame rate
                        if now - self.timing.last_render
                            >= Duration::from_millis(1000 / self.game_info.refresh_rate)
                        {
                            let befor_render = now;
                            self.timing.last_render = now;
                            render_info.render(&mut self.game_info);
                            self.timing.render_time = now - befor_render;
                        }
                        render_info.window.request_redraw();
                    }
                    _ => self
                        .input_handler
                        .window_event(commands, &mut self.game_info, event),
                }
            } else {
                self.input_handler.other_window_event(
                    commands,
                    &mut self.game_info,
                    window_id,
                    event,
                );
            }
        }
    }
}

impl SceneTree {
    pub fn new() -> Self {
        Self { root: vec![] }
    }
}

pub trait InputHandler {
    //TODO: don't give raw windowevents to the user
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
}

pub struct GameInfo {
    pub tree: SceneTree,
    pub window_id: Option<WindowId>,
    pub refresh_rate: u64,
}

impl GameInfo {
    fn new() -> Self {
        Self {
            tree: SceneTree::new(),
            window_id: None,
            refresh_rate: 144,
        }
    }
}
