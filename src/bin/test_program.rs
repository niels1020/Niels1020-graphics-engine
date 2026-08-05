use wgpu_game_engine::{
    include_wgsl,
    logic::{
        commands::Commands,
        game_window::{GameInfo, InputHandler},
    },
    render::{
        render_2d::{self, camera::Camera2D, layer::RenderLayer2D},
        render_layers::layer_as_type_mut,
    },
    start_engine,
};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};

fn main() {
    start_engine(Box::new(Input::new()), WindowAttributes::default());
}

struct Input {
    resolution: [f32; 2],
}

impl InputHandler for Input {
    fn window_event(
        &mut self,
        commands: &mut Commands,
        game_info: &mut GameInfo,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                commands.close_window(game_info.window_id);
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
                        KeyCode::Enter => {
                            commands.new_window(Box::new(Input::new()), WindowAttributes::default())
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                if let Some(layer) = game_info.tree.root.get_mut(0) {
                    match layer_as_type_mut::<RenderLayer2D>(layer) {
                        Some(layer2d) => {
                            layer2d.camera.data.position = [
                                ((position.x as f32) - (self.resolution[0] / 2.0)) * -1.0,
                                (position.y as f32) - (self.resolution[1] / 2.0),
                            ]
                        }
                        None => {}
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                self.resolution = [new_size.width as f32, new_size.height as f32]
            }
            _ => {}
        }
    }

    fn update(&mut self, _commands: &mut Commands, _game_info: &mut GameInfo, _delta: f64) {}

    fn start(&mut self, _commands: &mut Commands, game_info: &mut GameInfo) {
        let mut layer1 = RenderLayer2D::new(
            include_wgsl!("../../assets/2d.wgsl"),
            "Test 2D".to_string(),
            Camera2D::new([800.0, 600.0]),
        );
        layer1.add_child(render_2d::render_objects::test::VerticesTest::new());
        layer1.add_child(render_2d::render_objects::test::TextureTest::new());
        game_info.tree.root = vec![layer1];
    }

    fn exit(&mut self, _game_info: &mut GameInfo) {}
}

impl Input {
    pub fn new() -> Self {
        Self {
            resolution: [1.0; 2],
        }
    }
}
