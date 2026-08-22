use niels1020_graphics_engine::{
    logic::{
        commands::Commands,
        game_window::{GameInfo, InputHandler},
    },
    render::{
        render_2d::{camera::Camera2D, layer::RenderLayer2D, render_objects::text::Text},
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
            None,
            "Test 2D".to_string(),
            Camera2D::new([800.0, 600.0]),
        );
        layer1.add_child(VerticesTest::new());
        layer1.add_child(TextureTest::new());
        layer1.add_child(Text::new(
            "0xProto Nerd Font".to_string(),
            "Hello, world! ".to_string(),
            20.0,
            (255, 255, 255, 255),
            (0.0, 250.0, 0.5),
        ));
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

use niels1020_graphics_engine::{
    common::Vertex,
    render::render_2d::layer::{RenderLayer2DGlobal, RenderObject2D},
};

pub struct VerticesTest {
    has_updated: bool,
}

impl VerticesTest {
    pub fn new() -> Box<Self> {
        Box::new(Self { has_updated: false })
    }
}

impl RenderObject2D for VerticesTest {
    fn have_vertices_changed(&mut self) -> bool {
        !self.has_updated
    }

    fn get_vertices(&mut self, _: &mut RenderLayer2DGlobal) -> Vec<Vertex> {
        self.has_updated = true;
        vec![
            Vertex::new(400.0, 0.0, 0.01, 0.0, 0.0, 0),
            Vertex::new(400.0, 300.0, 0.01, 0.0, 0.0, 0),
            Vertex::new(0.0, 300.0, 0.01, 0.0, 0.0, 0),
            Vertex::new(-400.0, 0.0, 0.01, 0.0, 0.0, 0),
            Vertex::new(-400.0, -300.0, 0.01, 0.0, 0.0, 0),
            Vertex::new(0.0, -300.0, 0.01, 0.0, 0.0, 0),
        ]
    }

    fn get_name(&self) -> String {
        "VERTICES TEST".to_string()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct TextureTest {
    has_updated: bool,
    image_added: bool,
}

impl TextureTest {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            has_updated: false,
            image_added: false,
        })
    }
}

impl RenderObject2D for TextureTest {
    fn have_vertices_changed(&mut self) -> bool {
        !self.has_updated
    }

    fn get_vertices(&mut self, global: &mut RenderLayer2DGlobal) -> Vec<Vertex> {
        if !self.image_added {
            self.image_added = true;
            global.atlas_texture.add_image(
                image::load_from_memory(include_bytes!("../../assets/test/profiel.png")).unwrap(),
                "profiel".to_string(),
            );
        }

        self.has_updated = true;

        let rect = global
            .atlas_texture
            .get_relative_texture_rect("profiel".to_string())
            .unwrap();
        let (top_left, top_right, bottom_left, bottom_right) = rect.bounds();

        vec![
            Vertex::new(-50.0, -50.0, 0.01, bottom_left.0, bottom_left.1, 1),
            Vertex::new(50.0, -50.0, 0.01, bottom_right.0, bottom_right.1, 1),
            Vertex::new(-50.0, 50.0, 0.01, top_left.0, top_left.1, 1),
            Vertex::new(50.0, -50.0, 0.01, bottom_right.0, bottom_right.1, 1),
            Vertex::new(50.0, 50.0, 0.01, top_right.0, top_right.1, 1),
            Vertex::new(-50.0, 50.0, 0.01, top_left.0, top_left.1, 1),
        ]
    }

    fn get_name(&self) -> String {
        "TEXTURE TEST".to_string()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
