/* tslint:disable */
/* eslint-disable */

/**
 * WASM-exposed aggregate query.
 *
 * Builds an in-memory student id from the given school +
 * names and returns the serialized form as JSON. Demonstrates
 * that aggregate construction (pure logic) works in WASM.
 */
export function build_student_summary(school_uuid: string, first_name: string, last_name: string): any;

/**
 * WASM-exposed capability name lookup.
 *
 * Returns the kebab-case name of a capability, or `None` if
 * the name is not recognized by the engine.
 */
export function capability_known(name: string): boolean;

/**
 * WASM-exposed engine version.
 *
 * Returns the engine version string, useful for client-side
 * compatibility checks.
 */
export function engine_version(): string;

/**
 * Initialize the WASM module. Call once at startup.
 *
 * Sets up the panic hook so WASM panics surface as console
 * errors instead of opaque aborts.
 */
export function init(): void;

/**
 * WASM-exposed admission validator.
 *
 * Validates a student's admission payload in the browser
 * without any server round-trip. Returns a JSON object with
 * `ok: bool` and either `student_id` (on success) or `errors`
 * (on validation failure).
 */
export function validate_admission(school_uuid: string, first_name: string, last_name: string, email?: string | null): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly build_student_summary: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly capability_known: (a: number, b: number) => number;
    readonly engine_version: (a: number) => void;
    readonly validate_admission: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly init: () => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export4: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
