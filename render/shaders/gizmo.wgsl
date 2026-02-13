// Gizmo rotation rings: ring-shaped alpha and optional axis highlight (u_active).
// Vertex: position, normal, color (vec3), component (axis id vec3), texcoord (vec2).

struct GizmoUniforms {
    model_view: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    disk_width_rel: f32,
    _pad: vec3<f32>,
    active_axes: vec3<f32>,
    _pad2: f32,
}

@group(0) @binding(0)
var<uniform> gizmo_uniforms: GizmoUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) component: vec3<f32>,
    @location(4) texcoord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal_view: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) component: vec3<f32>,
    @location(3) texcoord: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = gizmo_uniforms.proj_matrix * gizmo_uniforms.model_view * vec4<f32>(in.position, 1.0);
    out.normal_view = (gizmo_uniforms.model_view * vec4<f32>(in.normal, 0.0)).xyz;
    out.color = in.color;
    out.component = in.component;
    out.texcoord = in.texcoord;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let disk_width = gizmo_uniforms.disk_width_rel;
    let disk_rad = 1.0 - disk_width;
    let point_rad = length(in.texcoord);
    let dist_from_ring = abs(point_rad - disk_rad);
    let ring_factor = dist_from_ring / disk_width;
    var shade_factor = 1.0;
    if (ring_factor > 1.0) {
        shade_factor = 0.0;
    }
    let alpha_out = shade_factor;
    if (alpha_out == 0.0) {
        discard;
    }
    var albedo_color = in.color;
    let active_mask = in.component * gizmo_uniforms.active_axes;
    let is_active = (active_mask.x + active_mask.y + active_mask.z) != 0.0;
    if (is_active) {
        albedo_color = mix(albedo_color, vec3(1.0, 1.0, 1.0), 0.3);
    }
    return vec4<f32>(albedo_color, alpha_out);
}
