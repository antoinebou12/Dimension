// Slice plane: quad with checker pattern and Oren-Nayar-style lighting.
// Vertex: position (vec4 in local plane space). Outputs clip position and world position.

struct SlicePlaneUniforms {
    view_matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    object_matrix: mat4x4<f32>,
    length_scale: f32,
    transparency: f32,
    _pad: vec2<f32>,
    color: vec3<f32>,
    _pad2: f32,
    grid_line_color: vec3<f32>,
    _pad3: f32,
}

@group(0) @binding(0)
var<uniform> slice_uniforms: SlicePlaneUniforms;

struct VertexInput {
    @location(0) position: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position_world_homog: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = slice_uniforms.proj_matrix * slice_uniforms.view_matrix * slice_uniforms.object_matrix * in.position;
    out.position_world_homog = slice_uniforms.object_matrix * in.position;
    return out;
}

fn oren_nayar_diffuse(light_dir: vec3<f32>, view_dir: vec3<f32>, normal: vec3<f32>, roughness: f32, albedo: f32) -> f32 {
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_v = max(dot(normal, view_dir), 0.0);
    let s = max(n_dot_l, n_dot_v);
    let t = min(n_dot_l, n_dot_v);
    let sigma2 = roughness * roughness;
    let a = 1.0 - 0.5 * sigma2 / (sigma2 + 0.33);
    let b = 0.45 * sigma2 / (sigma2 + 0.09);
    let v_par = view_dir - normal * n_dot_v;
    let l_par = light_dir - normal * n_dot_l;
    let cos_phi = clamp(dot(normalize(v_par), normalize(l_par)), -1.0, 1.0);
    return albedo * max(0.0, n_dot_l) * (a + b * cos_phi * sqrt(1.0 - s * s) / s);
}

fn specular(n: vec3<f32>, l: vec3<f32>, e: vec3<f32>, shininess: f32) -> f32 {
    let h = normalize(l + e);
    let n_dot_h = max(dot(n, h), 0.0);
    return pow(n_dot_h, shininess);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let basis_x = (slice_uniforms.object_matrix * vec4<f32>(1.0, 0.0, 0.0, 0.0)).xyz;
    let basis_y = (slice_uniforms.object_matrix * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz;
    let basis_z = (slice_uniforms.object_matrix * vec4<f32>(0.0, 0.0, 1.0, 0.0)).xyz;
    let coord = in.position_world_homog.xyz / in.position_world_homog.w;
    let coord_scaled = coord / (slice_uniforms.length_scale * 0.28);
    let coord_2d = vec2<f32>(dot(basis_y, coord_scaled), dot(basis_z, coord_scaled));
    let m1 = min(fract(coord_2d.x), fract(coord_2d.y));
    let m2 = min(fract(-coord_2d.x), fract(-coord_2d.y));
    let mod_dist = min(m1, m2);
    let stripe_blend = smoothstep(0.005, 0.02, mod_dist);
    let base_color = slice_uniforms.color;
    let ground_color = mix(slice_uniforms.grid_line_color, base_color, stripe_blend);
    let pos_camera = (slice_uniforms.view_matrix * in.position_world_homog).xyz / (slice_uniforms.view_matrix * in.position_world_homog).w;
    let normal_camera = (slice_uniforms.view_matrix * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz;
    let eye_camera = vec3<f32>(0.0, 0.0, 0.0);
    let light_pos_camera = vec3<f32>(5.0, 5.0, -5.0) * slice_uniforms.length_scale;
    let light_dir = normalize(light_pos_camera - pos_camera);
    let eye_dir = normalize(eye_camera - pos_camera);
    let colored_brightness = 1.2 * oren_nayar_diffuse(eye_dir, light_dir, normal_camera, 0.05, 1.0) + 0.3;
    let white_brightness = 0.25 * specular(normal_camera, light_dir, eye_dir, 12.0);
    let light_color = vec4<f32>(ground_color * colored_brightness + vec3<f32>(1.0, 1.0, 1.0) * white_brightness, slice_uniforms.transparency);
    return light_color;
}
