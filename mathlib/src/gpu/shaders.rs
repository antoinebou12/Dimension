//! WGSL compute shaders for GPU operations.
//!
//! Each constant defines a complete WGSL shader. Modify these to tune workgroup sizes,
//! optimize for specific hardware, or add new operations.

pub(crate) const WGSL_MATMUL: &str = r"
struct Params {
    M: u32,
    K: u32,
    N: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let j = gid.y;
    if (i >= params.M || j >= params.N) {
        return;
    }
    var sum = 0.0;
    for (var k = 0u; k < params.K; k++) {
        sum += A[k * params.M + i] * B[j * params.K + k];
    }
    C[j * params.M + i] = sum;
}
";

/// Same as WGSL_MATMUL but with 16×16 workgroup for better GPU utilization when device limits allow.
pub(crate) const WGSL_MATMUL_16: &str = r"
struct Params {
    M: u32,
    K: u32,
    N: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let j = gid.y;
    if (i >= params.M || j >= params.N) {
        return;
    }
    var sum = 0.0;
    for (var k = 0u; k < params.K; k++) {
        sum += A[k * params.M + i] * B[j * params.K + k];
    }
    C[j * params.M + i] = sum;
}
";

/// Tiled matmul with workgroup shared memory (16×16 tiles). Same binding layout as naive matmul.
/// A column-major M×K, B column-major K×N, C column-major M×N.
pub(crate) const WGSL_MATMUL_TILED_16: &str = r"
struct Params {
    M: u32,
    K: u32,
    N: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

var<workgroup> A_tile: array<f32, 256>;
var<workgroup> B_tile: array<f32, 256>;

@compute @workgroup_size(16, 16, 1)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wid: vec3<u32>,
) {
    let local_i = lid.x;
    let local_j = lid.y;
    let base_row = wid.y * 16u;
    let base_col = wid.x * 16u;
    var sum = 0.0;
    let num_tiles = (params.K + 15u) / 16u;
    for (var t = 0u; t < num_tiles; t++) {
        let t_base = t * 16u;
        let a_col = t_base + local_j;
        let a_row = base_row + local_i;
        let b_row = t_base + local_i;
        let b_col = base_col + local_j;
        var a_val = 0.0;
        var b_val = 0.0;
        if (a_row < params.M && a_col < params.K) {
            a_val = A[a_col * params.M + a_row];
        }
        if (b_row < params.K && b_col < params.N) {
            b_val = B[b_col * params.K + b_row];
        }
        A_tile[local_i * 16u + local_j] = a_val;
        B_tile[local_i * 16u + local_j] = b_val;
        workgroupBarrier();
        for (var k = 0u; k < 16u; k++) {
            let k_base = t_base + k;
            if (k_base < params.K) {
                sum += A_tile[local_i * 16u + k] * B_tile[k * 16u + local_j];
            }
        }
        workgroupBarrier();
    }
    let i = base_row + local_i;
    let j = base_col + local_j;
    if (i < params.M && j < params.N) {
        C[j * params.M + i] = sum;
    }
}
";

pub(crate) const WGSL_DOT: &str = r"
struct DotParams {
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;
@group(0) @binding(3) var<uniform> params: DotParams;

var<workgroup> partial: array<f32, 256>;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(local_invocation_id) lid: vec3<u32>) {
    let i = lid.x;
    var sum = 0.0;
    var idx = i;
    while (idx < params.n) {
        sum += A[idx] * B[idx];
        idx += 256u;
    }
    partial[i] = sum;
    workgroupBarrier();
    var stride = 128u;
    for (var s = 0; s < 8; s++) {
        if (i < stride) {
            partial[i] += partial[i + stride];
        }
        workgroupBarrier();
        stride = stride / 2u;
    }
    if (i == 0u) {
        result[0] = partial[0];
    }
}
";

pub(crate) const WGSL_MATVEC: &str = r"
struct MatvecParams {
    m: u32,
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> v: array<f32>;
@group(0) @binding(2) var<storage, read_write> y: array<f32>;
@group(0) @binding(3) var<uniform> params: MatvecParams;

var<workgroup> v_tile: array<f32, 256>;

@compute @workgroup_size(256, 1, 1)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let row = gid.x;
    if (row >= params.m) {
        return;
    }
    var sum = 0.0;
    let num_tiles = (params.n + 255u) / 256u;
    for (var t = 0u; t < num_tiles; t++) {
        let k_base = t * 256u;
        let local_k = lid.x;
        let k = k_base + local_k;
        if (k < params.n) {
            v_tile[local_k] = v[k];
        } else {
            v_tile[local_k] = 0.0;
        }
        workgroupBarrier();
        for (var local_k = 0u; local_k < 256u; local_k++) {
            let k = k_base + local_k;
            if (k < params.n) {
                sum += A[row + k * params.m] * v_tile[local_k];
            }
        }
        workgroupBarrier();
    }
    y[row] = sum;
}
";

