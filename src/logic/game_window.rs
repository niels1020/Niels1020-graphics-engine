use std::time::{Duration, Instant};

use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use crate::{
    common::Commands,
    render::{render_objects::RenderObject, renderer::Renderer},
};

pub struct GameWindow {
    pub game_info: GameInfo,
    input_handler: Box<dyn InputHandler>,
    pub render_info: Option<Renderer>,
    timing: Timing,
}

pub struct SceneTree {
    pub root: Vec<Box<dyn RenderObject>>,
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
    pub fn new(input_handler: Box<dyn InputHandler>) -> Self {
        Self {
            input_handler,
            render_info: None,
            game_info: GameInfo::new(),
            timing: Timing::new(),
        }
    }

    pub fn start(&mut self, event_loop: &ActiveEventLoop) -> Commands{
        let info = pollster::block_on(Renderer::new(&event_loop));
        self.game_info.window_id = Some(info.window.id());
        self.render_info = Some(info);
        self.render_info.as_ref().unwrap().window.request_redraw();
        self.input_handler.start(&mut self.game_info)
    }

    pub fn window_event(&mut self, window_id: WindowId, event: WindowEvent) -> Commands {
        let mut commands = vec![];
        if let Some(render_info) = self.render_info.as_mut() {
            if render_info.window.id() == window_id {
                match event {
                    WindowEvent::Resized(size) => render_info.resize(size.width, size.height),
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();

                        //logic
                        let delta = now - self.timing.last_update;
                        self.timing.last_update = now;
                        commands.extend(
                            self.input_handler
                                .update(&mut self.game_info, delta.as_secs_f64()),
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
                    _ => {
                        commands.extend(self.input_handler.window_event(&mut self.game_info, event))
                    }
                }
            } else {
                commands.extend(self.input_handler.other_window_event(
                    &mut self.game_info,
                    window_id,
                    event,
                ));
            }
        }
        commands
    }
}

impl SceneTree {
    pub fn new() -> Self {
        Self { root: vec![] }
    }
}

pub trait InputHandler {
    //TODO: don't give raw windowevents to the user
    fn window_event(&mut self, game_info: &mut GameInfo, event: WindowEvent) -> Commands;
    fn other_window_event(
        &mut self,
        _game_info: &mut GameInfo,
        _window_id: WindowId,
        _event: WindowEvent,
    ) -> Commands {
        vec![]
    }
    fn update(&mut self, game_info: &mut GameInfo, delta: f64) -> Commands;

    fn start(&mut self, game_info: &mut GameInfo) -> Commands;
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
