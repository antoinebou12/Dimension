/* tslint:disable */
/* eslint-disable */

/**
 * Label for DBSCAN noise points (not assigned to any cluster).
 * In JS this is the maximum 32-bit unsigned value when cast from u32.
 */
export function NOISE_LABEL(): number;

/**
 * Result of A*: path from start to goal, total distance, and predecessors.
 */
export class WasmAstarResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Total distance from start to goal; `Infinity` if no path.
     */
    getDist(): number;
    /**
     * Path from start to goal (empty if no path). Includes both start and goal.
     */
    getPath(): Uint32Array;
    /**
     * Predecessor on shortest path; -1 for start or unreachable.
     */
    getPredecessors(): Int32Array;
    /**
     * Reconstruct path from start to `target`. Returns array of node indices, or empty if unreachable.
     */
    pathTo(target: number): Uint32Array;
}

/**
 * Result of BFS: visit order and depth per node.
 */
export class WasmBfsResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Depth from source (usize::MAX for unreachable).
     */
    getDepth(): Uint32Array;
    /**
     * Visit order (nodes in discovery order).
     */
    getOrder(): Uint32Array;
}

/**
 * Camera and projection matrix builders (4×4, column-major).
 */
export class WasmCg {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Left-handed look-at view matrix.
     */
    static lookAtLh(eye_x: number, eye_y: number, eye_z: number, target_x: number, target_y: number, target_z: number, up_x: number, up_y: number, up_z: number): WasmMatrix32;
    /**
     * Right-handed look-at view matrix: eye (x,y,z), target (x,y,z), up (x,y,z).
     */
    static lookAtRh(eye_x: number, eye_y: number, eye_z: number, target_x: number, target_y: number, target_z: number, up_x: number, up_y: number, up_z: number): WasmMatrix32;
    /**
     * Orthographic projection: left, right, bottom, top, near, far.
     */
    static newOrthographic(left: number, right: number, bottom: number, top: number, near: number, far: number): WasmMatrix32;
    /**
     * Perspective projection: aspect = width/height, fov_y_rad vertical FOV (radians), near/far positive.
     */
    static newPerspective(aspect: number, fov_y_rad: number, near: number, far: number): WasmMatrix32;
    /**
     * Translation matrix with (x, y, z).
     */
    static newTranslation(x: number, y: number, z: number): WasmMatrix32;
}

/**
 * Cholesky decomposition: A = L L^T for symmetric positive definite A.
 */
export class WasmCholesky {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Lower triangular factor L (A = L L^T).
     */
    getL(): WasmMatrix;
    /**
     * Compute Cholesky decomposition of matrix A. A must be square and symmetric positive definite.
     */
    constructor(a: WasmMatrix);
    /**
     * Solve Ax = b where A = L L^T. Returns x.
     */
    solve(b: WasmVector): WasmVector;
}

/**
 * Result of D* Lite: path from start to goal and total distance.
 */
export class WasmDStarLiteResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Total distance from start to goal; `Infinity` if no path.
     */
    getDist(): number;
    /**
     * Path from start to goal (empty if no path). Includes both start and goal.
     */
    getPath(): Uint32Array;
}

/**
 * Result of DBSCAN clustering: labels (cluster index or noise).
 */
export class WasmDbscan {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Cluster label for each sample (0, 1, ...) or `NOISE_LABEL` for noise.
     */
    getLabels(): Uint32Array;
    /**
     * Whether sample `i` is classified as noise.
     */
    isNoise(i: number): boolean;
    /**
     * Number of clusters (excluding noise).
     */
    nClusters(): number;
    /**
     * Run DBSCAN on data matrix (rows = samples, cols = features).
     * Points within `eps` (Euclidean) are neighbors; core points have at least `min_pts` neighbors.
     */
    constructor(data: WasmMatrix, eps: number, min_pts: number);
}

/**
 * Result of Dijkstra: distances and predecessors.
 */
