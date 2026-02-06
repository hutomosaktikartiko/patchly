/* tslint:disable */
/* eslint-disable */

/**
 * Streaming binary patch builder.
 *
 * Processes source and target files in chunks to generate a binary patch.
 * Designed for memory-efficient handling of large files (multi-GB).
 */
export class PatchBuilder {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Adds a chunk of source (old file) data.
     */
    add_source_chunk(chunk: Uint8Array): void;
    /**
     * Adds a chunk of target (new file) data.
     *
     * Generates patch output immediately; call `flush_output()` to retrieve it.
     */
    add_target_chunk(chunk: Uint8Array): void;
    /**
     * Checks if source and target files are identical.
     *
     * Only accurate after all data has been processed.
     */
    are_files_identical(): boolean;
    /**
     * Finalizes source processing.
     */
    finalize_source(): void;
    /**
     * Finalizes target processing.
     *
     * Call this after all target chunks have been added.
     */
    finalize_target(): void;
    /**
     * Returns the next chunk of patch output.
     *
     * Returns serialized patch data ready to write to file.
     */
    flush_output(max_size: number): Uint8Array;
    /**
     * Checks if there's patch output available to read.
     */
    has_output(): boolean;
    /**
     * Creates a new `PatchBuilder` with default chunk size.
     */
    constructor();
    /**
     * Returns the approximate pending output size.
     */
    pending_output_size(): number;
    /**
     * Resets the builder for reuse.
     */
    reset(): void;
    /**
     * Sets the expected total target size.
     *
     * Must be called before `add_target_chunk()` for proper header generation.
     */
    set_target_size(size: bigint): void;
    /**
     * Returns the current source size in bytes.
     */
    source_size(): number;
    /**
     * Returns the current target size in bytes.
     */
    target_size(): number;
}

/**
 * WASM-bindable streaming hash builder.
 *
 * Use this to calculate hash incrementally from JavaScript without BigInt allocations.
 */
export class StreamingHasher {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Finalizes and returns the hash as a hex string.
     */
    finalize(): string;
    /**
     * Finalizes and returns the hash as a u64 for comparison.
     */
    finalize_u64(): bigint;
    /**
     * Creates a new hash builder.
     */
    constructor();
    /**
     * Updates the hash with a chunk of data.
     */
    update(data: Uint8Array): void;
}

/**
 * Calculates hash of data and returns it as a hex string.
 */
export function hash_data(data: Uint8Array): string;

/**
 * Parses only the patch header (33 bytes) without parsing instructions.
 *
 * Returns JSON with sourceSize, sourceHash, targetSize, chunkSize, and headerSize.
 * TypeScript will parse instructions directly from OPFS to avoid loading entire patch.
 */
export function parse_patch_header_only(header_data: Uint8Array): string;

/**
 * Returns the library version.
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
    readonly parse_patch_header_only: (a: number, b: number) => [number, number, number, number];
    readonly version: () => [number, number];
    readonly __wbg_streaminghasher_free: (a: number, b: number) => void;
    readonly streaminghasher_new: () => number;
    readonly streaminghasher_update: (a: number, b: number, c: number) => void;
    readonly streaminghasher_finalize: (a: number) => [number, number];
    readonly streaminghasher_finalize_u64: (a: number) => bigint;
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
