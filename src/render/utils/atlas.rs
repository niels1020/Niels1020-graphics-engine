use std::collections::HashMap;

use image::{DynamicImage, GenericImage, GenericImageView};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Device, Queue, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::render::utils::texture::Texture;

pub struct Rect {
    pub top_left: [f32; 2],
    pub bottom_right: [f32; 2],
}

pub struct AtlasTexture {
    need_update: bool,
    pub merged_texture: Option<Texture>,
    positions: Vec<Rect>, //array whitch contains the area each texture holds in the merged texture
    relative_positions: Vec<Rect>,
    size: (u32, u32),

    images: Vec<DynamicImage>,
    name_id_lookup: HashMap<String, usize>, //lookup for whitch texture name has whitch position in images and positions
}

impl AtlasTexture {
    pub fn new() -> Self {
        Self {
            need_update: false,
            merged_texture: None,
            images: vec![],
            positions: vec![],
            name_id_lookup: HashMap::new(),
            relative_positions: vec![],
            size: (0, 0),
        }
    }

    pub(crate) fn build(&mut self, queue: &Queue, device: &Device) {
        self.need_update = false;

        self.build_positions();
        let (needed_width, needed_height) = self.size;

        //check for empty atlas
        let merged_image = if needed_height == 0 || needed_width == 0 {
            self.size = (1, 1);
            DynamicImage::new(1, 1, image::ColorType::Rgba8)
        } else {
            let mut merged_image =
                DynamicImage::new(needed_width, needed_height, image::ColorType::Rgba8);
            for (i, img) in self.images.iter().enumerate() {
                let top_left = self.positions.get(i).unwrap().top_left;
                merged_image
                    .copy_from(img, top_left[0] as u32, top_left[1] as u32)
                    .unwrap();
            }
            merged_image
        };

        //uploading as texture
        self.merged_texture = Some(Texture::from_image(device, queue, &merged_image, None));
    }

    pub fn add_image(&mut self, img: DynamicImage, name: String) -> usize{
        self.need_update = true;
        let id = self.images.len();
        self.images.push(img);
        self.name_id_lookup.insert(name, id);
        id
    }

    pub fn remove_image(&mut self, name: String) -> Result<(), String> {
        self.need_update = true;
        match self.name_id_lookup.remove(&name) {
            Some(id) => {
                let last_id = self.images.len().saturating_sub(1);

                if id < self.images.len() {
                    self.images.swap_remove(id);
                }

                // If the removed slot was not the last one, the image that moved into it
                // must keep its original name lookup record, and every later ID must be
                // shifted back by one because the image vector is now compacted.
                if id != last_id && !self.images.is_empty() {
                    let swapped_name = self
                        .name_id_lookup
                        .iter()
                        .find_map(|(candidate_name, candidate_id)| {
                            if *candidate_id == last_id {
                                Some(candidate_name.clone())
                            } else {
                                None
                            }
                        });

                    if let Some(swapped_name) = swapped_name {
                        self.name_id_lookup.insert(swapped_name, id);
                    }

                    for (_, candidate_id) in self.name_id_lookup.iter_mut() {
                        if *candidate_id > id {
                            *candidate_id -= 1;
                        }
                    }
                }

                Ok(())
            }
            None => Err(format!("The image {} doesn't exist in this atlas", name)),
        }
    }

    pub fn build_if_needed(&mut self, queue: &Queue, device: &Device) -> bool {
        if self.need_update == true {
            self.build(queue, device);
            true
        } else {
            false
        }
    }

    pub fn get_relative_texture_rect(&mut self, name: String) -> Result<&Rect, String> {
        let _ = self.build_positions();
        match self.name_id_lookup.get(&name) {
            Some(id) => match self.relative_positions.get(*id) {
                Some(rect) => Ok(rect),
                None => Err(format!(
                    "The key {} exists but there isn't a linked rect. Please make sure you have called build_positions() or build_if_needed()",
                    name
                )),
            },
            None => Err(format!("Can't find key {}", name)),
        }
    }

    fn build_positions(&mut self) {
        let mut needed_height = 0;
        let mut needed_width = 0;

        self.positions.clear();

        for img in self.images.iter() {
            let dimensions = img.dimensions();
            if needed_height < dimensions.1 {
                needed_height = dimensions.1;
            }

            self.positions.push(Rect {
                top_left: [needed_width as f32, 0.0],
                bottom_right: [(needed_width + dimensions.0) as f32, dimensions.1 as f32],
            });

            needed_width += dimensions.0;
        }

        self.size = (needed_width, needed_height);
        self.make_psoitions_relative();
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

    fn make_psoitions_relative(&mut self) {
        self.relative_positions = self
            .positions
            .iter()
            .map(|r| Rect {
                top_left: [
                    r.top_left[0] / self.size.0 as f32,
                    r.top_left[1] / self.size.1 as f32,
                ],
                bottom_right: [
                    r.bottom_right[0] / self.size.0 as f32,
                    r.bottom_right[1] / self.size.1 as f32,
                ],
            })
            .collect()
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
