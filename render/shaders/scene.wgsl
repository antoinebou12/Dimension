// Scene pass: 3D/2D primitives with MVP transform and vertex color.
// Vertex buffer 0: position (vec3), uv (vec2), color (vec4). Buffer 1 (instance): mvp, model_view, material_selected, entity_color.
// Used for all scene primitives. Instancing: one draw per (mesh, material) island.
// Bind group 0: frame uniforms, colormap, sampler (no per-draw object uniform).

struct FrameUniforms {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    colormap_mode: u32,
    exposure: f32,
    gamma: f32,
    ambient: f32,
    num_lights: u32,
    _pad: vec3<u32>,
    light2_dir: vec4<f32>,
    selection_time: f32,
    _pad_selection: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

@group(0) @binding(1)
var colormap: texture_2d<f32>;

@group(0) @binding(2)
var colormap_sampler: sampler;

// Material bind group: 4 matcap textures (R,G,B,K) + sampler. Static uses same texture for all 4.
@group(1) @binding(0) var mat_r: texture_2d<f32>;
@group(1) @binding(1) var mat_g: texture_2d<f32>;
@group(1) @binding(2) var mat_b: texture_2d<f32>;
@group(1) @binding(3) var mat_k: texture_2d<f32>;
@group(1) @binding(4) var mat_sampler: sampler;

// Locations 0–2: vertex (position, uv, color). 3–5 reserved for Vertex UI fields when same layout is used elsewhere.
// Locations 6–15: instance data (mvp, model_view, material_selected, entity_color).
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(6) mvp_0: vec4<f32>,
    @location(7) mvp_1: vec4<f32>,
    @location(8) mvp_2: vec4<f32>,
    @location(9) mvp_3: vec4<f32>,
    @location(10) model_view_0: vec4<f32>,
    @location(11) model_view_1: vec4<f32>,
    @location(12) model_view_2: vec4<f32>,
    @location(13) model_view_3: vec4<f32>,
    @location(14) material_selected: vec4<u32>,
    @location(15) entity_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) view_pos: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) entity_color: vec4<f32>,
    @location(4) @interpolate(flat) material_mode: u32,
    @location(5) @interpolate(flat) selected: u32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let mvp = mat4x4<f32>(in.mvp_0, in.mvp_1, in.mvp_2, in.mvp_3);
    let model_view = mat4x4<f32>(in.model_view_0, in.model_view_1, in.model_view_2, in.model_view_3);
    out.clip_position = mvp * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    out.view_pos = (model_view * vec4<f32>(in.position, 1.0)).xyz;
    out.uv = in.uv;
    out.entity_color = in.entity_color;
    out.material_mode = in.material_selected.x;
    out.selected = in.material_selected.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n_cross = cross(dpdx(in.view_pos), dpdy(in.view_pos));
    let n = select(normalize(n_cross), vec3<f32>(0.0, 0.0, 1.0), length(n_cross) < 1e-6);
    if (frame.colormap_mode == 2u) {
        return vec4<f32>((n + vec3<f32>(1.0)) * 0.5, 1.0);
    }
    var base_color = in.color;
    if (frame.colormap_mode == 1u) {
        let scalar = clamp(in.color.r, 0.0, 1.0);
        base_color = textureSample(colormap, colormap_sampler, vec2(scalar, 0.5));
        base_color.a = in.color.a;
    }
    // Sample all material textures in uniform control flow (required by WebGPU).
    let matcap_uv = vec2(n.x * 0.5 + 0.5, 1.0 - (n.y * 0.5 + 0.5));
    let mat_r_matcap = textureSample(mat_r, mat_sampler, matcap_uv);
    let mat_g_matcap = textureSample(mat_g, mat_sampler, matcap_uv);
    let mat_b_matcap = textureSample(mat_b, mat_sampler, matcap_uv);
    let mat_k_matcap = textureSample(mat_k, mat_sampler, matcap_uv);
    let mat_r_uv = textureSample(mat_r, mat_sampler, in.uv);
    // Select result by material_mode (branching on varying is fine for non-texture ops).
    if (in.material_mode == 1u) {
        base_color = mat_r_matcap;
        base_color.a = in.color.a;
    } else if (in.material_mode == 2u) {
        let t = in.entity_color;
        let w = 1.0 - t.r - t.g - t.b;
        base_color = t.r * mat_r_matcap + t.g * mat_g_matcap + t.b * mat_b_matcap + w * mat_k_matcap;
        base_color.a = in.color.a;
    } else if (in.material_mode == 3u) {
        base_color = mat_r_uv;
        base_color.a = in.color.a;
    } else if (in.material_mode == 0u) {
        base_color = in.color * in.entity_color;
    }

    let num_lights = frame.num_lights;
    if (num_lights == 0u) {
        return tone_map(base_color);
    }
    let len = length(in.view_pos);
    if (len < 0.001) {
        return tone_map(base_color);
    }
    var factor = frame.ambient;
    if (num_lights >= 1u && frame.light_dir.w > 0.0) {
        let l = normalize(frame.light_dir.xyz);
        let n_dot_l = max(dot(n, l), 0.0);
        factor += (1.0 - frame.ambient) * n_dot_l * frame.light_dir.w;
    }
    if (num_lights >= 2u && frame.light2_dir.w > 0.0) {
        let l2 = normalize(frame.light2_dir.xyz);
        let n_dot_l2 = max(dot(n, l2), 0.0);
        factor += (1.0 - frame.ambient) * n_dot_l2 * frame.light2_dir.w;
    }
    factor = min(factor, 1.0);
    factor = max(factor, 0.2);
    var lit = vec4<f32>(base_color.rgb * factor, base_color.a);
    if (in.selected != 0u) {
        let pulse = 0.6 + 0.4 * sin(frame.selection_time * 4.0);
        let highlight = vec4<f32>(1.0, 1.0, 0.6, 1.0);
        lit = mix(lit, highlight, pulse * 0.55);
        let view_dir = normalize(in.view_pos);
        let rim = pow(max(0.0, 1.0 - dot(n, -view_dir)), 3.0);
        lit = mix(lit, vec4<f32>(1.0, 0.95, 0.7, 1.0), rim * 0.35);
    }
    return tone_map(lit);
}

fn tone_map(c: vec4<f32>) -> vec4<f32> {
    let exposed = c * frame.exposure;
    let inv_gamma = 1.0 / max(frame.gamma, 0.001);
    return vec4<f32>(
        pow(clamp(exposed.r, 0.0, 1.0), inv_gamma),
        pow(clamp(exposed.g, 0.0, 1.0), inv_gamma),
        pow(clamp(exposed.b, 0.0, 1.0), inv_gamma),
        c.a,
    );
}
