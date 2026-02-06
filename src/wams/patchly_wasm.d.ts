/* tslint:disable */
/* eslint-disable */

/**
 * Applier for pacthes with streaming output supoort.
 */
export class PatchApplier {
    free(): void;
    [Symbol.dispose](): void;
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
 *
 * # Memory Usage
 * - Source: O(blocks) - use BlockIndex
 * - Target: Processed incrementally via StreamingDiff
 *
 * # Usage Flow
 * 1. Call add_source_chunk() for all source data
 * 2. Call finalize_source() when done with source
 * 3. Call add_target_chunk() for all target data
 * 4. Call prepare_patch() to prepare for streaming output
 * 5. Call next_patch_chunk() repeatedly until has_more_patch() returns false
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
     * Get approximate pending output size (for progress calculation).
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
 * Calculate hash of data
 */
export function hash_data(data: Uint8Array): string;

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
    readonly patchapplier_remaining_output_size: (a: number) => bigint;
    readonly patchapplier_reset: (a: number) => void;
    readonly version: () => [number, number];
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
