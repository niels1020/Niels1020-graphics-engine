pub const MAX_FRAME_LATENCY: u32 = 2;

use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use crate::{
    logic::Commands,
    render::{render_info::RenderInfo, render_objects::RenderObject},
};

pub struct GameWindow<T: InputHandler> {
    pub game_info: GameInfo,
    input_handler: T,
    render_info: Option<RenderInfo>,
}

pub struct SceneTree {
    root: Option<RenderObject>,
}

impl<T: InputHandler> GameWindow<T> {
    pub fn new(input_handler: T) -> Self {
        Self {
            input_handler,
            render_info: None,
            game_info: GameInfo::new(),
        }
    }

    pub fn render_info_init(&mut self, event_loop: &ActiveEventLoop) {
        let info = pollster::block_on(RenderInfo::new(&event_loop));
        self.game_info.window_id = Some(info.id.clone());
        self.render_info = Some(info);
    }

    pub fn window_event(&mut self, window_id: WindowId, event: WindowEvent) -> Commands {
        let mut commands = vec![];
        if let Some(render_info) = self.render_info.as_mut() {
            if render_info.id == window_id {
                commands.extend(self.input_handler.window_event(&mut self.game_info, event));
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
        Self { root: None }
    }
}

pub trait InputHandler {
    fn window_event(&mut self, game_info: &mut GameInfo, event: WindowEvent) -> Commands;
    fn other_window_event(
        &mut self,
        game_info: &mut GameInfo,
        _window_id: WindowId,
        _event: WindowEvent,
    ) -> Commands {
        vec![]
    }
}

pub struct GameInfo {
    pub tree: SceneTree,
    pub window_id: Option<WindowId>,
}

impl GameInfo {
    fn new() -> Self {
        Self {
            tree: SceneTree::new(),
            window_id: None,
        }
    }
}
