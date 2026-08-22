use niels1020_graphics_engine::{
    common::{
        CAMERA_FAR_PLANE, CAMERA_FOV, CAMERA_NEAR_PLANE, CUBE_VERTICES, DEFAULT_CAMERA_EYE,
        DEFAULT_CAMERA_TARGET,
    },
    logic::{
        commands::Commands,
        game_window::{GameInfo, InputHandler},
    },
    render::render_3d::{
        camera::Camera3D,
        layer::{RenderLayer3D, RenderObject3D, Transform},
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
                commands.close_window(game_info.window.id());
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
                position: _,
            } => {}
            WindowEvent::Resized(new_size) => {
                self.resolution = [new_size.width as f32, new_size.height as f32]
            }
            _ => {}
        }
    }

    fn update(&mut self, _commands: &mut Commands, _game_info: &mut GameInfo, _delta: f64) {}

    fn start(&mut self, _commands: &mut Commands, game_info: &mut GameInfo) {
        let mut layer1 = RenderLayer3D::new(
            None,
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

        layer1.add_child(Box::new(CubeTest {
            have_vertices_changed: true,
            has_transform_changed: true,
            transform: Transform::new([2.0, 0.0, 0.0], [30.0; 3]),
        }));

        game_info.tree.root = vec![layer1];

        game_info.window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
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

pub struct CubeTest {
    have_vertices_changed: bool,
    transform: Transform,
    has_transform_changed: bool,
}

impl RenderObject3D for CubeTest {
    fn have_vertices_changed(&self) -> bool {
        self.have_vertices_changed
    }

    fn get_vertices(
        &mut self,
        global: &mut niels1020_graphics_engine::render::render_3d::layer::RenderLayer3DGlobal,
    ) -> Vec<niels1020_graphics_engine::common::Vertex> {
        self.have_vertices_changed = true;
        if !global.atlas_texture.has_image("profiel".to_string()) {
            global.atlas_texture.add_image(
                image::load_from_memory(include_bytes!("../../assets/test/profiel.png")).unwrap(),
                "profiel".to_string(),
            );
        }
        CUBE_VERTICES.to_vec()
    }

    fn get_name(&self) -> String {
        "cube test".to_string()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_transform(&self) -> niels1020_graphics_engine::render::render_3d::layer::Transform {
        self.transform
    }

    fn has_transform_changed(&self) -> bool {
        self.has_transform_changed
    }
}