export class WasmDijkstraResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Distance from source to `target`; `Infinity` if unreachable.
     */
    distanceTo(target: number): number;
    /**
     * Distance from source to each node (`Infinity` if unreachable).
     */
    getDistances(): Float64Array;
    /**
     * Predecessor on shortest path; `null` for source or unreachable.
     * Returns -1 for null (JS doesn't have Option<usize>).
     */
    getPredecessors(): Int32Array;
    /**
     * Reconstruct path from source to `target`. Returns array of node indices, or empty if unreachable.
     */
    pathTo(target: number): Uint32Array;
}

/**
 * Distance metric functions.
 */
export class WasmDistance {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Chebyshev (L-infinity) distance between two vectors.
     */
    static chebyshev(a: WasmVector, b: WasmVector): number;
    /**
     * Cosine distance between two vectors (1 - cosine_similarity).
     */
    static cosineDistance(a: WasmVector, b: WasmVector): number;
    /**
     * Cosine similarity between two vectors (1 = identical direction, 0 = orthogonal, -1 = opposite).
     */
    static cosineSimilarity(a: WasmVector, b: WasmVector): number;
    /**
     * Manhattan (L1) distance between two vectors.
     */
    static manhattan(a: WasmVector, b: WasmVector): number;
    /**
     * Minkowski distance with exponent p.
     */
    static minkowski(a: WasmVector, b: WasmVector, p: number): number;
}

/**
 * Directed weighted graph for pathfinding (Dijkstra, A*).
 */
export class WasmGraph {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a directed edge from `u` to `v` with weight `w`.
     */
    addEdge(u: number, v: number, w: number): void;
    /**
     * Add an undirected edge between `u` and `v` with weight `w`.
     */
    addEdgeUndirected(u: number, v: number, w: number): void;
    /**
     * Run A* from `start` to `goal` with zero heuristic (equivalent to Dijkstra).
     */
    astar(start: number, goal: number): WasmAstarResult;
    /**
     * Run A* from `start` to `goal` with Euclidean heuristic from node coordinates.
     * `coords` must have rows = `num_nodes` and cols = 2 or 3 (x,y or x,y,z).
     */
    astarWithCoords(start: number, goal: number, coords: WasmMatrix): WasmAstarResult;
    /**
     * BFS from source. Treats graph as undirected.
     */
    bfs(source: number): WasmBfsResult;
    /**
     * DFS postorder from source. Treats graph as undirected.
     */
    dfsPostorder(source: number): Uint32Array;
    /**
     * DFS preorder from source. Treats graph as undirected.
     */
    dfsPreorder(source: number): Uint32Array;
    /**
     * Run Dijkstra from `source`. Returns distances and predecessors.
     */
    dijkstra(source: number): WasmDijkstraResult;
    /**
     * DSatur vertex coloring. Returns array of color indices (one per node).
     */
    dsaturColoring(): Uint32Array;
    /**
     * Run D* Lite (one-shot replan) from `start` to `goal`. Mutates the graph internally.
     */
    dstarLite(start: number, goal: number): WasmDStarLiteResult;
    /**
     * Build a graph from a flat edge list: `[u0, v0, w0, u1, v1, w1, ...]` (u→v with weight w).
     */
    static fromEdges(n: number, edges: Float64Array): WasmGraph;
    /**
     * Greedy vertex coloring. Returns array of color indices (one per node).
     */
    greedyVertexColoring(): Uint32Array;
    /**
     * Returns 2-coloring if bipartite, or `null` if graph has odd cycle.
     */
    isBipartite(): Uint32Array | undefined;
    /**
     * Create a graph with `n` nodes and no edges.
     */
    constructor(n: number);
    /**
     * Total number of directed edges.
     */
    numEdges(): number;
    /**
     * Number of nodes.
     */
    numNodes(): number;
}

/**
 * Result of K-means clustering: labels and centroids.
 */
