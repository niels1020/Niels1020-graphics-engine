use wgpu_game_engine::{
    include_wgsl,
    logic::{
        commands::Commands,
        game_window::{GameInfo, InputHandler},
    },
    render::render_2d::{self, camera::Camera2D},
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
        commands: &mut Commands,
        game_info: &mut GameInfo,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                commands.close_window(game_info.window_id.unwrap());
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
                        KeyCode::Enter => commands.new_window(Box::new(Input::new())),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, _commands: &mut Commands, _game_info: &mut GameInfo, _delta: f64) {}

    fn start(&mut self, _commands: &mut Commands, game_info: &mut GameInfo) {
        let mut layer1 =
            Camera2D::new(include_wgsl!("../../assets/2d.wgsl"), "Test 2D".to_string());
        layer1.add_child(render_2d::render_objects::test::VerticesTest::new());
        game_info.tree.root = vec![layer1];
    }
}

impl Input {
    pub fn new() -> Self {
        Self {}
    }
}
