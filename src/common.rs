use bytemuck::NoUninit;
use winit::window::WindowId;

use crate::logic::game_window::InputHandler;

// ============================================================================
// Constants
// ============================================================================

/// Clear color for the render pass (dark blue-gray)
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.3,
    a: 1.0,
};

/// Camera movement speed (units per frame) delta is in microseconds (sorry)
pub const CAMERA_SPEED: f32 = 0.00005;

/// Depth value for clearing - set to 1.0 (far plane) since we use reverse-Z
pub const DEPTH_CLEAR_VALUE: f32 = 1.0;

/// Maximum frame latency to aim for
pub const MAX_FRAME_LATENCY: u32 = 2;

/// Default camera position (units: 1 up, 2 back from origin)
pub const DEFAULT_CAMERA_EYE: (f32, f32, f32) = (0.0, 1.0, 2.0);

/// Default point the camera looks at
pub const DEFAULT_CAMERA_TARGET: (f32, f32, f32) = (0.0, 0.0, 0.0);

/// Camera field of view in degrees
pub const CAMERA_FOV: f32 = 90.0;

/// Camera near plane distance
pub const CAMERA_NEAR_PLANE: f32 = 0.1;

/// Camera far plane distance
pub const CAMERA_FAR_PLANE: f32 = 1000.0; //maybe to far

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum CompareFunction {
    Undefined = 0,
    Never = 1,
    Less = 2,
    Equal = 3,
    LessEqual = 4,
    Greater = 5,
    NotEqual = 6,
    GreaterEqual = 7,
    Always = 8,
}

/// Geometric data for a vertex (position and texture coordinates)
#[repr(C)]
#[derive(Copy, Clone, Debug, NoUninit)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    /// Get the vertex buffer layout descriptor for use in render pipelines
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }

    /// Create a new vertex with the given position and texture coordinates
    pub fn new(x: f32, y: f32, z: f32, t_x: f32, t_y: f32) -> Self {
        Self {
            position: [x, y, z],
            tex_coords: [t_x, t_y],
        }
    }
}

pub type Commands = Vec<Command>;

//TODO: make into a struct to call the commands on
pub enum Command {
    CloseWindow(WindowId),
    Exit,
    NewWindow(Box<dyn InputHandler>),
}