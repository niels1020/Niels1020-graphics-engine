use wgpu_game_engine::{
    logic::{Command, Commands, game_window::{GameInfo, InputHandler}},
    start_engine,
};
use winit::event::WindowEvent;

fn main() {
    start_engine(Input::new());
}

struct Input {}

impl InputHandler for Input {
    fn window_event(&mut self, game_info: &mut GameInfo, event: winit::event::WindowEvent) -> Commands {
        let mut commands = vec![];

        match event {
            WindowEvent::CloseRequested => {commands.push(Command::CloseWindow(game_info.window_id.unwrap()));}
            _ => {}
        }

        commands
    }
}

impl Input {
    pub fn new() -> Self {
        Self {}
    }
}
