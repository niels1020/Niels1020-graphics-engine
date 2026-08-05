use crate::{
    common::Vertex,
    render::{
        atlas::{self, AtlasTexture, Rect},
        render_2d::layer::RenderObject2D,
    },
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

    fn get_vertices(&mut self, atlas: &mut AtlasTexture) -> Vec<crate::common::Vertex> {
        self.has_updated = true;
        vec![
            Vertex::new(400.0, 0.0, 0.0, 0.0, 0.0, 0),
            Vertex::new(400.0, 300.0, 0.0, 0.0, 0.0, 0),
            Vertex::new(0.0, 300.0, 0.0, 0.0, 0.0, 0),
            Vertex::new(-400.0, 0.0, 0.0, 0.0, 0.0, 0),
            Vertex::new(-400.0, -300.0, 0.0, 0.0, 0.0, 0),
            Vertex::new(0.0, -300.0, 0.0, 0.0, 0.0, 0),
        ]
    }

    fn get_name(&self) -> String {
        "VERTICES TEST".to_string()
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

    fn get_vertices(&mut self, atlas: &mut AtlasTexture) -> Vec<Vertex> {
        if !self.image_added {
            self.image_added = true;
            atlas.add_image(
                image::load_from_memory(include_bytes!(
                    "./../../../../assets/test_textures/profiel.png"
                ))
                .unwrap(),
                "profiel".to_string(),
            );
        }

        self.has_updated = true;

        let rect = atlas.get_relative_texture_rect("profiel".to_string());
        let (top_left, top_right, bottom_left, bottom_right) = rect.bounds();

        vec![
            Vertex::new(-50.0, -50.0, 0.0, bottom_left.0, bottom_left.1, 1),
            Vertex::new(50.0, -50.0, 0.0, bottom_right.0, bottom_right.1, 1),
            Vertex::new(-50.0, 50.0, 0.0, top_left.0, top_left.1, 1),
            Vertex::new(50.0, -50.0, 0.0, bottom_right.0, bottom_right.1, 1),
            Vertex::new(50.0, 50.0, 0.0, top_right.0, top_right.1, 1),
            Vertex::new(-50.0, 50.0, 0.0, top_left.0, top_left.1, 1),
        ]
    }

    fn get_name(&self) -> String {
        "TEXTURE TEST".to_string()
    }
}
