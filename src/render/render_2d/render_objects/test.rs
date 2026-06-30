use crate::{common::Vertex, render::render_2d::camera::RenderObject2D};

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
        if self.has_updated {
            false
        } else {
            self.has_updated = true;
            true
        }
    }

    fn get_vertices(&mut self) -> Vec<crate::common::Vertex> {
        vec![
            Vertex::new(1.0, 0.0, 0.0, 0.0, 0.0),
            Vertex::new(1.0, 1.0, 0.0, 0.0, 0.0),
            Vertex::new(0.0, 1.0, 0.0, 0.0, 0.0),
            Vertex::new(-1.0, 0.0, 0.0, 0.0, 0.0),
            Vertex::new(-1.0, -1.0, 0.0, 0.0, 0.0),
            Vertex::new(0.0, -1.0, 0.0, 0.0, 0.0),
        ]
    }

    fn get_name(&self) -> String {
        "VERTICES TEST".to_string()
    }
}
