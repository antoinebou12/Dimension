/* tslint:disable */
/* eslint-disable */

/**
 * Add a new entity as child of the root with the given primitive (e.g. "cube", "sphere"). Applied next frame; the new entity becomes selected.
 */
export function add_entity(primitive_name: string): void;

/**
 * Return the current gizmo mode: "translate", "rotate", or "scale". Gizmo is shown only when an entity is picked.
 */
export function get_gizmo_mode(): string;

/**
 * Return material names (newline-separated) for the HTML material dropdown.
 */
export function get_material_names(): string;

/**
 * Sentinel value for "no entity selected". Compare get_selected_entity() against this.
 */
export function get_no_entity(): number;

/**
 * Return the current world's primitive tree as a string (for the tree panel).
 */
export function get_primitive_tree(): string;

/**
 * Return the currently selected entity id, or get_no_entity() if none.
 */
export function get_selected_entity(): number;

/**
 * Local position of the selected entity as "x,y,z". Empty if none selected.
 */
export function get_selected_entity_local_position(): string;

/**
 * Local rotation (roll, pitch, yaw in radians) of the selected entity as "r,p,y". Empty if none selected.
 */
export function get_selected_entity_local_rotation(): string;

/**
 * Local scale of the selected entity as "x,y,z". Empty if none selected.
 */
export function get_selected_entity_local_scale(): string;

/**
 * World position of the selected entity as "x,y,z". Empty if none selected.
 */
export function get_selected_entity_world_position(): string;

export function main_js(): void;

/**
 * Remove the currently selected entity. Root cannot be removed.
 */
export function remove_selected_entity(): void;

/**
 * Switch the active scene. Call with "default" or "curves"; applied on next frame.
 */
export function set_demo(name: string): void;

/**
 * Set local position of an entity. Applied next frame.
 */
export function set_entity_local_position(entity_id: number, x: number, y: number, z: number): void;

/**
 * Set local rotation of an entity. Pass roll, pitch, yaw in radians. Applied next frame.
 */
export function set_entity_local_rotation(entity_id: number, roll: number, pitch: number, yaw: number): void;

/**
 * Set local scale of an entity. Applied next frame.
 */
export function set_entity_local_scale(entity_id: number, x: number, y: number, z: number): void;

/**
 * Set the material for an entity. No-op if id is invalid.
 */
export function set_entity_material(entity_id: number, material_name: string): void;

/**
 * Set the primitive for an entity. Name: "cube", "sphere", "line", "bezier", etc.
 */
export function set_entity_primitive(entity_id: number, primitive_name: string): void;

/**
 * Set world position of an entity. Applied next frame.
 */
export function set_entity_world_position(entity_id: number, x: number, y: number, z: number): void;

/**
 * Set the gizmo mode: "translate", "rotate", or "scale". Changes which transform handles are shown for the selected entity.
 */
export function set_gizmo_mode(mode: string): void;

/**
 * Set the selected entity. Pass get_no_entity() to clear selection.
 */
export function set_selected_entity(id: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly add_entity: (a: number, b: number) => void;
    readonly get_gizmo_mode: () => [number, number];
    readonly get_material_names: () => [number, number];
    readonly get_no_entity: () => number;
    readonly get_primitive_tree: () => [number, number];
    readonly get_selected_entity: () => number;
    readonly get_selected_entity_local_position: () => [number, number];
    readonly get_selected_entity_local_rotation: () => [number, number];
    readonly get_selected_entity_local_scale: () => [number, number];
    readonly get_selected_entity_world_position: () => [number, number];
    readonly main_js: () => void;
    readonly remove_selected_entity: () => void;
    readonly set_demo: (a: number, b: number) => void;
    readonly set_entity_local_position: (a: number, b: number, c: number, d: number) => void;
    readonly set_entity_local_rotation: (a: number, b: number, c: number, d: number) => void;
    readonly set_entity_local_scale: (a: number, b: number, c: number, d: number) => void;
    readonly set_entity_material: (a: number, b: number, c: number) => void;
    readonly set_entity_primitive: (a: number, b: number, c: number) => void;
    readonly set_entity_world_position: (a: number, b: number, c: number, d: number) => void;
    readonly set_gizmo_mode: (a: number, b: number) => void;
    readonly set_selected_entity: (a: number) => void;
    readonly main: (a: number, b: number) => number;
    readonly wasm_bindgen__closure__destroy__h49c2c8e30cc92599: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h338494b43caf1fe0: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h0f7c82f2acc0248f: (a: number, b: number) => void;
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