export class WasmKmeans {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Centroid matrix (k rows × features columns).
     */
    getCentroids(): WasmMatrix;
    /**
     * Cluster label for each sample (0 to k-1).
     */
    getLabels(): Uint32Array;
    /**
     * Number of clusters.
     */
    nClusters(): number;
    /**
     * Run K-means on data matrix (rows = samples, cols = features).
     * `k` is the number of clusters. `max_iters` is maximum iterations (0 = 300).
     */
    constructor(data: WasmMatrix, k: number, max_iters: number);
}

/**
 * LU decomposition: P A = L U for general square A.
 */
export class WasmLu {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Determinant of the original matrix.
     */
    determinant(): number;
    /**
     * Combined LU factor (L unit lower, U upper).
     */
    getLU(): WasmMatrix;
    /**
     * Compute LU decomposition of matrix A. A must be square and non-singular.
     */
    constructor(a: WasmMatrix);
    /**
     * Solve Ax = b. Returns x.
     */
    solve(b: WasmVector): WasmVector;
}

/**
 * A dense matrix accessible from JavaScript.
 */
export class WasmMatrix {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Matrix addition.
     */
    add(other: WasmMatrix): WasmMatrix;
    /**
     * Create a matrix from a flat array (column-major order).
     */
    static fromArray(rows: number, cols: number, data: Float64Array): WasmMatrix;
    /**
     * Get element at (i, j).
     */
    get(i: number, j: number): number;
    /**
     * Create an identity matrix.
     */
    static identity(n: number): WasmMatrix;
    /**
     * Matrix multiplication.
     */
    mul(other: WasmMatrix): WasmMatrix;
    /**
     * Matrix-vector multiplication.
     */
    mulVector(v: WasmVector): WasmVector;
    /**
     * Create a new zero matrix with given dimensions (column-major).
     */
    constructor(rows: number, cols: number);
    /**
     * Scalar multiplication.
     */
    scale(scalar: number): WasmMatrix;
    /**
     * Set element at (i, j).
     */
    set(i: number, j: number, value: number): void;
    /**
     * Solve Ax = b for square matrix A. Returns x or an error if A is singular or not square.
     */
    solve(b: WasmVector): WasmVector;
    /**
     * Matrix subtraction.
     */
    sub(other: WasmMatrix): WasmMatrix;
    /**
     * Economical SVD: returns U, V, and singular values sigma (min(m,n) components).
     */
    svdEcon(): WasmSvd;
    /**
     * Return data as a flat Float64Array (column-major).
     */
    toArray(): Float64Array;
    /**
     * Transpose the matrix (returns new matrix).
     */
    transpose(): WasmMatrix;
    /**
     * Get the number of columns.
     */
    readonly cols: number;
    /**
     * Get the number of rows.
     */
    readonly rows: number;
}

/**
 * A 32-bit float matrix for 3D graphics operations.
 */
export class WasmMatrix32 {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Create a matrix from a flat array (column-major order).
     */
    static fromArray(rows: number, cols: number, data: Float32Array): WasmMatrix32;
    /**
     * Get element at (i, j).
     */
    get(i: number, j: number): number;
    /**
     * Create a 4x4 identity matrix.
     */
    static identity4(): WasmMatrix32;
    /**
     * Inverse for 4×4 matrices (e.g. view/model). Errors if not 4×4.
     */
    inverse(): WasmMatrix32;
    /**
     * Matrix multiplication.
     */
    mul(other: WasmMatrix32): WasmMatrix32;
    /**
     * Create a new zero matrix with given dimensions.
     */
    constructor(rows: number, cols: number);
    /**
     * Create a 4x4 rotation matrix from Euler angles (radians).
     */
    static rotation(rx: number, ry: number, rz: number): WasmMatrix32;
    /**
     * Set element at (i, j).
     */
    set(i: number, j: number, value: number): void;
    /**
     * Return data as a flat array.
     */
    toArray(): Float32Array;
    /**
     * Transform a 3D point (x, y, z) by this 4×4 matrix. Returns [x', y', z'].
     */
    transformPoint(x: number, y: number, z: number): Float32Array;
    /**
     * Transform a 3D direction (x, y, z) by the 3×3 part only (no translation).
     */
    transformVector(x: number, y: number, z: number): Float32Array;
    /**
     * Transpose (returns new matrix).
     */
    transpose(): WasmMatrix32;
    /**
     * Get the number of columns.
     */
    readonly cols: number;
    /**
     * Get the number of rows.
     */
    readonly rows: number;
}

