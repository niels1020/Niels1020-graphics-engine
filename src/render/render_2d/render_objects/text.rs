use std::arch::global_asm;

use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, SwashCache};
use image::{Rgba, RgbaImage};

use crate::{
    common::Vertex,
    render::render_2d::layer::{RenderLayer2DGlobal, RenderObject2D},
};

pub struct Text {
    have_vertices_changed: bool,
    font_name: String,
    text: String,
    color: Color,
    old_image_name: Option<String>,
    scale: f32,
    vertices: Vec<Vertex>,
}

impl RenderObject2D for Text {
    fn have_vertices_changed(&mut self) -> bool {
        self.have_vertices_changed
    }

    fn get_vertices(&mut self, global: &mut RenderLayer2DGlobal) -> Vec<crate::common::Vertex> {
        if self.have_vertices_changed {
            if let Some(old) = self.old_image_name.as_ref() {
                global
                    .atlas_texture
                    .remove_image(old.clone())
                    .expect("couldn't remove something from atlas that should exist");
            }
            self.have_vertices_changed = false;

            if self.text.is_empty() {
                self.old_image_name = None;
                return vec![];
            }
            //render to an image

            let mut buffer = Buffer::new(
                &mut global.renderer_global.font_system,
                Metrics::new(self.scale, self.scale * 1.2),
            );
            let attrs = Attrs::new().family(cosmic_text::Family::Name(&self.font_name));

            buffer.set_text(&self.text, &attrs, cosmic_text::Shaping::Advanced, None);

            let mut min_x = i32::MAX;
            let mut min_y = i32::MAX;
            let mut max_x = i32::MIN;
            let mut max_y = i32::MIN;

            let mut pixels = Vec::new();

            buffer.draw(
                &mut global.renderer_global.font_system,
                &mut global.renderer_global.text_swash_cache,
                self.color,
                |x, y, w, h, color| {
                    // Ignore completely transparent pixels.
                    if color.a() == 0 {
                        return;
                    }

                    for py in 0..h {
                        for px in 0..w {
                            let pixel_x = x + px as i32;
                            let pixel_y = y + py as i32;

                            min_x = min_x.min(pixel_x);
                            min_y = min_y.min(pixel_y);
                            max_x = max_x.max(pixel_x);
                            max_y = max_y.max(pixel_y);

                            pixels.push((pixel_x, pixel_y, color));
                        }
                    }
                },
            );

            if pixels.is_empty() {
                return vec![];
            }

            let glyphs_width = (max_x - min_x + 1) as u32;
            let glyphs_height = (max_y - min_y + 1) as u32;

            let mut image = RgbaImage::new(glyphs_width, glyphs_height);

            for (x, y, color) in pixels {
                let image_x = (x - min_x) as u32;
                let image_y = (y - min_y) as u32;

                image.put_pixel(image_x, image_y, Rgba(color.as_rgba()));
            }

            let name = format!(
                "{},{},{},{:?}",
                self.font_name, self.text, self.scale, self.color
            );
            self.old_image_name = Some(name.clone());
            let _id = global.atlas_texture.add_image(image.into(), name.clone());
            let (top_left, top_right, bottom_left, bottom_right) = global
                .atlas_texture
                .get_relative_texture_rect(name)
                .unwrap()
                .bounds();

            let half_res = [global.render_res[0] / 2.0, global.render_res[1] / 2.0];
            self.vertices = vec![
                Vertex::new(
                    0.0 - half_res[0],
                    0.0 - glyphs_height as f32 + half_res[1],
                    0.0,
                    bottom_left.0,
                    bottom_left.1,
                    1,
                ),
                Vertex::new(
                    glyphs_width as f32 - half_res[0],
                    0.0 - glyphs_height as f32 + half_res[1],
                    0.0,
                    bottom_right.0,
                    bottom_right.1,
                    1,
                ),
                Vertex::new(
                    0.0 - half_res[0],
                    0.0 + half_res[1],
                    0.0,
                    top_left.0,
                    top_left.1,
                    1,
                ),
                Vertex::new(
                    glyphs_width as f32 - half_res[0],
                    0.0 - glyphs_height as f32 + half_res[1],
                    0.0,
                    bottom_right.0,
                    bottom_right.1,
                    1,
                ),
                Vertex::new(
                    glyphs_width as f32 - half_res[0],
                    0.0 + half_res[1],
                    0.0,
                    top_right.0,
                    top_right.1,
                    1,
                ),
                Vertex::new(
                    0.0 - half_res[0],
                    0.0 + half_res[1],
                    0.0,
                    top_left.0,
                    top_left.1,
                    1,
                ),
            ]
        }
        self.vertices.clone()
    }

    fn get_name(&self) -> String {
        format!("text: {}", self.text)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Text {
    //the font has to be installed on the users system
    pub fn new(font_name: String, text: String, scale: f32, color: (u8, u8, u8, u8)) -> Box<Self> {
        Box::new(Self {
            have_vertices_changed: true,
            font_name,
            text: text,
            color: Color::rgba(color.0, color.1, color.2, color.3),
            old_image_name: None,
            scale,
            vertices: vec![],
        })
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.have_vertices_changed = true;
    }

    pub fn get_text(self) -> String {
        self.text
    }

    pub fn set_colour(&mut self, color: (u8, u8, u8, u8)) {
        self.color = Color::rgba(color.0, color.1, color.2, color.3);
        self.have_vertices_changed = true;
    }

    pub fn get_colour(self) -> (u8, u8, u8, u8) {
        self.color.as_rgba_tuple()
    }

    pub fn scale(&mut self, scale: f32) {
        self.scale = scale;
        self.have_vertices_changed = true;
    }

    pub fn get_scale(self) -> f32 {
        self.scale
    }
}
