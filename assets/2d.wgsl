// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
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
    out.clip_position = vec4<f32>((model.position.x - camera.position.x) / camera.render_resolution.x, (model.position.y - camera.position.y) / camera.render_resolution.y, model.position.z, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = vec4(in.clip_position);
    return color;
}