/**
 * Result of PCA: mean, components, and explained variance.
 */
export class WasmPca {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Principal components matrix (features × components); each column is a PC.
     */
    getComponents(): WasmMatrix;
    /**
     * Explained variance for each component.
     */
    getExplainedVariance(): WasmVector;
    /**
     * Mean vector (one per feature).
     */
    getMean(): WasmVector;
    /**
     * Number of components.
     */
    nComponents(): number;
    /**
     * Run PCA on data matrix (rows = samples, cols = features).
     * Returns mean, principal components, and explained variance.
     * If `n_components` is 0, all components are kept.
     */
    constructor(data: WasmMatrix, n_components: number);
    /**
     * Project data onto principal components. Returns projected matrix (samples × components).
     */
    transform(data: WasmMatrix): WasmMatrix;
}

/**
 * Result of PSO: best position, cost, and iterations.
 */
export class WasmPsoResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Cost at best position.
     */
    getBestCost(): number;
    /**
     * Best position found.
     */
    getBestPosition(): Float64Array;
    /**
     * Number of iterations performed.
     */
    getIterations(): number;
}

/**
 * Result of simplex LP solve: solution vector, objective value, and status string.
 */
export class WasmSimplexResult {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Optimal objective value (c'x).
     */
    getObjective(): number;
    /**
     * Status string: "optimal", "unbounded", or "infeasible".
     */
    getStatus(): string;
    /**
     * Solution vector x (length n).
     */
    getX(): WasmVector;
    /**
     * Solve LP in standard form: minimize c'x subject to Ax = b, x >= 0.
     * Takes objective coefficients `c`, constraint matrix `A`, and RHS `b`.
     * Returns solution vector, objective value, and status ("optimal", "unbounded", "infeasible", or error message).
     */
    constructor(c: WasmVector, a: WasmMatrix, b: WasmVector);
}

/**
 * Result of economical SVD: U (m×k), V (n×k), sigma (length k) with k = min(m, n).
 */
export class WasmSvd {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Singular values (vector).
     */
    getSigma(): WasmVector;
    /**
     * Left singular vectors U (matrix).
     */
    getU(): WasmMatrix;
    /**
     * Right singular vectors V (matrix).
     */
    getV(): WasmMatrix;
}

/**
 * Linear SVM (binary classification). Train with X (rows = samples, cols = features) and labels ±1.
 */
export class WasmSvm {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Train linear SVM. Labels must be ±1 (or positive → 1, else -1). Uses default options.
     */
    static train(x: WasmMatrix, labels: Float64Array): WasmSvmResult;
}

/**
 * RBF-kernel SVM (binary classification). Train with X, labels ±1, and gamma.
 */
export class WasmSvmRbf {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Train RBF SVM. Labels ±1. Gamma controls kernel width (e.g. 0.5).
     */
    static train(x: WasmMatrix, labels: Float64Array, gamma: number): WasmSvmRbfResult;
}

/**
 * Trained RBF-kernel SVM: support vectors, dual coefficients, bias, γ. Prediction via sign(Σ αᵢyᵢ K(svᵢ,x) + b).
 */
export class WasmSvmRbfResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Bias term.
     */
    getBias(): number;
    /**
     * RBF kernel parameter γ.
     */
    getGamma(): number;
    /**
     * Support vectors matrix (n_sv × n_features).
     */
    getSupportVectors(): WasmMatrix;
    /**
     * Predict label for one sample: +1 or -1.
     */
    predict(sample: Float64Array): number;
    /**
     * Predict labels for all rows of X. Returns array of +1 or -1.
     */
    predictAll(x: WasmMatrix): Float64Array;
}

