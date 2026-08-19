// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) @interpolate(flat) type_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) type_id: u32,
}

struct Camera {
    position: vec2<f32>,
    render_resolution: vec2<f32>
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(model: VertexInput,) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        (model.position.x - camera.position.x) / (camera.render_resolution.x / 2), 
        (model.position.y - camera.position.y) / (camera.render_resolution.y / 2),
         (model.position.z * -1.0) + 1.0, 
         1.0);
    out.tex_coords = model.tex_coords;
    out.type_id = model.type_id;
    return out;
}

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;
@group(1) @binding(1)
var s_atlas: sampler;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec4(0.5);
    if in.type_id == 1 {
        color = textureSample(t_atlas, s_atlas, in.tex_coords);
    }
    return color;
}
