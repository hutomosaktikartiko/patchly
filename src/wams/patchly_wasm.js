/* @ts-self-types="./patchly_wasm.d.ts" */

/**
 * Applier for pacthes with streaming output supoort.
 */
export class PatchApplier {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PatchApplierFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_patchapplier_free(ptr, 0);
    }
    /**
     * Add a chunk of source (old file) data.
     * @param {Uint8Array} chunk
     */
    add_source_chunk(chunk) {
        const ptr0 = passArray8ToWasm0(chunk, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.patchapplier_add_source_chunk(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Get expected source size from patch metadata.
     * @returns {bigint}
     */
    expected_source_size() {
        const ret = wasm.patchapplier_expected_source_size(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Get expected target size from patch metadata.
     * @returns {bigint}
     */
    expected_target_size() {
        const ret = wasm.patchapplier_expected_target_size(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return BigInt.asUintN(64, ret[0]);
    }
    /**
     * Get patch info as JSON string.
     * @returns {string}
     */
    get_patch_info() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.patchapplier_get_patch_info(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Check if there's more output to read
     * @returns {boolean}
     */
    has_more_output() {
        const ret = wasm.patchapplier_has_more_output(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Check if patch has been loaded.
     * @returns {boolean}
     */
    is_patch_loaded() {
        const ret = wasm.patchapplier_is_patch_loaded(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Create a new PatchApplier
     */
    constructor() {
        const ret = wasm.patchapplier_new();
        this.__wbg_ptr = ret >>> 0;
        PatchApplierFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get next chunk of output for streaming to OPFS.
     * @param {number} max_chunk_size
     * @returns {Uint8Array}
     */
    next_output_chunk(max_chunk_size) {
        const ret = wasm.patchapplier_next_output_chunk(this.__wbg_ptr, max_chunk_size);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Get output progress as percentage (0-100).
     * @returns {number}
     */
    output_progress() {
        const ret = wasm.patchapplier_output_progress(this.__wbg_ptr);
        return ret;
    }
    /**
     * Prepare for streaming output.
     */
    prepare() {
        const ret = wasm.patchapplier_prepare(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get progress as percentage (0-100)
     * @param {number} source_expected
     * @returns {number}
     */
    progress(source_expected) {
        const ret = wasm.patchapplier_progress(this.__wbg_ptr, source_expected);
        return ret;
    }
    /**
     * Get remaining bytes to output
     * @returns {number}
     */
    remaining_output_size() {
        const ret = wasm.patchapplier_remaining_output_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Reset the applier for reuse.
     */
    reset() {
        wasm.patchapplier_reset(this.__wbg_ptr);
    }
    /**
     * Set the patch data.
     * @param {Uint8Array} patch_data
     */
    set_patch(patch_data) {
        const ptr0 = passArray8ToWasm0(patch_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.patchapplier_set_patch(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Get current source size.
     * @returns {number}
     */
    source_size() {
        const ret = wasm.patchapplier_source_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get total output size
     * @returns {number}
     */
    total_output_size() {
        const ret = wasm.patchapplier_total_output_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Validate source file before applying.
     * @returns {boolean}
     */
    validate_source() {
        const ret = wasm.patchapplier_validate_source(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
}
if (Symbol.dispose) PatchApplier.prototype[Symbol.dispose] = PatchApplier.prototype.free;

/**
 * Builder for creating patches from streamed file chunks.
 */
export class PatchBuilder {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PatchBuilder.prototype);
        obj.__wbg_ptr = ptr;
        PatchBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
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
     * Add a chunk of source (old file) data.
     * @param {Uint8Array} chunk
     */
    add_source_chunk(chunk) {
        const ptr0 = passArray8ToWasm0(chunk, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.patchbuilder_add_source_chunk(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Add a chunk of target (new file) data.
     * @param {Uint8Array} chunk
     */
    add_target_chunk(chunk) {
        const ptr0 = passArray8ToWasm0(chunk, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.patchbuilder_add_target_chunk(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Check if source and target files are indentical.
     * Files are identical if both size AND hash match
     * @returns {boolean}
     */
    are_files_identical() {
        const ret = wasm.patchbuilder_are_files_identical(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Finalize and generate the patch.
     * returns serialized patch data.
     * @returns {Uint8Array}
     */
    finalize() {
        const ret = wasm.patchbuilder_finalize(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Get patch info without without full serialization (for perview).
     * Returns JSON string with stats
     * @returns {string}
     */
    get_preview_info() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.patchbuilder_get_preview_info(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Create a new PatchBuilder with default chunk size
     */
    constructor() {
        const ret = wasm.patchbuilder_new();
        this.__wbg_ptr = ret >>> 0;
        PatchBuilderFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get progress as percentage (0-100) based on expected sizes.
     * Returns source progress if target_expected is 0.
     * @param {number} source_expected
     * @param {number} target_expected
     * @returns {number}
     */
    progress(source_expected, target_expected) {
        const ret = wasm.patchbuilder_progress(this.__wbg_ptr, source_expected, target_expected);
        return ret;
    }
    /**
     * Reset the builder for reuse.
     */
    reset() {
        wasm.patchbuilder_reset(this.__wbg_ptr);
    }
    /**
     * Get current source size (bytes received so far).
     * @returns {number}
     */
    source_size() {
        const ret = wasm.patchbuilder_source_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get current target size (bytes received so far).
     * @returns {number}
     */
    target_size() {
        const ret = wasm.patchbuilder_target_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Create a new PatchBuilder with custom chunk size.
     * @param {number} chunk_size
     * @returns {PatchBuilder}
     */
    static with_chunk_size(chunk_size) {
        const ret = wasm.patchbuilder_with_chunk_size(chunk_size);
        return PatchBuilder.__wrap(ret);
    }
}
if (Symbol.dispose) PatchBuilder.prototype[Symbol.dispose] = PatchBuilder.prototype.free;

/**
 * Calculate hash of data
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
 * Get the library version
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

const PatchApplierFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_patchapplier_free(ptr >>> 0, 1));
const PatchBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_patchbuilder_free(ptr >>> 0, 1));

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