/**
 * Trained linear SVM: weight vector and bias for prediction sign(w·x + b).
 */
export class WasmSvmResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Bias term.
     */
    getBias(): number;
    /**
     * Weight vector (one per feature).
     */
    getWeights(): WasmVector;
    /**
     * Predict label for one sample: +1 or -1. `sample` is a row of features (length = n_features).
     */
    predict(sample: Float64Array): number;
    /**
     * Predict labels for all rows of X. Returns array of +1 or -1.
     */
    predictAll(x: WasmMatrix): Float64Array;
}

/**
 * A vector accessible from JavaScript.
 */
export class WasmVector {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Vector addition.
     */
    add(other: WasmVector): WasmVector;
    /**
     * Dot product with another vector.
     */
    dot(other: WasmVector): number;
    /**
     * Euclidean distance to another vector.
     */
    euclideanDistance(other: WasmVector): number;
    /**
     * Create a vector from a Float64Array.
     */
    static fromArray(data: Float64Array): WasmVector;
    /**
     * Get element at index.
     */
    get(i: number): number;
    /**
     * Check if empty.
     */
    isEmpty(): boolean;
    /**
     * Linear interpolation with another vector: (1 - t) * self + t * other.
     */
    lerp(other: WasmVector, t: number): WasmVector;
    /**
     * Create a new zero vector with given length.
     */
    constructor(len: number);
    /**
     * Euclidean norm.
     */
    norm(): number;
    /**
     * Scalar multiplication.
     */
    scale(scalar: number): WasmVector;
    /**
     * Set element at index.
     */
    set(i: number, value: number): void;
    /**
     * Vector subtraction.
     */
    sub(other: WasmVector): WasmVector;
    /**
     * Return data as Float64Array.
     */
    toArray(): Float64Array;
    /**
     * Get the length.
     */
    readonly len: number;
}

/**
 * Apply window to signal. Returns windowed signal (signal[i] * window[i]).
 */
export function applyWindow(signal: Float64Array, window: Float64Array): Float64Array;

/**
 * Blackman window.
 */
export function blackman(len: number): Float64Array;

/**
 * 1D convolution (full).
 */
export function conv1d(signal: Float64Array, kernel: Float64Array): Float64Array;

/**
 * 1D convolution same-length.
 */
export function conv1dSame(signal: Float64Array, kernel: Float64Array): Float64Array;

/**
 * DCT-II forward.
 */
export function dct2Forward(signal: Float64Array): Float64Array;

/**
 * DCT-III inverse.
 */
export function dct2Inverse(coeffs: Float64Array): Float64Array;

/**
 * Haar DWT forward (even length).
 */
export function dwtHaarForward(signal: Float64Array): Float64Array;

/**
 * Haar DWT inverse.
 */
export function dwtHaarInverse(coeffs: Float64Array): Float64Array;

/**
 * FBM with Perlin base noise. Typical values: lacunarity 2.0, persistence 0.5.
 */
export function fbm2dPerlin(x: number, y: number, octaves: number, lacunarity: number, persistence: number): number;

/**
 * Forward real FFT. Input length must be power of 2.
 * Returns spectrum as [re0, im0, re1, im1, ...] (interleaved).
 */
export function fftForwardReal(signal: Float64Array): Float64Array;

/**
 * Inverse FFT. Input as interleaved [re0, im0, re1, im1, ...]. Returns real part.
 */
export function fftInverse(spectrum: Float64Array): Float64Array;

/**
 * Hamming window.
 */
export function hamming(len: number): Float64Array;

/**
 * Hann window.
 */
export function hann(len: number): Float64Array;

/**
 * Backtracking line search: find step length α so that Armijo holds.
 *
 * `x` and `d` are the current point and search direction. `f` is the cost at `x`,
 * `g_dot_d` is the gradient at `x` dotted with `d`. `cost_fn` is a JS function
 * `(point: Float64Array) => number` that evaluates the cost at a point.
 */
