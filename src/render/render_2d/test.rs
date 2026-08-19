#[cfg(test)]
mod tests {
    use crate::render::render_2d::camera::Camera2D;

    #[test]
    fn camera_new_sets_resolution() {
        let c = Camera2D::new([800.0, 600.0]);
        assert_eq!(c.data.render_resolution, [800.0, 600.0]);
    }
}
