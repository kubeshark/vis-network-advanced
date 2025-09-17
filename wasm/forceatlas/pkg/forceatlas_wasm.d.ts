/* tslint:disable */
/* eslint-disable */
export function compute_forces(positions: Float64Array, masses: Float64Array, theta: number, gravitational: number, avoid_overlap: number, radii: Float64Array, degrees: Float64Array): Float64Array;
export function compute_springs(positions: Float64Array, from_idx: Float64Array, to_idx: Float64Array, lengths: Float64Array, spring_constant: number): Float64Array;
export function compute_central_gravity(positions: Float64Array, masses: Float64Array, central_gravity: number, degrees: Float64Array): Float64Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly compute_central_gravity: (a: any, b: any, c: number, d: any) => any;
  readonly compute_forces: (a: any, b: any, c: number, d: number, e: number, f: any, g: any) => any;
  readonly compute_springs: (a: any, b: any, c: any, d: any, e: number) => any;
  readonly __wbindgen_export_0: WebAssembly.Table;
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
