// UI pass: screen-space orthographic (pixel coords), vertex color with alpha.
// Quads only; text is rendered separately via wgpu_text.
// Vertex layout: position (vec3), uv (vec2), color (vec4), rect_min (vec2), rect_max (vec2), corner_radius (f32).
// When corner_radius > 0, fragment shader draws a rounded rect.

struct Uniforms {
    ortho: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) rect_min: vec2<f32>,
    @location(4) rect_max: vec2<f32>,
    @location(5) corner_radius: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) position_px: vec2<f32>,
    @location(2) rect_min: vec2<f32>,
    @location(3) rect_max: vec2<f32>,
    @location(4) corner_radius: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.ortho * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.position_px = in.position.xy;
    out.rect_min = in.rect_min;
    out.rect_max = in.rect_max;
    out.corner_radius = in.corner_radius;
    return out;
}

// SDF for rounded box (center, half-size, radius). Returns distance; negative = inside.
fn rounded_box_sdf(p: vec2<f32>, center: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p - center) - half_size + r;
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.corner_radius <= 0.0 {
        return in.color;
    }
    let center = (in.rect_min + in.rect_max) * 0.5;
    let half_size = max((in.rect_max - in.rect_min) * 0.5 - in.corner_radius, vec2<f32>(0.0, 0.0));
    let d = rounded_box_sdf(in.position_px, center, half_size, in.corner_radius);
    if d > 0.0 {
        discard;
    }
    return in.color;
}
