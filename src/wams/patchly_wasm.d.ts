/* tslint:disable */
/* eslint-disable */

/**
 * Applier for pacthes with streaming output supoort.
 */
export class PatchApplier {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a chunk of patch data.
     */
    add_patch_chunk(chunk: Uint8Array): void;
    /**
     * Add a chunk of source (old file) data.
     */
    add_source_chunk(chunk: Uint8Array): void;
    /**
     * Get expected source size from patch metadata.
     */
    expected_source_size(): bigint;
    /**
     * Get expected target size from patch metadata.
     */
    expected_target_size(): bigint;
    /**
     * Finalize patch loading and parse header.
     */
    finalize_patch(): void;
    /**
     * Check if there's more output to read.
     */
    has_more_output(): boolean;
    /**
     * Create a new PatchApplier
     */
    constructor();
    /**
     * Get next chunk of output data.
     */
    next_output_chunk(max_size: number): Uint8Array;
    /**
     * Prepare for streaming output.
     */
    prepare(): void;
    /**
     * Get remaining bytes to output
     */
    remaining_output_size(): bigint;
    /**
     * Reset the applier for reuse.
     */
    reset(): void;
    /**
     * Set the patch data.
     */
    set_patch(patch_data: Uint8Array): void;
    /**
     * Get current source size.
     */
    source_size(): number;
    /**
     * Validate source file before applying.
     */
    validate_source(): boolean;
}

/**
 * Streaming patch build
 */
export class PatchBuilder {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a chunk of source (old file) data.
     */
    add_source_chunk(chunk: Uint8Array): void;
    /**
     * Add a chunk of target (new file) data.
     * This immediately generates patch output - call flush_output() to retrieve it.
     */
    add_target_chunk(chunk: Uint8Array): void;
    /**
     * Check if source and target files are identical.
     * Only accurate after all data has been processed.
     */
    are_files_identical(): boolean;
    /**
     * Finalize source processing.
     */
    finalize_source(): void;
    /**
     * Finalize target processing.
     * Call this after all target chunks have been added.
     */
    finalize_target(): void;
    /**
     * Get next chunk of patch output.
     * Returns serialized patch data ready to write to file.
     */
    flush_output(max_size: number): Uint8Array;
    /**
     * Check if there's patch output available to read.
     */
    has_output(): boolean;
    /**
     * Create a new PatchBuilder with default chunk size
     */
    constructor();
    /**
     * Get approximate pending output size
     */
    pending_output_size(): number;
    /**
     * Reset the builder for reuse.
     */
    reset(): void;
    /**
     * Set the expected total target size.
     * Must be called before add_target_chunk() for proper header generation.
     */
    set_target_size(size: bigint): void;
    /**
     * Get current source size (bytes received so far).
     */
    source_size(): number;
    /**
     * Get current target size (bytes received so far).
     */
    target_size(): number;
}

/**
 * WASM-bindable streaming hash builder.
 * Use this to calculate hash incrementally from JavaScript without BigInt allocations.
 */
export class WasmHashBuilder {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Finalize and return the hash as a hex string
     */
    finalize(): string;
    /**
     * Finalize and return the hash as a u64 (for comparison)
     */
    finalize_u64(): bigint;
    /**
     * Create a new hash builder
     */
    constructor();
    /**
     * Update the hash with a chunk of data
     */
    update(data: Uint8Array): void;
}

/**
 * Calculate hash of data
 */
export function hash_data(data: Uint8Array): string;

/**
 * Parse patch header and return JSON with metadata and instructions.
 * This is a lightweight function for the TypeScript-based applier.
 * Returns JSON string with structure:
 * {
 *   "sourceSize": number,
 *   "sourceHash": string (hex),
 *   "targetSize": number,
 *   "instructions": [
 *     { "type": "copy", "offset": number, "length": number } |
 *     { "type": "insert", "patchOffset": number, "length": number }
 *   ]
 * }
 */
export function parse_patch_header(patch_data: Uint8Array): string;

/**
 * Parse ONLY the patch header (33 bytes) without parsing instructions.
 * Returns JSON: { "sourceSize": number, "sourceHash": string, "targetSize": number, "headerSize": 33 }
 * TypeScript will parse instructions directly from OPFS to avoid loading entire patch.
 */
export function parse_patch_header_only(header_data: Uint8Array): string;

/**
 * Get the library version
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_patchbuilder_free: (a: number, b: number) => void;
    readonly patchbuilder_new: () => number;
    readonly patchbuilder_add_source_chunk: (a: number, b: number, c: number) => void;
    readonly patchbuilder_finalize_source: (a: number) => void;
    readonly patchbuilder_set_target_size: (a: number, b: bigint) => void;
    readonly patchbuilder_add_target_chunk: (a: number, b: number, c: number) => void;
    readonly patchbuilder_finalize_target: (a: number) => void;
    readonly patchbuilder_source_size: (a: number) => number;
    readonly patchbuilder_target_size: (a: number) => number;
    readonly patchbuilder_are_files_identical: (a: number) => number;
    readonly patchbuilder_has_output: (a: number) => number;
    readonly patchbuilder_flush_output: (a: number, b: number) => [number, number];
    readonly patchbuilder_pending_output_size: (a: number) => number;
    readonly patchbuilder_reset: (a: number) => void;
    readonly __wbg_patchapplier_free: (a: number, b: number) => void;
    readonly patchapplier_new: () => number;
    readonly patchapplier_add_source_chunk: (a: number, b: number, c: number) => void;
    readonly patchapplier_set_patch: (a: number, b: number, c: number) => void;
    readonly patchapplier_source_size: (a: number) => number;
    readonly patchapplier_validate_source: (a: number) => [number, number, number];
    readonly patchapplier_expected_source_size: (a: number) => [bigint, number, number];
    readonly patchapplier_expected_target_size: (a: number) => [bigint, number, number];
    readonly patchapplier_prepare: (a: number) => [number, number];
    readonly patchapplier_has_more_output: (a: number) => number;
    readonly patchapplier_next_output_chunk: (a: number, b: number) => [number, number];
    readonly patchapplier_add_patch_chunk: (a: number, b: number, c: number) => void;
    readonly patchapplier_finalize_patch: (a: number) => [number, number];
    readonly patchapplier_remaining_output_size: (a: number) => bigint;
    readonly patchapplier_reset: (a: number) => void;
    readonly parse_patch_header: (a: number, b: number) => [number, number, number, number];
    readonly parse_patch_header_only: (a: number, b: number) => [number, number, number, number];
    readonly version: () => [number, number];
    readonly __wbg_wasmhashbuilder_free: (a: number, b: number) => void;
    readonly wasmhashbuilder_new: () => number;
    readonly wasmhashbuilder_update: (a: number, b: number, c: number) => void;
    readonly wasmhashbuilder_finalize: (a: number) => [number, number];
    readonly wasmhashbuilder_finalize_u64: (a: number) => bigint;
    readonly hash_data: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
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
