
let wasm;

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}
/**
*/
export class ImageApproximation {

    static __wrap(ptr) {
        const obj = Object.create(ImageApproximation.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_imageapproximation_free(ptr);
    }
    /**
    * @returns {number}
    */
    get max_iter() {
        var ret = wasm.__wbg_get_imageapproximation_max_iter(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set max_iter(arg0) {
        wasm.__wbg_set_imageapproximation_max_iter(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get current_iter() {
        var ret = wasm.__wbg_get_imageapproximation_current_iter(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set current_iter(arg0) {
        wasm.__wbg_set_imageapproximation_current_iter(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get error() {
        var ret = wasm.__wbg_get_imageapproximation_error(this.ptr);
        return ret;
    }
    /**
    * @param {number} arg0
    */
    set error(arg0) {
        wasm.__wbg_set_imageapproximation_error(this.ptr, arg0);
    }
    /**
    * @param {RGBAImage} im
    * @param {number} max_iter
    * @returns {ImageApproximation}
    */
    static constructor(im, max_iter) {
        _assertClass(im, RGBAImage);
        var ptr0 = im.ptr;
        im.ptr = 0;
        var ret = wasm.imageapproximation_constructor(ptr0, max_iter);
        return ImageApproximation.__wrap(ret);
    }
    /**
    * @returns {number}
    */
    im_result_data_as_pointer() {
        var ret = wasm.imageapproximation_im_result_data_as_pointer(this.ptr);
        return ret;
    }
    /**
    * @returns {boolean}
    */
    next() {
        var ret = wasm.imageapproximation_next(this.ptr);
        return ret !== 0;
    }
}
/**
*/
export class RGBAImage {

    static __wrap(ptr) {
        const obj = Object.create(RGBAImage.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_rgbaimage_free(ptr);
    }
    /**
    * @param {Uint8Array} data
    * @param {number} width
    * @param {number} height
    * @returns {RGBAImage}
    */
    static constructor(data, width, height) {
        var ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.rgbaimage_constructor(ptr0, len0, width, height);
        return RGBAImage.__wrap(ret);
    }
    /**
    * @returns {number}
    */
    data_as_pointer() {
        var ret = wasm.rgbaimage_data_as_pointer(this.ptr);
        return ret;
    }
    /**
    * @returns {number}
    */
    width() {
        var ret = wasm.rgbaimage_width(this.ptr);
        return ret >>> 0;
    }
    /**
    * @returns {number}
    */
    height() {
        var ret = wasm.rgbaimage_height(this.ptr);
        return ret >>> 0;
    }
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
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
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;

    return wasm;
}

export default init;

