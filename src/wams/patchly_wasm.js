/* @ts-self-types="./patchly_wasm.d.ts" */

/**
 * Streaming binary patch builder.
 *
 * Processes source and target files in chunks to generate a binary patch.
 * Designed for memory-efficient handling of large files (multi-GB).
 */
export class PatchBuilder {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PatchBuilderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_patchbuilder_free(ptr, 0);
    }
    /**
     * Adds a chunk of source (old file) data.
     * @param {Uint8Array} chunk
     */
    add_source_chunk(chunk) {
        const ptr0 = passArray8ToWasm0(chunk, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.patchbuilder_add_source_chunk(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Adds a chunk of target (new file) data.
     *
     * Generates patch output immediately; call `flush_output()` to retrieve it.
     * @param {Uint8Array} chunk
     */
    add_target_chunk(chunk) {
        const ptr0 = passArray8ToWasm0(chunk, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.patchbuilder_add_target_chunk(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Checks if source and target files are identical.
     *
     * Only accurate after all data has been processed.
     * @returns {boolean}
     */
    are_files_identical() {
        const ret = wasm.patchbuilder_are_files_identical(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Finalizes source processing.
     */
    finalize_source() {
        wasm.patchbuilder_finalize_source(this.__wbg_ptr);
    }
    /**
     * Finalizes target processing.
     *
     * Call this after all target chunks have been added.
     */
    finalize_target() {
        wasm.patchbuilder_finalize_target(this.__wbg_ptr);
    }
    /**
     * Returns the next chunk of patch output.
     *
     * Returns serialized patch data ready to write to file.
     * @param {number} max_size
     * @returns {Uint8Array}
     */
    flush_output(max_size) {
        const ret = wasm.patchbuilder_flush_output(this.__wbg_ptr, max_size);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Checks if there's patch output available to read.
     * @returns {boolean}
     */
    has_output() {
        const ret = wasm.patchbuilder_has_output(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Creates a new `PatchBuilder` with default chunk size.
     */
    constructor() {
        const ret = wasm.patchbuilder_new();
        this.__wbg_ptr = ret >>> 0;
        PatchBuilderFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Returns the approximate pending output size.
     * @returns {number}
     */
    pending_output_size() {
        const ret = wasm.patchbuilder_pending_output_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Resets the builder for reuse.
     */
    reset() {
        wasm.patchbuilder_reset(this.__wbg_ptr);
    }
    /**
     * Sets the expected total target size.
     *
     * Must be called before `add_target_chunk()` for proper header generation.
     * @param {bigint} size
     */
    set_target_size(size) {
        wasm.patchbuilder_set_target_size(this.__wbg_ptr, size);
    }
    /**
     * Returns the current source size in bytes.
     * @returns {number}
     */
    source_size() {
        const ret = wasm.patchbuilder_source_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the current target size in bytes.
     * @returns {number}
     */
    target_size() {
        const ret = wasm.patchbuilder_target_size(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) PatchBuilder.prototype[Symbol.dispose] = PatchBuilder.prototype.free;

/**
 * WASM-bindable streaming hash builder.
 *
 * Use this to calculate hash incrementally from JavaScript without BigInt allocations.
 */
export class StreamingHasher {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StreamingHasherFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_streaminghasher_free(ptr, 0);
    }
    /**
     * Finalizes and returns the hash as a hex string.
     * @returns {string}
     */
    finalize() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.streaminghasher_finalize(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Finalizes and returns the hash as a u64 for comparison.
     * @returns {bigint}
     */
    finalize_u64() {
        const ret = wasm.streaminghasher_finalize_u64(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Creates a new hash builder.
     */
    constructor() {
        const ret = wasm.streaminghasher_new();
        this.__wbg_ptr = ret >>> 0;
        StreamingHasherFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Updates the hash with a chunk of data.
     * @param {Uint8Array} data
     */
    update(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.streaminghasher_update(this.__wbg_ptr, ptr0, len0);
    }
}
if (Symbol.dispose) StreamingHasher.prototype[Symbol.dispose] = StreamingHasher.prototype.free;

/**
 * Calculates hash of data and returns it as a hex string.
 * @param {Uint8Array} data
 * @returns {string}
 */
export function hash_data(data) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.hash_data(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * Parses only the patch header (33 bytes) without parsing instructions.
 *
 * Returns JSON with sourceSize, sourceHash, targetSize, chunkSize, and headerSize.
 * TypeScript will parse instructions directly from OPFS to avoid loading entire patch.
 * @param {Uint8Array} header_data
 * @returns {string}
 */
export function parse_patch_header_only(header_data) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passArray8ToWasm0(header_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.parse_patch_header_only(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Returns the library version.
 * @returns {string}
 */
export function version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./patchly_wasm_bg.js": import0,
    };
}

const PatchBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_patchbuilder_free(ptr >>> 0, 1));
const StreamingHasherFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_streaminghasher_free(ptr >>> 0, 1));

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('patchly_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
