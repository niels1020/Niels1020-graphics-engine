use cosmic_text::{Attrs, Buffer, Color, Metrics};
use image::{Rgba, RgbaImage};

use crate::{
    common::Vertex,
    render::render_2d::layer::{RenderLayer2DGlobal, RenderObject2D},
};

pub struct Text {
    pos_changed: bool,
    text_changed: bool,
    font_name: String,
    text: String,
    color: Color,
    old_image_name: Option<String>,
    scale: f32,
    pos: (f32, f32, f32),
    glyphs_width: u32,
    glyphs_height: u32,
}

impl RenderObject2D for Text {
    fn have_vertices_changed(&mut self) -> bool {
        self.text_changed | self.pos_changed
    }

    fn get_vertices(&mut self, global: &mut RenderLayer2DGlobal) -> Vec<crate::common::Vertex> {
        if self.text_changed {
            if let Some(old) = self.old_image_name.as_ref() {
                global
                    .atlas_texture
                    .remove_image(old.clone())
                    .expect("couldn't remove something from atlas that should exist");
            }
            self.text_changed = false;
            self.pos_changed = true;

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

            self.glyphs_width = (max_x - min_x + 1) as u32;
            self.glyphs_height = (max_y - min_y + 1) as u32;

            let mut image = RgbaImage::new(self.glyphs_width, self.glyphs_height);

            for (x, y, color) in pixels {
                let image_x = (x - min_x) as u32;
                let image_y = (y - min_y) as u32;

                image.put_pixel(image_x, image_y, Rgba(color.as_rgba()));
            }

            let name = format!(
                "{},{},{},{:?}, {:?}",
                self.font_name, self.text, self.scale, self.color, self.pos
            );
            self.old_image_name = Some(name.clone());
            let _id = global.atlas_texture.add_image(image.into(), name.clone());
            let (top_left, top_right, bottom_left, bottom_right) = global
                .atlas_texture
                .get_relative_texture_rect(name)
                .unwrap()
                .bounds();
            let half_width = self.glyphs_width as f32 / 2.0;
            let half_height = self.glyphs_height as f32 / 2.0;
            vec![
                Vertex::new(
                    -half_width + self.pos.0,
                    -half_height - half_height + self.pos.1,
                    self.pos.2,
                    bottom_left.0,
                    bottom_left.1,
                    1,
                ),
                Vertex::new(
                    half_width + self.pos.0,
                    -half_height - half_height + self.pos.1,
                    self.pos.2,
                    bottom_right.0,
                    bottom_right.1,
                    1,
                ),
                Vertex::new(
                    -half_width + self.pos.0,
                    half_height + self.pos.1,
                    self.pos.2,
                    top_left.0,
                    top_left.1,
                    1,
                ),
                Vertex::new(
                    half_width + self.pos.0,
                    -half_height - half_height + self.pos.1,
                    self.pos.2,
                    bottom_right.0,
                    bottom_right.1,
                    1,
                ),
                Vertex::new(
                    half_width + self.pos.0,
                    half_height + self.pos.1,
                    self.pos.2,
                    top_right.0,
                    top_right.1,
                    1,
                ),
                Vertex::new(
                    -half_width + self.pos.0,
                    half_height + self.pos.1,
                    self.pos.2,
                    top_left.0,
                    top_left.1,
                    1,
                ),
            ]
        } else {
            if self.old_image_name.is_none() {
                vec![]
            } else {
                let (top_left, top_right, bottom_left, bottom_right) = global
                    .atlas_texture
                    .get_relative_texture_rect(self.old_image_name.as_ref().unwrap().to_string())
                    .unwrap()
                    .bounds();
                let half_width = self.glyphs_width as f32 / 2.0;
                let half_height = self.glyphs_height as f32 / 2.0;
                vec![
                    Vertex::new(
                        -half_width + self.pos.0,
                        -half_height - half_height + self.pos.1,
                        self.pos.2,
                        bottom_left.0,
                        bottom_left.1,
                        1,
                    ),
                    Vertex::new(
                        half_width + self.pos.0,
                        -half_height - half_height + self.pos.1,
                        self.pos.2,
                        bottom_right.0,
                        bottom_right.1,
                        1,
                    ),
                    Vertex::new(
                        -half_width + self.pos.0,
                        half_height + self.pos.1,
                        self.pos.2,
                        top_left.0,
                        top_left.1,
                        1,
                    ),
                    Vertex::new(
                        half_width + self.pos.0,
                        -half_height - half_height + self.pos.1,
                        self.pos.2,
                        bottom_right.0,
                        bottom_right.1,
                        1,
                    ),
                    Vertex::new(
                        half_width + self.pos.0,
                        half_height + self.pos.1,
                        self.pos.2,
                        top_right.0,
                        top_right.1,
                        1,
                    ),
                    Vertex::new(
                        -half_width + self.pos.0,
                        half_height + self.pos.1,
                        self.pos.2,
                        top_left.0,
                        top_left.1,
                        1,
                    ),
                ]
            }
        }
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
    pub fn new(
        font_name: String,
        text: String,
        scale: f32,
        color: (u8, u8, u8, u8),
        pos: (f32, f32, f32),
    ) -> Box<Self> {
        Box::new(Self {
            text_changed: true,
            pos_changed: true,
            font_name,
            text: text,
            color: Color::rgba(color.0, color.1, color.2, color.3),
            old_image_name: None,
            scale,
            pos,
            glyphs_width: 1,
            glyphs_height: 1,
        })
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.text_changed = true;
    }

    pub fn get_text(self) -> String {
        self.text
    }

    pub fn set_colour(&mut self, color: (u8, u8, u8, u8)) {
        self.color = Color::rgba(color.0, color.1, color.2, color.3);
        self.text_changed = true;
    }

    pub fn get_colour(self) -> (u8, u8, u8, u8) {
        self.color.as_rgba_tuple()
    }

    pub fn scale(&mut self, scale: f32) {
        self.scale = scale;
        self.text_changed = true;
    }

    pub fn get_scale(self) -> f32 {
        self.scale
    }

    pub fn set_position(&mut self, pos: (f32, f32, f32)) {
        self.pos_changed = true;
        self.pos = pos
    }

    pub fn get_position(self) -> (f32, f32, f32) {
        self.pos
    }
}
