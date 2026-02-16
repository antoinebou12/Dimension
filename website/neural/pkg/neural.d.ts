/* tslint:disable */
/* eslint-disable */

/**
 * WASM-facing neural IK wrapper for inference.
 */
export class NeuralIkWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * DOF (number of output joints).
     */
    dof(): number;
    /**
     * Return the current input buffer (for use with external model inference).
     */
    getInput(): Float32Array;
    /**
     * Create a placeholder for inference (model must be set via set_weights or ONNX).
     */
    constructor(dof: number, use_current_state: boolean);
    /**
     * Set current joint state (length must match dof when use_current_state was true).
     */
    setCurrentJoints(joints: Float32Array): void;
    /**
     * Set target position (x, y, z) for next prediction.
     */
    setTarget(x: number, y: number, z: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_neuralikwasm_free: (a: number, b: number) => void;
    readonly neuralikwasm_dof: (a: number) => number;
    readonly neuralikwasm_getInput: (a: number) => [number, number];
    readonly neuralikwasm_new: (a: number, b: number) => number;
    readonly neuralikwasm_setCurrentJoints: (a: number, b: number, c: number) => void;
    readonly neuralikwasm_setTarget: (a: number, b: number, c: number, d: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
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
