use niels1020_graphics_engine::{
    common::{CAMERA_FAR_PLANE, CAMERA_FOV, CAMERA_NEAR_PLANE, DEFAULT_CAMERA_EYE, DEFAULT_CAMERA_TARGET}, include_wgsl, logic::{
        commands::Commands,
        game_window::{GameInfo, InputHandler},
    }, render::{
        render_2d::{camera::Camera2D, layer::RenderLayer2D, render_objects::text::Text},
        render_3d::{camera::Camera3D, layer::RenderLayer3D},
        render_layers::layer_as_type_mut,
    }, start_engine,
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
            } => {}
            WindowEvent::Resized(new_size) => {
                self.resolution = [new_size.width as f32, new_size.height as f32]
            }
            _ => {}
        }
    }

    fn update(&mut self, _commands: &mut Commands, _game_info: &mut GameInfo, _delta: f64) {}

    fn start(&mut self, _commands: &mut Commands, game_info: &mut GameInfo) {
        let layer1 = RenderLayer3D::new(
            include_wgsl!("../../assets/test/3d.wgsl"),
            "test 3D".to_string(),
            Camera3D {
                eye: DEFAULT_CAMERA_EYE.into(),
                // Point camera at the origin
                target: DEFAULT_CAMERA_TARGET.into(),
                // Define which direction is "up"
                up: cgmath::Vector3::unit_y(),
                aspect: 16.0 / 9.0,
                fovy: CAMERA_FOV,
                znear: CAMERA_NEAR_PLANE,
                zfar: CAMERA_FAR_PLANE,
            },
        );

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
