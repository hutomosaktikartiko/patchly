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
     * Get patch info as JSON string.
     */
    get_patch_info(): string;
    /**
     * Check if there's more output to read
     */
    has_more_output(): boolean;
    /**
     * Check if patch has been loaded.
     */
    is_patch_loaded(): boolean;
    /**
     * Create a new PatchApplier
     */
    constructor();
    /**
     * Get next chunk of output for streaming to OPFS.
     */
    next_output_chunk(max_chunk_size: number): Uint8Array;
    /**
     * Get output progress as percentage (0-100).
     */
    output_progress(): number;
    /**
     * Prepare for streaming output.
     */
    prepare(): void;
    /**
     * Get progress as percentage (0-100)
     */
    progress(source_expected: number): number;
    /**
     * Get remaining bytes to output
     */
    remaining_output_size(): number;
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
     * Get total output size
     */
    total_output_size(): number;
    /**
     * Validate source file before applying.
     */
    validate_source(): boolean;
}

/**
 * Builder for creating patches from streamed file chunks.
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
     */
    add_target_chunk(chunk: Uint8Array): void;
    /**
     * Check if source and target files are indentical.
     * Files are identical if both size AND hash match
     */
    are_files_identical(): boolean;
    /**
     * Finalize and generate the patch.
     * returns serialized patch data.
     */
    finalize(): Uint8Array;
    /**
     * Get patch info without without full serialization (for perview).
     * Returns JSON string with stats
     */
    get_preview_info(): string;
    /**
     * Create a new PatchBuilder with default chunk size
     */
    constructor();
    /**
     * Get progress as percentage (0-100) based on expected sizes.
     * Returns source progress if target_expected is 0.
     */
    progress(source_expected: number, target_expected: number): number;
    /**
     * Reset the builder for reuse.
     */
    reset(): void;
    /**
     * Get current source size (bytes received so far).
     */
    source_size(): number;
    /**
     * Get current target size (bytes received so far).
     */
    target_size(): number;
    /**
     * Create a new PatchBuilder with custom chunk size.
     */
    static with_chunk_size(chunk_size: number): PatchBuilder;
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
    readonly patchbuilder_with_chunk_size: (a: number) => number;
    readonly patchbuilder_add_source_chunk: (a: number, b: number, c: number) => void;
    readonly patchbuilder_add_target_chunk: (a: number, b: number, c: number) => void;
    readonly patchbuilder_source_size: (a: number) => number;
    readonly patchbuilder_target_size: (a: number) => number;
    readonly patchbuilder_progress: (a: number, b: number, c: number) => number;
    readonly patchbuilder_finalize: (a: number) => [number, number, number, number];
    readonly patchbuilder_get_preview_info: (a: number) => [number, number];
    readonly patchbuilder_are_files_identical: (a: number) => number;
    readonly patchbuilder_reset: (a: number) => void;
    readonly __wbg_patchapplier_free: (a: number, b: number) => void;
    readonly patchapplier_new: () => number;
    readonly patchapplier_add_source_chunk: (a: number, b: number, c: number) => void;
    readonly patchapplier_set_patch: (a: number, b: number, c: number) => void;
    readonly patchapplier_source_size: (a: number) => number;
    readonly patchapplier_is_patch_loaded: (a: number) => number;
    readonly patchapplier_progress: (a: number, b: number) => number;
    readonly patchapplier_validate_source: (a: number) => [number, number, number];
    readonly patchapplier_expected_source_size: (a: number) => [bigint, number, number];
    readonly patchapplier_expected_target_size: (a: number) => [bigint, number, number];
    readonly patchapplier_get_patch_info: (a: number) => [number, number, number, number];
    readonly patchapplier_prepare: (a: number) => [number, number];
    readonly patchapplier_has_more_output: (a: number) => number;
    readonly patchapplier_output_progress: (a: number) => number;
    readonly patchapplier_next_output_chunk: (a: number, b: number) => [number, number];
    readonly patchapplier_total_output_size: (a: number) => number;
    readonly patchapplier_remaining_output_size: (a: number) => number;
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
