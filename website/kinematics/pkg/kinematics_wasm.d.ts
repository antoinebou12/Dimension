/* tslint:disable */
/* eslint-disable */

/**
 * Return the number of nodes in the active chain's armature. Updated each frame.
 */
export function get_armature_node_count(): number;

/**
 * Return the armature tree as a newline-separated string for the HTML tree panel. Updated each frame.
 */
export function get_armature_tree(): string;

/**
 * Return comma-separated list of solver names for the UI.
 */
export function get_available_solvers(): string;

/**
 * Return whether debug logging is enabled.
 */
export function get_debug_kinematics(): boolean;

/**
 * Return the end-effector position as "x,y,z". Updated each frame.
 */
export function get_ee_position(): string;

/**
 * Return the current end-effector node index. Updated each frame.
 */
export function get_end_effector_index(): number;

/**
 * Return the last Hessian snapshot when solver is Hessian: object with hessian (Float64Array), size, error, or null.
 */
export function get_hessian_snapshot(): any;

/**
 * Return the current IK solver. Updated each frame.
 */
export function get_ik_solver(): string;

/**
 * Return the IK target position as "x,y,z". Updated each frame.
 */
export function get_target_position(): string;

export function main_js(): void;

/**
 * Trigger randomize IK target. Applied on next frame.
 */
export function randomize_target(): void;

/**
 * Reset the scene to initial state. Applied on next frame.
 */
export function reset_scene(): void;

/**
 * Set the active chain. Call with "a" or "b"; applied on next frame.
 */
export function set_active_chain(name: string): void;

/**
 * Set the arm preset for the active chain. Call with "spherical", "revolute", "mixed", or "snake"; applied on next frame.
 */
export function set_arm_preset(name: string): void;

/**
 * Enable or disable verbose debug logging. Use `?debug=1` in URL to auto-enable.
 */
export function set_debug_kinematics(enabled: boolean): void;

/**
 * Set the end-effector node index. Applied on next frame.
 */
export function set_end_effector(idx: number): void;

/**
 * Set the IK solver. Call with "fabrik" or "jacobian"; applied on next frame.
 */
export function set_ik_solver(name: string): void;

/**
 * Set the IK target to the current end-effector position for the active chain. Applied on next frame.
 */
export function set_target_to_ee(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_armature_node_count: () => number;
    readonly get_armature_tree: () => [number, number];
    readonly get_available_solvers: () => [number, number];
    readonly get_debug_kinematics: () => number;
    readonly get_ee_position: () => [number, number];
    readonly get_end_effector_index: () => number;
    readonly get_hessian_snapshot: () => any;
    readonly get_ik_solver: () => [number, number];
    readonly get_target_position: () => [number, number];
    readonly main_js: () => void;
    readonly randomize_target: () => void;
    readonly reset_scene: () => void;
    readonly set_active_chain: (a: number, b: number) => void;
    readonly set_arm_preset: (a: number, b: number) => void;
    readonly set_debug_kinematics: (a: number) => void;
    readonly set_end_effector: (a: number) => void;
    readonly set_ik_solver: (a: number, b: number) => void;
    readonly set_target_to_ee: () => void;
    readonly main: (a: number, b: number) => number;
    readonly wasm_bindgen__closure__destroy__h12a98130bbdbd912: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h027e5b6af5ffaf77: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hdbec3385a70c55d6: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
