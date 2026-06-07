use wgpu_game_engine::{
    common::{Command, Commands},
    logic::game_window::{GameInfo, InputHandler},
    start_engine,
};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

fn main() {
    start_engine(Box::new(Input::new()));
}

struct Input {}

impl InputHandler for Input {
    fn window_event(
        &mut self,
        game_info: &mut GameInfo,
        event: winit::event::WindowEvent,
    ) -> Commands {
        let mut commands = vec![];

        match event {
            WindowEvent::CloseRequested => {
                commands.push(Command::CloseWindow(game_info.window_id.unwrap()));
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        text,
                        ..
                    },
                ..
            } => {
                if key_state == ElementState::Released {
                    match code {
                        KeyCode::Enter => commands.push(Command::NewWindow(Box::new(Input::new()))),
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        commands
    }

    fn update(&mut self, delta: f64) -> Commands {
        vec![]
    }
}

impl Input {
    pub fn new() -> Self {
        Self {}
    }
}
