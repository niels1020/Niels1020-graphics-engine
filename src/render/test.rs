#[cfg(test)]
mod tests {
    #[test]
    fn smoke_compile_render_modules() {
        // smoke test to ensure render submodules are reachable and compile
        let _ = crate::render::render_2d::camera::Camera2D::new([1.0, 1.0]);
    }
}
