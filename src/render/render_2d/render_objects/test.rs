#[cfg(test)]
mod tests {
    use crate::render::render_2d::render_objects::text::Text;

    #[test]
    fn text_setters_getters() {
        // Test set_text and get_text (get_text consumes self)
        let mut t1 = Text::new("Arial".to_string(), "hello".to_string(), 12.0, (255, 255, 255, 255), (0.0, 0.0, 0.0));
        t1.set_text("world".to_string());
        assert_eq!(t1.get_text(), "world");

        // Test colour setter/getter
        let mut t2 = Text::new("Arial".to_string(), "".to_string(), 12.0, (255, 255, 255, 255), (0.0, 0.0, 0.0));
        t2.set_colour((1, 2, 3, 4));
        assert_eq!(t2.get_colour(), (1, 2, 3, 4));

        // Test scale setter/getter
        let mut t3 = Text::new("Arial".to_string(), "".to_string(), 1.0, (255, 255, 255, 255), (0.0, 0.0, 0.0));
        t3.scale(2.5);
        assert!((t3.get_scale() - 2.5).abs() < f32::EPSILON);

        // Test position setter/getter
        let mut t4 = Text::new("Arial".to_string(), "".to_string(), 1.0, (255, 255, 255, 255), (0.0, 0.0, 0.0));
        t4.set_position((1.0, 2.0, 3.0));
        assert_eq!(t4.get_position(), (1.0, 2.0, 3.0));
    }
}
