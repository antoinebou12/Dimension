// Grid cube: instanced unit cube per cell. No geometry shader; vertex + instance buffer.

struct GridCubeUniforms {
    model_view: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
    grid_spacing: vec3<f32>,
    _pad: f32,
    cube_size_factor: f32,
    _pad2: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> grid_uniforms: GridCubeUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) instance_cell_position: vec3<f32>,
    @location(2) instance_cell_index: vec3<u32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) cell_index: vec3<u32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let dvec = 0.5 * grid_uniforms.grid_spacing * grid_uniforms.cube_size_factor;
    let world_pos = in.instance_cell_position + in.position * dvec;
    var out: VertexOutput;
    out.clip_position = grid_uniforms.proj_matrix * grid_uniforms.model_view * vec4<f32>(world_pos, 1.0);
    out.cell_index = in.instance_cell_index;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let shade = 0.5 + 0.5 * sin(f32(in.cell_index.x) * 0.1 + f32(in.cell_index.y) * 0.1 + f32(in.cell_index.z) * 0.1);
    return vec4<f32>(shade, shade, shade * 0.9, 1.0);
}
