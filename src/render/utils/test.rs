#[cfg(test)]
mod tests {
    use crate::render::utils::atlas::AtlasTexture;
    use image::DynamicImage;

    #[test]
    fn atlas_add_get_remove() {
        let mut atlas = AtlasTexture::new();
        atlas.add_image(DynamicImage::new_rgba8(2, 2), "a".to_string());
        assert!(atlas.get_relative_texture_rect("a".to_string()).is_ok());
        assert!(atlas.remove_image("a".to_string()).is_ok());
        assert!(atlas.get_relative_texture_rect("a".to_string()).is_err());
    }
}
