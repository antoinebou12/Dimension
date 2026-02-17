/* @ts-self-types="./mathlib.d.ts" */

/**
 * Label for DBSCAN noise points (not assigned to any cluster).
 * In JS this is the maximum 32-bit unsigned value when cast from u32.
 * @returns {number}
 */
export function NOISE_LABEL() {
    const ret = wasm.NOISE_LABEL();
    return ret >>> 0;
}

/**
 * Result of A*: path from start to goal, total distance, and predecessors.
 */
export class WasmAstarResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmAstarResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmAstarResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmAstarResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmastarresult_free(ptr, 0);
    }
    /**
     * Total distance from start to goal; `Infinity` if no path.
     * @returns {number}
     */
    getDist() {
        const ret = wasm.wasmastarresult_getDist(this.__wbg_ptr);
        return ret;
    }
    /**
     * Path from start to goal (empty if no path). Includes both start and goal.
     * @returns {Uint32Array}
     */
    getPath() {
        const ret = wasm.wasmastarresult_getPath(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Predecessor on shortest path; -1 for start or unreachable.
     * @returns {Int32Array}
     */
    getPredecessors() {
        const ret = wasm.wasmastarresult_getPredecessors(this.__wbg_ptr);
        var v1 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Reconstruct path from start to `target`. Returns array of node indices, or empty if unreachable.
     * @param {number} target
     * @returns {Uint32Array}
     */
    pathTo(target) {
        const ret = wasm.wasmastarresult_pathTo(this.__wbg_ptr, target);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) WasmAstarResult.prototype[Symbol.dispose] = WasmAstarResult.prototype.free;

/**
 * Result of BFS: visit order and depth per node.
 */
export class WasmBfsResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmBfsResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmBfsResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmBfsResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmbfsresult_free(ptr, 0);
    }
    /**
     * Depth from source (usize::MAX for unreachable).
     * @returns {Uint32Array}
     */
    getDepth() {
        const ret = wasm.wasmbfsresult_getDepth(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Visit order (nodes in discovery order).
     * @returns {Uint32Array}
     */
    getOrder() {
        const ret = wasm.wasmbfsresult_getOrder(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) WasmBfsResult.prototype[Symbol.dispose] = WasmBfsResult.prototype.free;

/**
 * Camera and projection matrix builders (4×4, column-major).
 */
export class WasmCg {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmCgFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcg_free(ptr, 0);
    }
    /**
     * Left-handed look-at view matrix.
     * @param {number} eye_x
     * @param {number} eye_y
     * @param {number} eye_z
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} target_z
     * @param {number} up_x
     * @param {number} up_y
     * @param {number} up_z
     * @returns {WasmMatrix32}
     */
    static lookAtLh(eye_x, eye_y, eye_z, target_x, target_y, target_z, up_x, up_y, up_z) {
        const ret = wasm.wasmcg_lookAtLh(eye_x, eye_y, eye_z, target_x, target_y, target_z, up_x, up_y, up_z);
        return WasmMatrix32.__wrap(ret);
    }
    /**
     * Right-handed look-at view matrix: eye (x,y,z), target (x,y,z), up (x,y,z).
     * @param {number} eye_x
     * @param {number} eye_y
     * @param {number} eye_z
     * @param {number} target_x
     * @param {number} target_y
     * @param {number} target_z
     * @param {number} up_x
     * @param {number} up_y
     * @param {number} up_z
     * @returns {WasmMatrix32}
     */
    static lookAtRh(eye_x, eye_y, eye_z, target_x, target_y, target_z, up_x, up_y, up_z) {
        const ret = wasm.wasmcg_lookAtRh(eye_x, eye_y, eye_z, target_x, target_y, target_z, up_x, up_y, up_z);
        return WasmMatrix32.__wrap(ret);
    }
    /**
     * Orthographic projection: left, right, bottom, top, near, far.
     * @param {number} left
     * @param {number} right
     * @param {number} bottom
     * @param {number} top
     * @param {number} near
     * @param {number} far
     * @returns {WasmMatrix32}
     */
    static newOrthographic(left, right, bottom, top, near, far) {
        const ret = wasm.wasmcg_newOrthographic(left, right, bottom, top, near, far);
        return WasmMatrix32.__wrap(ret);
    }
    /**
     * Perspective projection: aspect = width/height, fov_y_rad vertical FOV (radians), near/far positive.
     * @param {number} aspect
     * @param {number} fov_y_rad
     * @param {number} near
     * @param {number} far
     * @returns {WasmMatrix32}
     */
    static newPerspective(aspect, fov_y_rad, near, far) {
        const ret = wasm.wasmcg_newPerspective(aspect, fov_y_rad, near, far);
        return WasmMatrix32.__wrap(ret);
    }
    /**
     * Translation matrix with (x, y, z).
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @returns {WasmMatrix32}
     */
    static newTranslation(x, y, z) {
        const ret = wasm.wasmcg_newTranslation(x, y, z);
        return WasmMatrix32.__wrap(ret);
    }
}
if (Symbol.dispose) WasmCg.prototype[Symbol.dispose] = WasmCg.prototype.free;

/**
 * Cholesky decomposition: A = L L^T for symmetric positive definite A.
 */
export class WasmCholesky {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmCholeskyFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcholesky_free(ptr, 0);
    }
    /**
     * Lower triangular factor L (A = L L^T).
     * @returns {WasmMatrix}
     */
    getL() {
        const ret = wasm.wasmcholesky_getL(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Compute Cholesky decomposition of matrix A. A must be square and symmetric positive definite.
     * @param {WasmMatrix} a
     */
    constructor(a) {
        _assertClass(a, WasmMatrix);
        const ret = wasm.wasmcholesky_new(a.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        WasmCholeskyFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Solve Ax = b where A = L L^T. Returns x.
     * @param {WasmVector} b
     * @returns {WasmVector}
     */
    solve(b) {
        _assertClass(b, WasmVector);
        const ret = wasm.wasmcholesky_solve(this.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmVector.__wrap(ret[0]);
    }
}
if (Symbol.dispose) WasmCholesky.prototype[Symbol.dispose] = WasmCholesky.prototype.free;

/**
 * Result of D* Lite: path from start to goal and total distance.
 */
export class WasmDStarLiteResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmDStarLiteResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmDStarLiteResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmDStarLiteResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmdstarliteresult_free(ptr, 0);
    }
    /**
     * Total distance from start to goal; `Infinity` if no path.
     * @returns {number}
     */
    getDist() {
        const ret = wasm.wasmastarresult_getDist(this.__wbg_ptr);
        return ret;
    }
    /**
     * Path from start to goal (empty if no path). Includes both start and goal.
     * @returns {Uint32Array}
     */
    getPath() {
        const ret = wasm.wasmdstarliteresult_getPath(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) WasmDStarLiteResult.prototype[Symbol.dispose] = WasmDStarLiteResult.prototype.free;

/**
 * Result of DBSCAN clustering: labels (cluster index or noise).
 */
export class WasmDbscan {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmDbscanFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmdbscan_free(ptr, 0);
    }
    /**
     * Cluster label for each sample (0, 1, ...) or `NOISE_LABEL` for noise.
     * @returns {Uint32Array}
     */
    getLabels() {
        const ret = wasm.wasmdbscan_getLabels(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Whether sample `i` is classified as noise.
     * @param {number} i
     * @returns {boolean}
     */
    isNoise(i) {
        const ret = wasm.wasmdbscan_isNoise(this.__wbg_ptr, i);
        return ret !== 0;
    }
    /**
     * Number of clusters (excluding noise).
     * @returns {number}
     */
    nClusters() {
        const ret = wasm.wasmdbscan_nClusters(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Run DBSCAN on data matrix (rows = samples, cols = features).
     * Points within `eps` (Euclidean) are neighbors; core points have at least `min_pts` neighbors.
     * @param {WasmMatrix} data
     * @param {number} eps
     * @param {number} min_pts
     */
    constructor(data, eps, min_pts) {
        _assertClass(data, WasmMatrix);
        const ret = wasm.wasmdbscan_new(data.__wbg_ptr, eps, min_pts);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        WasmDbscanFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) WasmDbscan.prototype[Symbol.dispose] = WasmDbscan.prototype.free;

/**
 * Result of Dijkstra: distances and predecessors.
 */
export class WasmDijkstraResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmDijkstraResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmDijkstraResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmDijkstraResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmdijkstraresult_free(ptr, 0);
    }
    /**
     * Distance from source to `target`; `Infinity` if unreachable.
     * @param {number} target
     * @returns {number}
     */
    distanceTo(target) {
        const ret = wasm.wasmdijkstraresult_distanceTo(this.__wbg_ptr, target);
        return ret;
    }
    /**
     * Distance from source to each node (`Infinity` if unreachable).
     * @returns {Float64Array}
     */
    getDistances() {
        const ret = wasm.wasmdijkstraresult_getDistances(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * Predecessor on shortest path; `null` for source or unreachable.
     * Returns -1 for null (JS doesn't have Option<usize>).
     * @returns {Int32Array}
     */
    getPredecessors() {
        const ret = wasm.wasmdijkstraresult_getPredecessors(this.__wbg_ptr);
        var v1 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Reconstruct path from source to `target`. Returns array of node indices, or empty if unreachable.
     * @param {number} target
     * @returns {Uint32Array}
     */
    pathTo(target) {
        const ret = wasm.wasmdijkstraresult_pathTo(this.__wbg_ptr, target);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) WasmDijkstraResult.prototype[Symbol.dispose] = WasmDijkstraResult.prototype.free;

/**
 * Distance metric functions.
 */
export class WasmDistance {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmDistanceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmdistance_free(ptr, 0);
    }
    /**
     * Chebyshev (L-infinity) distance between two vectors.
     * @param {WasmVector} a
     * @param {WasmVector} b
     * @returns {number}
     */
    static chebyshev(a, b) {
        _assertClass(a, WasmVector);
        _assertClass(b, WasmVector);
        const ret = wasm.wasmdistance_chebyshev(a.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Cosine distance between two vectors (1 - cosine_similarity).
     * @param {WasmVector} a
     * @param {WasmVector} b
     * @returns {number}
     */
    static cosineDistance(a, b) {
        _assertClass(a, WasmVector);
        _assertClass(b, WasmVector);
        const ret = wasm.wasmdistance_cosineDistance(a.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Cosine similarity between two vectors (1 = identical direction, 0 = orthogonal, -1 = opposite).
     * @param {WasmVector} a
     * @param {WasmVector} b
     * @returns {number}
     */
    static cosineSimilarity(a, b) {
        _assertClass(a, WasmVector);
        _assertClass(b, WasmVector);
        const ret = wasm.wasmdistance_cosineSimilarity(a.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Manhattan (L1) distance between two vectors.
     * @param {WasmVector} a
     * @param {WasmVector} b
     * @returns {number}
     */
    static manhattan(a, b) {
        _assertClass(a, WasmVector);
        _assertClass(b, WasmVector);
        const ret = wasm.wasmdistance_manhattan(a.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Minkowski distance with exponent p.
     * @param {WasmVector} a
     * @param {WasmVector} b
     * @param {number} p
     * @returns {number}
     */
    static minkowski(a, b, p) {
        _assertClass(a, WasmVector);
        _assertClass(b, WasmVector);
        const ret = wasm.wasmdistance_minkowski(a.__wbg_ptr, b.__wbg_ptr, p);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
}
if (Symbol.dispose) WasmDistance.prototype[Symbol.dispose] = WasmDistance.prototype.free;

/**
 * Directed weighted graph for pathfinding (Dijkstra, A*).
 */
export class WasmGraph {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmGraph.prototype);
        obj.__wbg_ptr = ptr;
        WasmGraphFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmGraphFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmgraph_free(ptr, 0);
    }
    /**
     * Add a directed edge from `u` to `v` with weight `w`.
     * @param {number} u
     * @param {number} v
     * @param {number} w
     */
    addEdge(u, v, w) {
        const ret = wasm.wasmgraph_addEdge(this.__wbg_ptr, u, v, w);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Add an undirected edge between `u` and `v` with weight `w`.
     * @param {number} u
     * @param {number} v
     * @param {number} w
     */
    addEdgeUndirected(u, v, w) {
        const ret = wasm.wasmgraph_addEdgeUndirected(this.__wbg_ptr, u, v, w);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Run A* from `start` to `goal` with zero heuristic (equivalent to Dijkstra).
     * @param {number} start
     * @param {number} goal
     * @returns {WasmAstarResult}
     */
    astar(start, goal) {
        const ret = wasm.wasmgraph_astar(this.__wbg_ptr, start, goal);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmAstarResult.__wrap(ret[0]);
    }
    /**
     * Run A* from `start` to `goal` with Euclidean heuristic from node coordinates.
     * `coords` must have rows = `num_nodes` and cols = 2 or 3 (x,y or x,y,z).
     * @param {number} start
     * @param {number} goal
     * @param {WasmMatrix} coords
     * @returns {WasmAstarResult}
     */
    astarWithCoords(start, goal, coords) {
        _assertClass(coords, WasmMatrix);
        const ret = wasm.wasmgraph_astarWithCoords(this.__wbg_ptr, start, goal, coords.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmAstarResult.__wrap(ret[0]);
    }
    /**
     * BFS from source. Treats graph as undirected.
     * @param {number} source
     * @returns {WasmBfsResult}
     */
    bfs(source) {
        const ret = wasm.wasmgraph_bfs(this.__wbg_ptr, source);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmBfsResult.__wrap(ret[0]);
    }
    /**
     * DFS postorder from source. Treats graph as undirected.
     * @param {number} source
     * @returns {Uint32Array}
     */
    dfsPostorder(source) {
        const ret = wasm.wasmgraph_dfsPostorder(this.__wbg_ptr, source);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * DFS preorder from source. Treats graph as undirected.
     * @param {number} source
     * @returns {Uint32Array}
     */
    dfsPreorder(source) {
        const ret = wasm.wasmgraph_dfsPreorder(this.__wbg_ptr, source);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Run Dijkstra from `source`. Returns distances and predecessors.
     * @param {number} source
     * @returns {WasmDijkstraResult}
     */
    dijkstra(source) {
        const ret = wasm.wasmgraph_dijkstra(this.__wbg_ptr, source);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmDijkstraResult.__wrap(ret[0]);
    }
    /**
     * DSatur vertex coloring. Returns array of color indices (one per node).
     * @returns {Uint32Array}
     */
    dsaturColoring() {
        const ret = wasm.wasmgraph_dsaturColoring(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Run D* Lite (one-shot replan) from `start` to `goal`. Mutates the graph internally.
     * @param {number} start
     * @param {number} goal
     * @returns {WasmDStarLiteResult}
     */
    dstarLite(start, goal) {
        const ret = wasm.wasmgraph_dstarLite(this.__wbg_ptr, start, goal);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmDStarLiteResult.__wrap(ret[0]);
    }
    /**
     * Build a graph from a flat edge list: `[u0, v0, w0, u1, v1, w1, ...]` (u→v with weight w).
     * @param {number} n
     * @param {Float64Array} edges
     * @returns {WasmGraph}
     */
    static fromEdges(n, edges) {
        const ptr0 = passArrayF64ToWasm0(edges, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmgraph_fromEdges(n, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmGraph.__wrap(ret[0]);
    }
    /**
     * Greedy vertex coloring. Returns array of color indices (one per node).
     * @returns {Uint32Array}
     */
    greedyVertexColoring() {
        const ret = wasm.wasmgraph_greedyVertexColoring(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Returns 2-coloring if bipartite, or `null` if graph has odd cycle.
     * @returns {Uint32Array | undefined}
     */
    isBipartite() {
        const ret = wasm.wasmgraph_isBipartite(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        }
        return v1;
    }
    /**
     * Create a graph with `n` nodes and no edges.
     * @param {number} n
     */
    constructor(n) {
        const ret = wasm.wasmgraph_new(n);
        this.__wbg_ptr = ret >>> 0;
        WasmGraphFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Total number of directed edges.
     * @returns {number}
     */
    numEdges() {
        const ret = wasm.wasmgraph_numEdges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Number of nodes.
     * @returns {number}
     */
    numNodes() {
        const ret = wasm.wasmgraph_numNodes(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) WasmGraph.prototype[Symbol.dispose] = WasmGraph.prototype.free;

/**
 * Result of K-means clustering: labels and centroids.
 */
export class WasmKmeans {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmKmeansFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmkmeans_free(ptr, 0);
    }
    /**
     * Centroid matrix (k rows × features columns).
     * @returns {WasmMatrix}
     */
    getCentroids() {
        const ret = wasm.wasmkmeans_getCentroids(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Cluster label for each sample (0 to k-1).
     * @returns {Uint32Array}
     */
    getLabels() {
        const ret = wasm.wasmkmeans_getLabels(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Number of clusters.
     * @returns {number}
     */
    nClusters() {
        const ret = wasm.wasmgraph_numNodes(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Run K-means on data matrix (rows = samples, cols = features).
     * `k` is the number of clusters. `max_iters` is maximum iterations (0 = 300).
     * @param {WasmMatrix} data
     * @param {number} k
     * @param {number} max_iters
     */
    constructor(data, k, max_iters) {
        _assertClass(data, WasmMatrix);
        const ret = wasm.wasmkmeans_new(data.__wbg_ptr, k, max_iters);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        WasmKmeansFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) WasmKmeans.prototype[Symbol.dispose] = WasmKmeans.prototype.free;

/**
 * LU decomposition: P A = L U for general square A.
 */
export class WasmLu {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmLuFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmlu_free(ptr, 0);
    }
    /**
     * Determinant of the original matrix.
     * @returns {number}
     */
    determinant() {
        const ret = wasm.wasmlu_determinant(this.__wbg_ptr);
        return ret;
    }
    /**
     * Combined LU factor (L unit lower, U upper).
     * @returns {WasmMatrix}
     */
    getLU() {
        const ret = wasm.wasmlu_getLU(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Compute LU decomposition of matrix A. A must be square and non-singular.
     * @param {WasmMatrix} a
     */
    constructor(a) {
        _assertClass(a, WasmMatrix);
        const ret = wasm.wasmlu_new(a.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        WasmLuFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Solve Ax = b. Returns x.
     * @param {WasmVector} b
     * @returns {WasmVector}
     */
    solve(b) {
        _assertClass(b, WasmVector);
        const ret = wasm.wasmlu_solve(this.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmVector.__wrap(ret[0]);
    }
}
if (Symbol.dispose) WasmLu.prototype[Symbol.dispose] = WasmLu.prototype.free;

/**
 * A dense matrix accessible from JavaScript.
 */
export class WasmMatrix {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmMatrix.prototype);
        obj.__wbg_ptr = ptr;
        WasmMatrixFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmMatrixFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmmatrix_free(ptr, 0);
    }
    /**
     * Matrix addition.
     * @param {WasmMatrix} other
     * @returns {WasmMatrix}
     */
    add(other) {
        _assertClass(other, WasmMatrix);
        const ret = wasm.wasmmatrix_add(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix.__wrap(ret[0]);
    }
    /**
     * Get the number of columns.
     * @returns {number}
     */
    get cols() {
        const ret = wasm.wasmmatrix32_cols(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a matrix from a flat array (column-major order).
     * @param {number} rows
     * @param {number} cols
     * @param {Float64Array} data
     * @returns {WasmMatrix}
     */
    static fromArray(rows, cols, data) {
        const ptr0 = passArrayF64ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmmatrix_fromArray(rows, cols, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix.__wrap(ret[0]);
    }
    /**
     * Get element at (i, j).
     * @param {number} i
     * @param {number} j
     * @returns {number}
     */
    get(i, j) {
        const ret = wasm.wasmmatrix_get(this.__wbg_ptr, i, j);
        return ret;
    }
    /**
     * Create an identity matrix.
     * @param {number} n
     * @returns {WasmMatrix}
     */
    static identity(n) {
        const ret = wasm.wasmmatrix_identity(n);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Matrix multiplication.
     * @param {WasmMatrix} other
     * @returns {WasmMatrix}
     */
    mul(other) {
        _assertClass(other, WasmMatrix);
        const ret = wasm.wasmmatrix_mul(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix.__wrap(ret[0]);
    }
    /**
     * Matrix-vector multiplication.
     * @param {WasmVector} v
     * @returns {WasmVector}
     */
    mulVector(v) {
        _assertClass(v, WasmVector);
        const ret = wasm.wasmmatrix_mulVector(this.__wbg_ptr, v.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmVector.__wrap(ret[0]);
    }
    /**
     * Create a new zero matrix with given dimensions (column-major).
     * @param {number} rows
     * @param {number} cols
     */
    constructor(rows, cols) {
        const ret = wasm.wasmmatrix_new(rows, cols);
        this.__wbg_ptr = ret >>> 0;
        WasmMatrixFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get the number of rows.
     * @returns {number}
     */
    get rows() {
        const ret = wasm.wasmmatrix32_rows(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Scalar multiplication.
     * @param {number} scalar
     * @returns {WasmMatrix}
     */
    scale(scalar) {
        const ret = wasm.wasmmatrix_scale(this.__wbg_ptr, scalar);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Set element at (i, j).
     * @param {number} i
     * @param {number} j
     * @param {number} value
     */
    set(i, j, value) {
        wasm.wasmmatrix_set(this.__wbg_ptr, i, j, value);
    }
    /**
     * Solve Ax = b for square matrix A. Returns x or an error if A is singular or not square.
     * @param {WasmVector} b
     * @returns {WasmVector}
     */
    solve(b) {
        _assertClass(b, WasmVector);
        const ret = wasm.wasmmatrix_solve(this.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmVector.__wrap(ret[0]);
    }
    /**
     * Matrix subtraction.
     * @param {WasmMatrix} other
     * @returns {WasmMatrix}
     */
    sub(other) {
        _assertClass(other, WasmMatrix);
        const ret = wasm.wasmmatrix_sub(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix.__wrap(ret[0]);
    }
    /**
     * Economical SVD: returns U, V, and singular values sigma (min(m,n) components).
     * @returns {WasmSvd}
     */
    svdEcon() {
        const ret = wasm.wasmmatrix_svdEcon(this.__wbg_ptr);
        return WasmSvd.__wrap(ret);
    }
    /**
     * Return data as a flat Float64Array (column-major).
     * @returns {Float64Array}
     */
    toArray() {
        const ret = wasm.wasmmatrix_toArray(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * Transpose the matrix (returns new matrix).
     * @returns {WasmMatrix}
     */
    transpose() {
        const ret = wasm.wasmmatrix_transpose(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
}
if (Symbol.dispose) WasmMatrix.prototype[Symbol.dispose] = WasmMatrix.prototype.free;

/**
 * A 32-bit float matrix for 3D graphics operations.
 */
export class WasmMatrix32 {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmMatrix32.prototype);
        obj.__wbg_ptr = ptr;
        WasmMatrix32Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmMatrix32Finalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmmatrix32_free(ptr, 0);
    }
    /**
     * Get the number of columns.
     * @returns {number}
     */
    get cols() {
        const ret = wasm.wasmmatrix32_cols(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a matrix from a flat array (column-major order).
     * @param {number} rows
     * @param {number} cols
     * @param {Float32Array} data
     * @returns {WasmMatrix32}
     */
    static fromArray(rows, cols, data) {
        const ptr0 = passArrayF32ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmmatrix32_fromArray(rows, cols, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix32.__wrap(ret[0]);
    }
    /**
     * Get element at (i, j).
     * @param {number} i
     * @param {number} j
     * @returns {number}
     */
    get(i, j) {
        const ret = wasm.wasmmatrix32_get(this.__wbg_ptr, i, j);
        return ret;
    }
    /**
     * Create a 4x4 identity matrix.
     * @returns {WasmMatrix32}
     */
    static identity4() {
        const ret = wasm.wasmmatrix32_identity4();
        return WasmMatrix32.__wrap(ret);
    }
    /**
     * Inverse for 4×4 matrices (e.g. view/model). Errors if not 4×4.
     * @returns {WasmMatrix32}
     */
    inverse() {
        const ret = wasm.wasmmatrix32_inverse(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix32.__wrap(ret[0]);
    }
    /**
     * Matrix multiplication.
     * @param {WasmMatrix32} other
     * @returns {WasmMatrix32}
     */
    mul(other) {
        _assertClass(other, WasmMatrix32);
        const ret = wasm.wasmmatrix32_mul(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix32.__wrap(ret[0]);
    }
    /**
     * Create a new zero matrix with given dimensions.
     * @param {number} rows
     * @param {number} cols
     */
    constructor(rows, cols) {
        const ret = wasm.wasmmatrix32_new(rows, cols);
        this.__wbg_ptr = ret >>> 0;
        WasmMatrix32Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Create a 4x4 rotation matrix from Euler angles (radians).
     * @param {number} rx
     * @param {number} ry
     * @param {number} rz
     * @returns {WasmMatrix32}
     */
    static rotation(rx, ry, rz) {
        const ret = wasm.wasmmatrix32_rotation(rx, ry, rz);
        return WasmMatrix32.__wrap(ret);
    }
    /**
     * Get the number of rows.
     * @returns {number}
     */
    get rows() {
        const ret = wasm.wasmmatrix32_rows(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Set element at (i, j).
     * @param {number} i
     * @param {number} j
     * @param {number} value
     */
    set(i, j, value) {
        wasm.wasmmatrix32_set(this.__wbg_ptr, i, j, value);
    }
    /**
     * Return data as a flat array.
     * @returns {Float32Array}
     */
    toArray() {
        const ret = wasm.wasmmatrix32_toArray(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Transform a 3D point (x, y, z) by this 4×4 matrix. Returns [x', y', z'].
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @returns {Float32Array}
     */
    transformPoint(x, y, z) {
        const ret = wasm.wasmmatrix32_transformPoint(this.__wbg_ptr, x, y, z);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Transform a 3D direction (x, y, z) by the 3×3 part only (no translation).
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @returns {Float32Array}
     */
    transformVector(x, y, z) {
        const ret = wasm.wasmmatrix32_transformVector(this.__wbg_ptr, x, y, z);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Transpose (returns new matrix).
     * @returns {WasmMatrix32}
     */
    transpose() {
        const ret = wasm.wasmmatrix32_transpose(this.__wbg_ptr);
        return WasmMatrix32.__wrap(ret);
    }
}
if (Symbol.dispose) WasmMatrix32.prototype[Symbol.dispose] = WasmMatrix32.prototype.free;

/**
 * Result of PCA: mean, components, and explained variance.
 */
export class WasmPca {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmPcaFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmpca_free(ptr, 0);
    }
    /**
     * Principal components matrix (features × components); each column is a PC.
     * @returns {WasmMatrix}
     */
    getComponents() {
        const ret = wasm.wasmpca_getComponents(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Explained variance for each component.
     * @returns {WasmVector}
     */
    getExplainedVariance() {
        const ret = wasm.wasmpca_getExplainedVariance(this.__wbg_ptr);
        return WasmVector.__wrap(ret);
    }
    /**
     * Mean vector (one per feature).
     * @returns {WasmVector}
     */
    getMean() {
        const ret = wasm.wasmpca_getMean(this.__wbg_ptr);
        return WasmVector.__wrap(ret);
    }
    /**
     * Number of components.
     * @returns {number}
     */
    nComponents() {
        const ret = wasm.wasmpca_nComponents(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Run PCA on data matrix (rows = samples, cols = features).
     * Returns mean, principal components, and explained variance.
     * If `n_components` is 0, all components are kept.
     * @param {WasmMatrix} data
     * @param {number} n_components
     */
    constructor(data, n_components) {
        _assertClass(data, WasmMatrix);
        const ret = wasm.wasmpca_new(data.__wbg_ptr, n_components);
        this.__wbg_ptr = ret >>> 0;
        WasmPcaFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Project data onto principal components. Returns projected matrix (samples × components).
     * @param {WasmMatrix} data
     * @returns {WasmMatrix}
     */
    transform(data) {
        _assertClass(data, WasmMatrix);
        const ret = wasm.wasmpca_transform(this.__wbg_ptr, data.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmMatrix.__wrap(ret[0]);
    }
}
if (Symbol.dispose) WasmPca.prototype[Symbol.dispose] = WasmPca.prototype.free;

/**
 * Result of PSO: best position, cost, and iterations.
 */
export class WasmPsoResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmPsoResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmPsoResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmPsoResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmpsoresult_free(ptr, 0);
    }
    /**
     * Cost at best position.
     * @returns {number}
     */
    getBestCost() {
        const ret = wasm.wasmastarresult_getDist(this.__wbg_ptr);
        return ret;
    }
    /**
     * Best position found.
     * @returns {Float64Array}
     */
    getBestPosition() {
        const ret = wasm.wasmpsoresult_getBestPosition(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * Number of iterations performed.
     * @returns {number}
     */
    getIterations() {
        const ret = wasm.wasmgraph_numNodes(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) WasmPsoResult.prototype[Symbol.dispose] = WasmPsoResult.prototype.free;

/**
 * Result of simplex LP solve: solution vector, objective value, and status string.
 */
export class WasmSimplexResult {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSimplexResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsimplexresult_free(ptr, 0);
    }
    /**
     * Optimal objective value (c'x).
     * @returns {number}
     */
    getObjective() {
        const ret = wasm.wasmastarresult_getDist(this.__wbg_ptr);
        return ret;
    }
    /**
     * Status string: "optimal", "unbounded", or "infeasible".
     * @returns {string}
     */
    getStatus() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmsimplexresult_getStatus(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Solution vector x (length n).
     * @returns {WasmVector}
     */
    getX() {
        const ret = wasm.wasmsimplexresult_getX(this.__wbg_ptr);
        return WasmVector.__wrap(ret);
    }
    /**
     * Solve LP in standard form: minimize c'x subject to Ax = b, x >= 0.
     * Takes objective coefficients `c`, constraint matrix `A`, and RHS `b`.
     * Returns solution vector, objective value, and status ("optimal", "unbounded", "infeasible", or error message).
     * @param {WasmVector} c
     * @param {WasmMatrix} a
     * @param {WasmVector} b
     */
    constructor(c, a, b) {
        _assertClass(c, WasmVector);
        _assertClass(a, WasmMatrix);
        _assertClass(b, WasmVector);
        const ret = wasm.wasmsimplexresult_new(c.__wbg_ptr, a.__wbg_ptr, b.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        WasmSimplexResultFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) WasmSimplexResult.prototype[Symbol.dispose] = WasmSimplexResult.prototype.free;

/**
 * Result of economical SVD: U (m×k), V (n×k), sigma (length k) with k = min(m, n).
 */
export class WasmSvd {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmSvd.prototype);
        obj.__wbg_ptr = ptr;
        WasmSvdFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSvdFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsvd_free(ptr, 0);
    }
    /**
     * Singular values (vector).
     * @returns {WasmVector}
     */
    getSigma() {
        const ret = wasm.wasmsvd_getSigma(this.__wbg_ptr);
        return WasmVector.__wrap(ret);
    }
    /**
     * Left singular vectors U (matrix).
     * @returns {WasmMatrix}
     */
    getU() {
        const ret = wasm.wasmsvd_getU(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Right singular vectors V (matrix).
     * @returns {WasmMatrix}
     */
    getV() {
        const ret = wasm.wasmsvd_getV(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
}
if (Symbol.dispose) WasmSvd.prototype[Symbol.dispose] = WasmSvd.prototype.free;

/**
 * Linear SVM (binary classification). Train with X (rows = samples, cols = features) and labels ±1.
 */
export class WasmSvm {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSvmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsvm_free(ptr, 0);
    }
    /**
     * Train linear SVM. Labels must be ±1 (or positive → 1, else -1). Uses default options.
     * @param {WasmMatrix} x
     * @param {Float64Array} labels
     * @returns {WasmSvmResult}
     */
    static train(x, labels) {
        _assertClass(x, WasmMatrix);
        const ptr0 = passArrayF64ToWasm0(labels, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmsvm_train(x.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmSvmResult.__wrap(ret[0]);
    }
}
if (Symbol.dispose) WasmSvm.prototype[Symbol.dispose] = WasmSvm.prototype.free;

/**
 * RBF-kernel SVM (binary classification). Train with X, labels ±1, and gamma.
 */
export class WasmSvmRbf {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSvmRbfFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsvmrbf_free(ptr, 0);
    }
    /**
     * Train RBF SVM. Labels ±1. Gamma controls kernel width (e.g. 0.5).
     * @param {WasmMatrix} x
     * @param {Float64Array} labels
     * @param {number} gamma
     * @returns {WasmSvmRbfResult}
     */
    static train(x, labels, gamma) {
        _assertClass(x, WasmMatrix);
        const ptr0 = passArrayF64ToWasm0(labels, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmsvmrbf_train(x.__wbg_ptr, ptr0, len0, gamma);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmSvmRbfResult.__wrap(ret[0]);
    }
}
if (Symbol.dispose) WasmSvmRbf.prototype[Symbol.dispose] = WasmSvmRbf.prototype.free;

/**
 * Trained RBF-kernel SVM: support vectors, dual coefficients, bias, γ. Prediction via sign(Σ αᵢyᵢ K(svᵢ,x) + b).
 */
export class WasmSvmRbfResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmSvmRbfResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmSvmRbfResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSvmRbfResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsvmrbfresult_free(ptr, 0);
    }
    /**
     * Bias term.
     * @returns {number}
     */
    getBias() {
        const ret = wasm.wasmsvmrbfresult_getBias(this.__wbg_ptr);
        return ret;
    }
    /**
     * RBF kernel parameter γ.
     * @returns {number}
     */
    getGamma() {
        const ret = wasm.wasmsvmrbfresult_getGamma(this.__wbg_ptr);
        return ret;
    }
    /**
     * Support vectors matrix (n_sv × n_features).
     * @returns {WasmMatrix}
     */
    getSupportVectors() {
        const ret = wasm.wasmsvmrbfresult_getSupportVectors(this.__wbg_ptr);
        return WasmMatrix.__wrap(ret);
    }
    /**
     * Predict label for one sample: +1 or -1.
     * @param {Float64Array} sample
     * @returns {number}
     */
    predict(sample) {
        const ptr0 = passArrayF64ToWasm0(sample, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmsvmrbfresult_predict(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Predict labels for all rows of X. Returns array of +1 or -1.
     * @param {WasmMatrix} x
     * @returns {Float64Array}
     */
    predictAll(x) {
        _assertClass(x, WasmMatrix);
        const ret = wasm.wasmsvmrbfresult_predictAll(this.__wbg_ptr, x.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
}
if (Symbol.dispose) WasmSvmRbfResult.prototype[Symbol.dispose] = WasmSvmRbfResult.prototype.free;

/**
 * Trained linear SVM: weight vector and bias for prediction sign(w·x + b).
 */
export class WasmSvmResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmSvmResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmSvmResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSvmResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsvmresult_free(ptr, 0);
    }
    /**
     * Bias term.
     * @returns {number}
     */
    getBias() {
        const ret = wasm.wasmastarresult_getDist(this.__wbg_ptr);
        return ret;
    }
    /**
     * Weight vector (one per feature).
     * @returns {WasmVector}
     */
    getWeights() {
        const ret = wasm.wasmsvmresult_getWeights(this.__wbg_ptr);
        return WasmVector.__wrap(ret);
    }
    /**
     * Predict label for one sample: +1 or -1. `sample` is a row of features (length = n_features).
     * @param {Float64Array} sample
     * @returns {number}
     */
    predict(sample) {
        const ptr0 = passArrayF64ToWasm0(sample, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmsvmresult_predict(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Predict labels for all rows of X. Returns array of +1 or -1.
     * @param {WasmMatrix} x
     * @returns {Float64Array}
     */
    predictAll(x) {
        _assertClass(x, WasmMatrix);
        const ret = wasm.wasmsvmresult_predictAll(this.__wbg_ptr, x.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
}
if (Symbol.dispose) WasmSvmResult.prototype[Symbol.dispose] = WasmSvmResult.prototype.free;

/**
 * A vector accessible from JavaScript.
 */
export class WasmVector {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmVector.prototype);
        obj.__wbg_ptr = ptr;
        WasmVectorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmVectorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmvector_free(ptr, 0);
    }
    /**
     * Vector addition.
     * @param {WasmVector} other
     * @returns {WasmVector}
     */
    add(other) {
        _assertClass(other, WasmVector);
        const ret = wasm.wasmvector_add(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmVector.__wrap(ret[0]);
    }
    /**
     * Dot product with another vector.
     * @param {WasmVector} other
     * @returns {number}
     */
    dot(other) {
        _assertClass(other, WasmVector);
        const ret = wasm.wasmvector_dot(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Euclidean distance to another vector.
     * @param {WasmVector} other
     * @returns {number}
     */
    euclideanDistance(other) {
        _assertClass(other, WasmVector);
        const ret = wasm.wasmvector_euclideanDistance(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Create a vector from a Float64Array.
     * @param {Float64Array} data
     * @returns {WasmVector}
     */
    static fromArray(data) {
        const ptr0 = passArrayF64ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmvector_fromArray(ptr0, len0);
        return WasmVector.__wrap(ret);
    }
    /**
     * Get element at index.
     * @param {number} i
     * @returns {number}
     */
    get(i) {
        const ret = wasm.wasmvector_get(this.__wbg_ptr, i);
        return ret;
    }
    /**
     * Check if empty.
     * @returns {boolean}
     */
    isEmpty() {
        const ret = wasm.wasmvector_isEmpty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Get the length.
     * @returns {number}
     */
    get len() {
        const ret = wasm.wasmmatrix32_rows(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Linear interpolation with another vector: (1 - t) * self + t * other.
     * @param {WasmVector} other
     * @param {number} t
     * @returns {WasmVector}
     */
    lerp(other, t) {
        _assertClass(other, WasmVector);
        const ret = wasm.wasmvector_lerp(this.__wbg_ptr, other.__wbg_ptr, t);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmVector.__wrap(ret[0]);
    }
    /**
     * Create a new zero vector with given length.
     * @param {number} len
     */
    constructor(len) {
        const ret = wasm.wasmvector_new(len);
        this.__wbg_ptr = ret >>> 0;
        WasmVectorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Euclidean norm.
     * @returns {number}
     */
    norm() {
        const ret = wasm.wasmvector_norm(this.__wbg_ptr);
        return ret;
    }
    /**
     * Scalar multiplication.
     * @param {number} scalar
     * @returns {WasmVector}
     */
    scale(scalar) {
        const ret = wasm.wasmvector_scale(this.__wbg_ptr, scalar);
        return WasmVector.__wrap(ret);
    }
    /**
     * Set element at index.
     * @param {number} i
     * @param {number} value
     */
    set(i, value) {
        wasm.wasmvector_set(this.__wbg_ptr, i, value);
    }
    /**
     * Vector subtraction.
     * @param {WasmVector} other
     * @returns {WasmVector}
     */
    sub(other) {
        _assertClass(other, WasmVector);
        const ret = wasm.wasmvector_sub(this.__wbg_ptr, other.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmVector.__wrap(ret[0]);
    }
    /**
     * Return data as Float64Array.
     * @returns {Float64Array}
     */
    toArray() {
        const ret = wasm.wasmvector_toArray(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
}
if (Symbol.dispose) WasmVector.prototype[Symbol.dispose] = WasmVector.prototype.free;

/**
 * Apply window to signal. Returns windowed signal (signal[i] * window[i]).
 * @param {Float64Array} signal
 * @param {Float64Array} window
 * @returns {Float64Array}
 */
export function applyWindow(signal, window) {
    const ptr0 = passArrayF64ToWasm0(signal, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(window, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.applyWindow(ptr0, len0, ptr1, len1);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v3 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v3;
}

/**
 * Blackman window.
 * @param {number} len
 * @returns {Float64Array}
 */
export function blackman(len) {
    const ret = wasm.blackman(len);
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * 1D convolution (full).
 * @param {Float64Array} signal
 * @param {Float64Array} kernel
 * @returns {Float64Array}
 */
export function conv1d(signal, kernel) {
    const ptr0 = passArrayF64ToWasm0(signal, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(kernel, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.conv1d(ptr0, len0, ptr1, len1);
    var v3 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v3;
}

/**
 * 1D convolution same-length.
 * @param {Float64Array} signal
 * @param {Float64Array} kernel
 * @returns {Float64Array}
 */
export function conv1dSame(signal, kernel) {
    const ptr0 = passArrayF64ToWasm0(signal, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(kernel, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.conv1dSame(ptr0, len0, ptr1, len1);
    var v3 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v3;
}

/**
 * DCT-II forward.
 * @param {Float64Array} signal
 * @returns {Float64Array}
 */
export function dct2Forward(signal) {
    const ptr0 = passArrayF64ToWasm0(signal, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.dct2Forward(ptr0, len0);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * DCT-III inverse.
 * @param {Float64Array} coeffs
 * @returns {Float64Array}
 */
export function dct2Inverse(coeffs) {
    const ptr0 = passArrayF64ToWasm0(coeffs, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.dct2Inverse(ptr0, len0);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * Haar DWT forward (even length).
 * @param {Float64Array} signal
 * @returns {Float64Array}
 */
export function dwtHaarForward(signal) {
    const ptr0 = passArrayF64ToWasm0(signal, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.dwtHaarForward(ptr0, len0);
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * Haar DWT inverse.
 * @param {Float64Array} coeffs
 * @returns {Float64Array}
 */
export function dwtHaarInverse(coeffs) {
    const ptr0 = passArrayF64ToWasm0(coeffs, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.dwtHaarInverse(ptr0, len0);
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * FBM with Perlin base noise. Typical values: lacunarity 2.0, persistence 0.5.
 * @param {number} x
 * @param {number} y
 * @param {number} octaves
 * @param {number} lacunarity
 * @param {number} persistence
 * @returns {number}
 */
export function fbm2dPerlin(x, y, octaves, lacunarity, persistence) {
    const ret = wasm.fbm2dPerlin(x, y, octaves, lacunarity, persistence);
    return ret;
}

/**
 * Forward real FFT. Input length must be power of 2.
 * Returns spectrum as [re0, im0, re1, im1, ...] (interleaved).
 * @param {Float64Array} signal
 * @returns {Float64Array}
 */
export function fftForwardReal(signal) {
    const ptr0 = passArrayF64ToWasm0(signal, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.fftForwardReal(ptr0, len0);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * Inverse FFT. Input as interleaved [re0, im0, re1, im1, ...]. Returns real part.
 * @param {Float64Array} spectrum
 * @returns {Float64Array}
 */
export function fftInverse(spectrum) {
    const ptr0 = passArrayF64ToWasm0(spectrum, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.fftInverse(ptr0, len0);
    if (ret[3]) {
        throw takeFromExternrefTable0(ret[2]);
    }
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * Hamming window.
 * @param {number} len
 * @returns {Float64Array}
 */
export function hamming(len) {
    const ret = wasm.hamming(len);
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * Hann window.
 * @param {number} len
 * @returns {Float64Array}
 */
export function hann(len) {
    const ret = wasm.hann(len);
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * Backtracking line search: find step length α so that Armijo holds.
 *
 * `x` and `d` are the current point and search direction. `f` is the cost at `x`,
 * `g_dot_d` is the gradient at `x` dotted with `d`. `cost_fn` is a JS function
 * `(point: Float64Array) => number` that evaluates the cost at a point.
 * @param {Float64Array} x
 * @param {Float64Array} d
 * @param {number} f
 * @param {number} g_dot_d
 * @param {Function} cost_fn
 * @returns {number}
 */
export function lineSearchBacktracking(x, d, f, g_dot_d, cost_fn) {
    const ptr0 = passArrayF64ToWasm0(x, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(d, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.lineSearchBacktracking(ptr0, len0, ptr1, len1, f, g_dot_d, cost_fn);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

/**
 * 2D Perlin noise at (x, y). Output is approximately in [-1, 1].
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function perlin2d(x, y) {
    const ret = wasm.perlin2d(x, y);
    return ret;
}

/**
 * Run PSO to minimize a cost function over a box.
 *
 * `cost_fn` is a JS function `(position: Float64Array) => number`.
 * `lower` and `upper` define the search bounds per dimension.
 * @param {Float64Array} lower
 * @param {Float64Array} upper
 * @param {number} num_particles
 * @param {number} max_iters
 * @param {Function} cost_fn
 * @returns {WasmPsoResult}
 */
export function psoMinimize(lower, upper, num_particles, max_iters, cost_fn) {
    const ptr0 = passArrayF64ToWasm0(lower, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayF64ToWasm0(upper, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.psoMinimize(ptr0, len0, ptr1, len1, num_particles, max_iters, cost_fn);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return WasmPsoResult.__wrap(ret[0]);
}

/**
 * Tukey window. Alpha in [0, 1]: 0=rectangular, 1=Hann.
 * @param {number} len
 * @param {number} alpha
 * @returns {Float64Array}
 */
export function tukey(len, alpha) {
    const ret = wasm.tukey(len, alpha);
    var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v1;
}

/**
 * Wave height at (u, v) in [0, 1]². Returns value in [0, 1].
 * @param {number} u
 * @param {number} v
 * @returns {number}
 */
export function wave2d(u, v) {
    const ret = wasm.wave2d(u, v);
    return ret;
}

/**
 * Wave height with configurable wave numbers (radians per unit).
 * @param {number} u
 * @param {number} v
 * @param {number} k1
 * @param {number} k2
 * @returns {number}
 */
export function wave2dParams(u, v, k1, k2) {
    const ret = wasm.wave2dParams(u, v, k1, k2);
    return ret;
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_4708e0c13bdc8e95: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_length_f7386240689107f3: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_new_with_length_6523745c0bd32809: function(arg0) {
            const ret = new Float64Array(arg0 >>> 0);
            return ret;
        },
        __wbg_set_a7e6b10165583fc4: function(arg0, arg1, arg2) {
            arg0.set(getArrayF64FromWasm0(arg1, arg2));
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./mathlib_bg.js": import0,
    };
}

const WasmAstarResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmastarresult_free(ptr >>> 0, 1));
const WasmBfsResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmbfsresult_free(ptr >>> 0, 1));
const WasmCgFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcg_free(ptr >>> 0, 1));
const WasmCholeskyFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcholesky_free(ptr >>> 0, 1));
const WasmDStarLiteResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmdstarliteresult_free(ptr >>> 0, 1));
const WasmDbscanFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmdbscan_free(ptr >>> 0, 1));
const WasmDijkstraResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmdijkstraresult_free(ptr >>> 0, 1));
const WasmDistanceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmdistance_free(ptr >>> 0, 1));
const WasmGraphFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmgraph_free(ptr >>> 0, 1));
const WasmKmeansFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmkmeans_free(ptr >>> 0, 1));
const WasmLuFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmlu_free(ptr >>> 0, 1));
const WasmMatrixFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmmatrix_free(ptr >>> 0, 1));
const WasmMatrix32Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmmatrix32_free(ptr >>> 0, 1));
const WasmPcaFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmpca_free(ptr >>> 0, 1));
const WasmPsoResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmpsoresult_free(ptr >>> 0, 1));
const WasmSimplexResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsimplexresult_free(ptr >>> 0, 1));
const WasmSvdFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsvd_free(ptr >>> 0, 1));
const WasmSvmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsvm_free(ptr >>> 0, 1));
const WasmSvmRbfFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsvmrbf_free(ptr >>> 0, 1));
const WasmSvmRbfResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsvmrbfresult_free(ptr >>> 0, 1));
const WasmSvmResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsvmresult_free(ptr >>> 0, 1));
const WasmVectorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmvector_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

let cachedInt32ArrayMemory0 = null;
function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArrayF32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getFloat32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedInt32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('mathlib_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
