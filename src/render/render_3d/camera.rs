use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

use crate::common::{
    CAMERA_FAR_PLANE, CAMERA_FOV, CAMERA_NEAR_PLANE, DEFAULT_CAMERA_EYE, DEFAULT_CAMERA_TARGET,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Clone, Debug)]
pub struct Camera3D {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {
            eye: DEFAULT_CAMERA_EYE.into(),
            // Point camera at the origin
            target: DEFAULT_CAMERA_TARGET.into(),
            // Define which direction is "up"
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: 16.0 / 9.0,
            fovy: CAMERA_FOV,
            znear: CAMERA_NEAR_PLANE,
            zfar: CAMERA_FAR_PLANE,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera3DUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Camera3DUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera3D) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

impl Camera3D {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = Perspective3::new(self.fovy.to_radians(), self.aspect, self.znear, self.zfar)
            .to_homogeneous();
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