/// Matvec with warp-sized workgroup (32 threads, tile 32). Same binding layout as WGSL_MATVEC.
/// Use when device max_compute_workgroup_size_x < 256 or for better warp alignment.
pub(crate) const WGSL_MATVEC_32: &str = r"
struct MatvecParams {
    m: u32,
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> v: array<f32>;
@group(0) @binding(2) var<storage, read_write> y: array<f32>;
@group(0) @binding(3) var<uniform> params: MatvecParams;

var<workgroup> v_tile: array<f32, 32>;

@compute @workgroup_size(32, 1, 1)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let row = gid.x;
    if (row >= params.m) {
        return;
    }
    var sum = 0.0;
    let num_tiles = (params.n + 31u) / 32u;
    for (var t = 0u; t < num_tiles; t++) {
        let k_base = t * 32u;
        let local_k = lid.x;
        let k = k_base + local_k;
        if (k < params.n) {
            v_tile[local_k] = v[k];
        } else {
            v_tile[local_k] = 0.0;
        }
        workgroupBarrier();
        for (var local_k = 0u; local_k < 32u; local_k++) {
            let k = k_base + local_k;
            if (k < params.n) {
                sum += A[row + k * params.m] * v_tile[local_k];
            }
        }
        workgroupBarrier();
    }
    y[row] = sum;
}
";

pub(crate) const WGSL_ADD: &str = r"
struct AddParams {
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: AddParams;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) {
        return;
    }
    C[i] = A[i] + B[i];
}
";

/// Vectorized add: 4 elements per thread for better memory throughput.
pub(crate) const WGSL_ADD_VEC4: &str = r"
struct AddParams {
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: AddParams;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let base = gid.x * 4u;
    if (base >= params.n) { return; }
    for (var j = 0u; j < 4u; j++) {
        let i = base + j;
        if (i < params.n) {
            C[i] = A[i] + B[i];
        }
    }
}
";

pub(crate) const WGSL_SUB: &str = r"
struct SubParams {
    n: u32,
}

@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: SubParams;

@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) {
        return;
    }
    C[i] = A[i] - B[i];
}
";

pub(crate) const WGSL_MUL: &str = r"
struct MulParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: MulParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = A[i] * B[i];
}
";

pub(crate) const WGSL_SCALE: &str = r"
struct ScaleParams { n: u32, alpha: f32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read_write> C: array<f32>;
@group(0) @binding(2) var<uniform> params: ScaleParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = params.alpha * A[i];
}
";

/// Vectorized scale: 4 elements per thread for better memory throughput.
pub(crate) const WGSL_SCALE_VEC4: &str = r"
struct ScaleParams { n: u32, alpha: f32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read_write> C: array<f32>;
@group(0) @binding(2) var<uniform> params: ScaleParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let base = gid.x * 4u;
    if (base >= params.n) { return; }
    for (var j = 0u; j < 4u; j++) {
        let i = base + j;
        if (i < params.n) {
            C[i] = params.alpha * A[i];
        }
    }
}
";

pub(crate) const WGSL_AXPY: &str = r"
struct AxpyParams { n: u32, alpha: f32, }
@group(0) @binding(0) var<storage, read> x: array<f32>;
@group(0) @binding(1) var<storage, read> y: array<f32>;
@group(0) @binding(2) var<storage, read_write> z: array<f32>;
@group(0) @binding(3) var<uniform> params: AxpyParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    z[i] = params.alpha * x[i] + y[i];
}
";

pub(crate) const WGSL_ABS: &str = r"
struct AbsParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read_write> C: array<f32>;
@group(0) @binding(2) var<uniform> params: AbsParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = abs(A[i]);
}
";

pub(crate) const WGSL_SQRT: &str = r"
struct SqrtParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read_write> C: array<f32>;
@group(0) @binding(2) var<uniform> params: SqrtParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = sqrt(A[i]);
}
";

pub(crate) const WGSL_DIV: &str = r"
struct DivParams { n: u32, }
@group(0) @binding(0) var<storage, read> A: array<f32>;
@group(0) @binding(1) var<storage, read> B: array<f32>;
@group(0) @binding(2) var<storage, read_write> C: array<f32>;
@group(0) @binding(3) var<uniform> params: DivParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    C[i] = select(0.0, A[i] / B[i], B[i] != 0.0);
}
";

pub(crate) const WGSL_SPMV: &str = r"
struct SpmvParams { m: u32, n: u32, nnz: u32, }
@group(0) @binding(0) var<storage, read> row_ptr: array<u32>;
@group(0) @binding(1) var<storage, read> col_ind: array<u32>;
@group(0) @binding(2) var<storage, read> values: array<f32>;
@group(0) @binding(3) var<storage, read> x: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;
@group(0) @binding(5) var<uniform> params: SpmvParams;
@compute @workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.m) { return; }
    var sum = 0.0;
    let start = row_ptr[i];
    let end = row_ptr[i + 1u];
    for (var k = start; k < end; k++) {
        sum += values[k] * x[col_ind[k]];
    }
    y[i] = sum;
}
";
