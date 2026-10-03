use image::{DynamicImage, GenericImage};
use rect_packer::{Config, Packer};
use std::collections::HashMap;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Device, Queue, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::render::utils::texture::Texture;

#[derive(Clone, Debug)]
pub struct Rect {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
}

#[derive(Clone)]
pub struct AtlasTexture {
    need_update: bool,
    pub(crate) merged_texture: Option<Texture>,
    relative_positions: Vec<Rect>, //array whitch contains the area each texture holds in the merged texture relative to size

    images: Vec<DynamicImage>,
    name_id_lookup: HashMap<String, usize>, //lookup for whitch texture name has whitch position in images and positions

    config: Config,
    packer: Packer,
}

impl AtlasTexture {
    pub fn new() -> Self {
        let config = Config {
            width: 1024,
            height: 1024,
            border_padding: 0,
            rectangle_padding: 1,
        };
        Self {
            need_update: false,
            merged_texture: None,
            images: vec![],
            name_id_lookup: HashMap::new(),
            relative_positions: vec![],
            config,
            packer: Packer::new(config),
        }
    }

    pub(crate) fn build(&mut self, queue: &Queue, device: &Device) {
        self.need_update = false;

        let mut merged_image = DynamicImage::new(
            self.config.width as u32,
            self.config.height as u32,
            image::ColorType::Rgba8,
        );

        for i in 0..self.relative_positions.len() {
            let rel_rect = self.relative_positions.get(i).unwrap();
            merged_image.copy_from(
                self.images.get(i).unwrap(),
                (rel_rect.top_left[0] * self.config.width as f32) as u32,
                (rel_rect.top_left[1] * self.config.height as f32) as u32,
            ).unwrap();
        }

        //uploading as texture
        self.merged_texture = Some(Texture::from_image(device, queue, &merged_image, None));
    }

    pub fn add_image(&mut self, img: DynamicImage, name: String) -> usize {
        self.need_update = true;
        loop {
            let h = img.height();
            let w = img.width();

            if let Some(rect) = self.packer.pack(w as i32, h as i32, false) {
                let own_rect_rel = Rect {
                    top_left: [
                        rect.left() as f32 / self.config.width as f32,
                        rect.top() as f32 / self.config.height as f32,
                    ],
                    bottom_right: [
                        rect.right() as f32 / self.config.width as f32,
                        rect.bottom() as f32 / self.config.height as f32,
                    ],
                };
                self.relative_positions.push(own_rect_rel);
                let id = self.relative_positions.len() - 1;
                self.name_id_lookup.insert(name, id);
                self.images.push(img);
                return id;
            } else {
                println!("texture atlass to small expanding");
                self.config = Config {
                    height: self.config.height * 2,
                    width: self.config.width * 2,
                    ..self.config
                };
                self.packer = Packer::new(self.config);
                self.repack_all_images();
            }
        }
    }

    fn repack_all_images(&mut self) {
        for (i, img) in self.images.iter().enumerate() {
            let h = img.height();
            let w = img.width();

            if let Some(rect) = self.packer.pack(w as i32, h as i32, false) {
                let own_rect_rel = Rect {
                    top_left: [
                        rect.left() as f32 / self.config.width as f32,
                        rect.top() as f32 / self.config.height as f32,
                    ],
                    bottom_right: [
                        rect.right() as f32 / self.config.width as f32,
                        rect.bottom() as f32 / self.config.height as f32,
                    ],
                };
                self.relative_positions[i] = own_rect_rel;
                break;
            } else {
                panic!(
                    "packer size increased but images witch fitted in the previous one dont fit in the large one"
                )
            }
        }
    }

    pub fn remove_image(&mut self, name: String) -> Result<(), String> {
        if let Some(id) = self.name_id_lookup.get(&name) {
            self.images.remove(*id);
            self.relative_positions.remove(*id);
            self.name_id_lookup.remove(&name);
            Ok(())
        } else {
            Err("name doesnt exist in atlas".to_string())
        }
    }

    pub fn build_if_needed(&mut self, queue: &Queue, device: &Device) -> bool {
        if self.need_update {
            self.build(queue, device);
            true
        } else {
            false
        }
    }

    pub fn get_relative_texture_rect(&mut self, name: String) -> Result<&Rect, String> {
        if let Some(id) = self.name_id_lookup.get(&name) {
            Ok(self.relative_positions.get(*id).unwrap())
        } else {
            Err("name doesnt exist in atlas".to_string())
        }
    }

    //always binding 1
    pub fn create_layout(&mut self, device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("an atlas texture bindgroup layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn bind(&mut self, device: &Device) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("an atlas texture bindgroup"),
            layout: &self.create_layout(device),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self.merged_texture.as_ref().unwrap().view,
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &self.merged_texture.as_ref().unwrap().sampler,
                    ),
                },
            ],
        })
    }

    pub fn has_image(&self, name: String) -> bool {
        self.name_id_lookup.contains_key(&name)
    }
}

impl Rect {
    ///top_left, top_right, bottom_left, bottom_right
    pub fn bounds(&self) -> ((f32, f32), (f32, f32), (f32, f32), (f32, f32)) {
        return (
            (self.top_left[0], self.top_left[1]),
            (self.bottom_right[0], self.top_left[1]),
            (self.top_left[0], self.bottom_right[1]),
            (self.bottom_right[0], self.bottom_right[1]),
        );
    }
}
