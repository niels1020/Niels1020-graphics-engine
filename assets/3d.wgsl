struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct Transform {
    position: vec3<f32>,
    rotation: vec3<f32>, // in DEGREES
};

@group(2) @binding(0)
var<uniform> transform: Transform;

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

fn deg_to_rad(d: f32) -> f32 {
    return radians(d);
}

fn rotation_matrix_x(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, c,   -s),
        vec3<f32>(0.0, s,    c),
    );
}

fn rotation_matrix_y(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(c,   0.0, s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-s,  0.0, c),
    );
}

fn rotation_matrix_z(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(c, -s, 0.0),
        vec3<f32>(s,  c, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
    );
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    // Convert degrees to radians
    let rx = deg_to_rad(transform.rotation.x);
    let ry = deg_to_rad(transform.rotation.y);
    let rz = deg_to_rad(transform.rotation.z);

    // Build rotation matrix (Z * Y * X order)
    let rot = rotation_matrix_z(rz) *
              rotation_matrix_y(ry) *
              rotation_matrix_x(rx);

    // Apply rotation + translation
    let world_pos = rot * model.position + transform.position;

    // Apply camera
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);

    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
