/* tslint:disable */
/* eslint-disable */

/**
 * Return the bodies tree as a newline-separated string for the HTML tree panel. Updated each frame.
 */
export function get_bodies_tree(): string;

/**
 * Return current body count (rigid + soft, excluding ground) for HTML display.
 */
export function get_body_count(): number;

/**
 * Return whether debug logging is enabled.
 */
export function get_debug_physics(): boolean;

/**
 * Return total momentum magnitude as a string.
 */
export function get_total_momentum(): string;

export function main_js(): void;

/**
 * Reset the scene from HTML toolbar. Applied on next frame.
 */
export function reset_scene(): void;

/**
 * Enable or disable verbose debug logging. Use `?debug=1` in URL to auto-enable.
 */
export function set_debug_physics(enabled: boolean): void;

/**
 * Spawn a box from HTML toolbar. Applied on next frame.
 */
export function spawn_box_body(): void;

/**
 * Spawn a jelly (soft body) from HTML toolbar. Applied on next frame.
 */
export function spawn_jelly(): void;

/**
 * Spawn a sphere from HTML toolbar. Applied on next frame.
 */
export function spawn_sphere(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_bodies_tree: () => [number, number];
    readonly get_body_count: () => number;
    readonly get_debug_physics: () => number;
    readonly get_total_momentum: () => [number, number];
    readonly main_js: () => void;
    readonly reset_scene: () => void;
    readonly set_debug_physics: (a: number) => void;
    readonly spawn_box_body: () => void;
    readonly spawn_jelly: () => void;
    readonly spawn_sphere: () => void;
    readonly main: (a: number, b: number) => number;
    readonly wasm_bindgen__closure__destroy__h10f167144040dd12: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h16ce8a352a8d472c: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hd056974acd1e2972: (a: number, b: number) => void;
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
