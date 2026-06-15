use wgpu_game_engine::{
    common::{Command, Commands}, logic::game_window::{GameInfo, InputHandler}, render::render_2d::camera::Camera2D, start_engine
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
                        text: _,
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

    fn update(&mut self, _game_info: &mut GameInfo, _delta: f64) -> Commands {
        vec![]
    }
    
    fn start(&mut self, game_info: &mut GameInfo) -> Commands {
        game_info.tree.root = vec![Camera2D::new("I DONT KNOW".to_string(), "Test 2D".to_string())];
        vec![]
    }
}

impl Input {
    pub fn new() -> Self {
        Self {}
    }
}