export function lineSearchBacktracking(x: Float64Array, d: Float64Array, f: number, g_dot_d: number, cost_fn: Function): number;

/**
 * 2D Perlin noise at (x, y). Output is approximately in [-1, 1].
 */
export function perlin2d(x: number, y: number): number;

/**
 * Run PSO to minimize a cost function over a box.
 *
 * `cost_fn` is a JS function `(position: Float64Array) => number`.
 * `lower` and `upper` define the search bounds per dimension.
 */
export function psoMinimize(lower: Float64Array, upper: Float64Array, num_particles: number, max_iters: number, cost_fn: Function): WasmPsoResult;

/**
 * Tukey window. Alpha in [0, 1]: 0=rectangular, 1=Hann.
 */
export function tukey(len: number, alpha: number): Float64Array;

/**
 * Wave height at (u, v) in [0, 1]². Returns value in [0, 1].
 */
export function wave2d(u: number, v: number): number;

/**
 * Wave height with configurable wave numbers (radians per unit).
 */
export function wave2dParams(u: number, v: number, k1: number, k2: number): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly NOISE_LABEL: () => number;
    readonly __wbg_wasmastarresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmbfsresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmcg_free: (a: number, b: number) => void;
    readonly __wbg_wasmcholesky_free: (a: number, b: number) => void;
    readonly __wbg_wasmdbscan_free: (a: number, b: number) => void;
    readonly __wbg_wasmdijkstraresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmdstarliteresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmgraph_free: (a: number, b: number) => void;
    readonly __wbg_wasmkmeans_free: (a: number, b: number) => void;
    readonly __wbg_wasmlu_free: (a: number, b: number) => void;
    readonly __wbg_wasmmatrix32_free: (a: number, b: number) => void;
    readonly __wbg_wasmpca_free: (a: number, b: number) => void;
    readonly __wbg_wasmpsoresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmsimplexresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmsvd_free: (a: number, b: number) => void;
    readonly __wbg_wasmsvmrbfresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmsvmresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmvector_free: (a: number, b: number) => void;
    readonly applyWindow: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly blackman: (a: number) => [number, number];
    readonly conv1d: (a: number, b: number, c: number, d: number) => [number, number];
    readonly conv1dSame: (a: number, b: number, c: number, d: number) => [number, number];
    readonly dct2Forward: (a: number, b: number) => [number, number, number, number];
    readonly dct2Inverse: (a: number, b: number) => [number, number, number, number];
    readonly dwtHaarForward: (a: number, b: number) => [number, number];
    readonly dwtHaarInverse: (a: number, b: number) => [number, number];
    readonly fbm2dPerlin: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly fftForwardReal: (a: number, b: number) => [number, number, number, number];
    readonly fftInverse: (a: number, b: number) => [number, number, number, number];
    readonly hamming: (a: number) => [number, number];
    readonly hann: (a: number) => [number, number];
    readonly lineSearchBacktracking: (a: number, b: number, c: number, d: number, e: number, f: number, g: any) => [number, number, number];
    readonly psoMinimize: (a: number, b: number, c: number, d: number, e: number, f: number, g: any) => [number, number, number];
    readonly tukey: (a: number, b: number) => [number, number];
    readonly wasmastarresult_getDist: (a: number) => number;
    readonly wasmastarresult_getPath: (a: number) => [number, number];
    readonly wasmastarresult_getPredecessors: (a: number) => [number, number];
    readonly wasmastarresult_pathTo: (a: number, b: number) => [number, number];
    readonly wasmbfsresult_getDepth: (a: number) => [number, number];
    readonly wasmbfsresult_getOrder: (a: number) => [number, number];
    readonly wasmcg_lookAtLh: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
    readonly wasmcg_lookAtRh: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
    readonly wasmcg_newOrthographic: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly wasmcg_newPerspective: (a: number, b: number, c: number, d: number) => number;
    readonly wasmcg_newTranslation: (a: number, b: number, c: number) => number;
    readonly wasmcholesky_getL: (a: number) => number;
    readonly wasmcholesky_new: (a: number) => [number, number, number];
    readonly wasmcholesky_solve: (a: number, b: number) => [number, number, number];
    readonly wasmdbscan_getLabels: (a: number) => [number, number];
    readonly wasmdbscan_isNoise: (a: number, b: number) => number;
    readonly wasmdbscan_nClusters: (a: number) => number;
    readonly wasmdbscan_new: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmdijkstraresult_distanceTo: (a: number, b: number) => number;
    readonly wasmdijkstraresult_getDistances: (a: number) => [number, number];
    readonly wasmdijkstraresult_getPredecessors: (a: number) => [number, number];
    readonly wasmdijkstraresult_pathTo: (a: number, b: number) => [number, number];
    readonly wasmdistance_chebyshev: (a: number, b: number) => [number, number, number];
    readonly wasmdistance_cosineDistance: (a: number, b: number) => [number, number, number];
    readonly wasmdistance_cosineSimilarity: (a: number, b: number) => [number, number, number];
    readonly wasmdistance_manhattan: (a: number, b: number) => [number, number, number];
    readonly wasmdistance_minkowski: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmdstarliteresult_getPath: (a: number) => [number, number];
    readonly wasmgraph_addEdge: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmgraph_addEdgeUndirected: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmgraph_astar: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmgraph_astarWithCoords: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmgraph_bfs: (a: number, b: number) => [number, number, number];
    readonly wasmgraph_dfsPostorder: (a: number, b: number) => [number, number, number, number];
    readonly wasmgraph_dfsPreorder: (a: number, b: number) => [number, number, number, number];
    readonly wasmgraph_dijkstra: (a: number, b: number) => [number, number, number];
    readonly wasmgraph_dsaturColoring: (a: number) => [number, number];
    readonly wasmgraph_dstarLite: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmgraph_fromEdges: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmgraph_greedyVertexColoring: (a: number) => [number, number];
    readonly wasmgraph_isBipartite: (a: number) => [number, number];
    readonly wasmgraph_new: (a: number) => number;
    readonly wasmgraph_numEdges: (a: number) => number;
    readonly wasmgraph_numNodes: (a: number) => number;
    readonly wasmkmeans_getCentroids: (a: number) => number;
    readonly wasmkmeans_getLabels: (a: number) => [number, number];
    readonly wasmkmeans_new: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmlu_determinant: (a: number) => number;
    readonly wasmlu_getLU: (a: number) => number;
    readonly wasmlu_new: (a: number) => [number, number, number];
    readonly wasmlu_solve: (a: number, b: number) => [number, number, number];
    readonly wasmmatrix32_cols: (a: number) => number;
    readonly wasmmatrix32_fromArray: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmmatrix32_get: (a: number, b: number, c: number) => number;
    readonly wasmmatrix32_identity4: () => number;
    readonly wasmmatrix32_inverse: (a: number) => [number, number, number];
    readonly wasmmatrix32_mul: (a: number, b: number) => [number, number, number];
    readonly wasmmatrix32_new: (a: number, b: number) => number;
    readonly wasmmatrix32_rotation: (a: number, b: number, c: number) => number;
    readonly wasmmatrix32_rows: (a: number) => number;
    readonly wasmmatrix32_set: (a: number, b: number, c: number, d: number) => void;
    readonly wasmmatrix32_toArray: (a: number) => [number, number];
    readonly wasmmatrix32_transformPoint: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly wasmmatrix32_transformVector: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly wasmmatrix32_transpose: (a: number) => number;
    readonly wasmmatrix_add: (a: number, b: number) => [number, number, number];
    readonly wasmmatrix_fromArray: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmmatrix_get: (a: number, b: number, c: number) => number;
    readonly wasmmatrix_identity: (a: number) => number;
    readonly wasmmatrix_mul: (a: number, b: number) => [number, number, number];
    readonly wasmmatrix_mulVector: (a: number, b: number) => [number, number, number];
    readonly wasmmatrix_new: (a: number, b: number) => number;
    readonly wasmmatrix_scale: (a: number, b: number) => number;
    readonly wasmmatrix_set: (a: number, b: number, c: number, d: number) => void;
    readonly wasmmatrix_solve: (a: number, b: number) => [number, number, number];
    readonly wasmmatrix_sub: (a: number, b: number) => [number, number, number];
    readonly wasmmatrix_svdEcon: (a: number) => number;
    readonly wasmmatrix_toArray: (a: number) => [number, number];
    readonly wasmmatrix_transpose: (a: number) => number;
    readonly wasmpca_getComponents: (a: number) => number;
    readonly wasmpca_getExplainedVariance: (a: number) => number;
    readonly wasmpca_getMean: (a: number) => number;
    readonly wasmpca_nComponents: (a: number) => number;
    readonly wasmpca_new: (a: number, b: number) => number;
    readonly wasmpca_transform: (a: number, b: number) => [number, number, number];
    readonly wasmpsoresult_getBestPosition: (a: number) => [number, number];
    readonly wasmsimplexresult_getStatus: (a: number) => [number, number];
    readonly wasmsimplexresult_getX: (a: number) => number;
    readonly wasmsimplexresult_new: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmsvd_getSigma: (a: number) => number;
    readonly wasmsvd_getU: (a: number) => number;
    readonly wasmsvd_getV: (a: number) => number;
    readonly wasmsvm_train: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmsvmrbf_train: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmsvmrbfresult_getBias: (a: number) => number;
    readonly wasmsvmrbfresult_getGamma: (a: number) => number;
    readonly wasmsvmrbfresult_getSupportVectors: (a: number) => number;
    readonly wasmsvmrbfresult_predict: (a: number, b: number, c: number) => number;
    readonly wasmsvmrbfresult_predictAll: (a: number, b: number) => [number, number];
    readonly wasmsvmresult_getWeights: (a: number) => number;
    readonly wasmsvmresult_predict: (a: number, b: number, c: number) => number;
    readonly wasmsvmresult_predictAll: (a: number, b: number) => [number, number];
    readonly wasmvector_add: (a: number, b: number) => [number, number, number];
    readonly wasmvector_dot: (a: number, b: number) => [number, number, number];
    readonly wasmvector_euclideanDistance: (a: number, b: number) => [number, number, number];
    readonly wasmvector_fromArray: (a: number, b: number) => number;
    readonly wasmvector_get: (a: number, b: number) => number;
    readonly wasmvector_isEmpty: (a: number) => number;
    readonly wasmvector_lerp: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmvector_new: (a: number) => number;
    readonly wasmvector_norm: (a: number) => number;
    readonly wasmvector_scale: (a: number, b: number) => number;
    readonly wasmvector_set: (a: number, b: number, c: number) => void;
    readonly wasmvector_sub: (a: number, b: number) => [number, number, number];
    readonly wasmvector_toArray: (a: number) => [number, number];
    readonly wave2d: (a: number, b: number) => number;
    readonly wave2dParams: (a: number, b: number, c: number, d: number) => number;
    readonly perlin2d: (a: number, b: number) => number;
    readonly wasmdstarliteresult_getDist: (a: number) => number;
    readonly wasmkmeans_nClusters: (a: number) => number;
    readonly wasmmatrix_cols: (a: number) => number;
    readonly wasmmatrix_rows: (a: number) => number;
    readonly wasmpsoresult_getBestCost: (a: number) => number;
    readonly wasmpsoresult_getIterations: (a: number) => number;
    readonly wasmsimplexresult_getObjective: (a: number) => number;
    readonly wasmsvmresult_getBias: (a: number) => number;
    readonly wasmvector_len: (a: number) => number;
    readonly __wbg_wasmdistance_free: (a: number, b: number) => void;
    readonly __wbg_wasmsvm_free: (a: number, b: number) => void;
    readonly __wbg_wasmsvmrbf_free: (a: number, b: number) => void;
    readonly __wbg_wasmmatrix_free: (a: number, b: number) => void;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
