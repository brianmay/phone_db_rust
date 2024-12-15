import { getMetaContents } from './snippets/dioxus-cli-config-4e61e40bcf6c18f8/inline0.js';
import { RawInterpreter } from './snippets/dioxus-interpreter-js-161bb2cc5b21a4da/inline0.js';
import { setAttributeInner } from './snippets/dioxus-interpreter-js-161bb2cc5b21a4da/src/js/common.js';
import { get_select_data } from './snippets/dioxus-web-ac47560c608d507f/inline0.js';
import { WebDioxusChannel } from './snippets/dioxus-web-ac47560c608d507f/src/js/eval.js';

let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}


let WASM_VECTOR_LEN = 0;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_4.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addToExternrefTable0(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_export_4.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_7.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_7.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}
function __wbg_adapter_50(arg0, arg1, arg2) {
    wasm.closure410_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_53(arg0, arg1, arg2) {
    wasm.closure416_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_56(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__he97b158fd99f2633(arg0, arg1);
}

function __wbg_adapter_59(arg0, arg1, arg2) {
    wasm.closure414_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_62(arg0, arg1, arg2) {
    wasm.closure412_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_65(arg0, arg1, arg2) {
    wasm.closure500_externref_shim(arg0, arg1, arg2);
}

const __wbindgen_enum_ScrollBehavior = ["auto", "instant", "smooth"];

const __wbindgen_enum_ScrollRestoration = ["auto", "manual"];

const JSOwnerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsowner_free(ptr >>> 0, 1));

export class JSOwner {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(JSOwner.prototype);
        obj.__wbg_ptr = ptr;
        JSOwnerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JSOwnerFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsowner_free(ptr, 0);
    }
}

export function __wbg_String_eecc4a11987127d6(arg0, arg1) {
    const ret = String(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_addEventListener_b9481c2c2cab6047() { return handleError(function (arg0, arg1, arg2, arg3) {
    arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
}, arguments) };

export function __wbg_altKey_d5409f5ddaa29593(arg0) {
    const ret = arg0.altKey;
    return ret;
};

export function __wbg_altKey_d54599b3b6b6cf22(arg0) {
    const ret = arg0.altKey;
    return ret;
};

export function __wbg_altKey_d751fb926977ab64(arg0) {
    const ret = arg0.altKey;
    return ret;
};

export function __wbg_animationName_f1af247de3572fef(arg0, arg1) {
    const ret = arg1.animationName;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_appendChild_d22bc7af6b96b3f1() { return handleError(function (arg0, arg1) {
    const ret = arg0.appendChild(arg1);
    return ret;
}, arguments) };

export function __wbg_back_ab128eec71a6f563() { return handleError(function (arg0) {
    arg0.back();
}, arguments) };

export function __wbg_blockSize_6464e214800294a9(arg0) {
    const ret = arg0.blockSize;
    return ret;
};

export function __wbg_blur_51f415004ecbe327() { return handleError(function (arg0) {
    arg0.blur();
}, arguments) };

export function __wbg_borderBoxSize_d91515e84a720174(arg0) {
    const ret = arg0.borderBoxSize;
    return ret;
};

export function __wbg_boundingClientRect_c15b43e6cd42d56f(arg0) {
    const ret = arg0.boundingClientRect;
    return ret;
};

export function __wbg_bubbles_48182817f8ec169f(arg0) {
    const ret = arg0.bubbles;
    return ret;
};

export function __wbg_buffer_61b7ce01341d7f88(arg0) {
    const ret = arg0.buffer;
    return ret;
};

export function __wbg_button_12b22015f2d5993d(arg0) {
    const ret = arg0.button;
    return ret;
};

export function __wbg_buttons_e83cec0abc6f937f(arg0) {
    const ret = arg0.buttons;
    return ret;
};

export function __wbg_call_500db948e69c7330() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

export function __wbg_call_b0d8e36992d9900d() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

export function __wbg_changedTouches_86448a1d3a872098(arg0) {
    const ret = arg0.changedTouches;
    return ret;
};

export function __wbg_charCodeAt_f90f5a110314c4fb(arg0, arg1) {
    const ret = arg0.charCodeAt(arg1 >>> 0);
    return ret;
};

export function __wbg_checked_fc3b0aba823c9a35(arg0) {
    const ret = arg0.checked;
    return ret;
};

export function __wbg_clientX_18c5fbacc6398ad8(arg0) {
    const ret = arg0.clientX;
    return ret;
};

export function __wbg_clientX_f73b86b8aba3591d(arg0) {
    const ret = arg0.clientX;
    return ret;
};

export function __wbg_clientY_0974153484cf0d09(arg0) {
    const ret = arg0.clientY;
    return ret;
};

export function __wbg_clientY_af033356579f2b9c(arg0) {
    const ret = arg0.clientY;
    return ret;
};

export function __wbg_code_878e1961e18ba92f(arg0, arg1) {
    const ret = arg1.code;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_code_cd82312abb9d9ff9(arg0) {
    const ret = arg0.code;
    return ret;
};

export function __wbg_contentBoxSize_c8dcd6b272f821ba(arg0) {
    const ret = arg0.contentBoxSize;
    return ret;
};

export function __wbg_createComment_8bc5f42232aeee70(arg0, arg1, arg2) {
    const ret = arg0.createComment(getStringFromWasm0(arg1, arg2));
    return ret;
};

export function __wbg_createElementNS_494cc14f5fdee138() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    const ret = arg0.createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    return ret;
}, arguments) };

export function __wbg_createElement_89923fcb809656b7() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
    return ret;
}, arguments) };

export function __wbg_createTextNode_457c122eb9cb5753(arg0, arg1, arg2) {
    const ret = arg0.createTextNode(getStringFromWasm0(arg1, arg2));
    return ret;
};

export function __wbg_ctrlKey_2d0caa58efbf4a14(arg0) {
    const ret = arg0.ctrlKey;
    return ret;
};

export function __wbg_ctrlKey_5a324c8556fbce1c(arg0) {
    const ret = arg0.ctrlKey;
    return ret;
};

export function __wbg_ctrlKey_5c308955b0d5492d(arg0) {
    const ret = arg0.ctrlKey;
    return ret;
};

export function __wbg_dataTransfer_c29d7d69c9576def(arg0) {
    const ret = arg0.dataTransfer;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_data_4ce8a82394d8b110(arg0) {
    const ret = arg0.data;
    return ret;
};

export function __wbg_data_8980cafa6731c6b5(arg0, arg1) {
    const ret = arg1.data;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_deltaMode_b2e9bb0dca5cf196(arg0) {
    const ret = arg0.deltaMode;
    return ret;
};

export function __wbg_deltaX_5c26d3b55d406732(arg0) {
    const ret = arg0.deltaX;
    return ret;
};

export function __wbg_deltaY_1683a859ce933add(arg0) {
    const ret = arg0.deltaY;
    return ret;
};

export function __wbg_deltaZ_6f5c87687327f34c(arg0) {
    const ret = arg0.deltaZ;
    return ret;
};

export function __wbg_detail_299f8f0e25ca09c5(arg0) {
    const ret = arg0.detail;
    return ret;
};

export function __wbg_document_f11bc4f7c03e1745(arg0) {
    const ret = arg0.document;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_done_f22c1561fa919baa(arg0) {
    const ret = arg0.done;
    return ret;
};

export function __wbg_elapsedTime_4ec00766814845a1(arg0) {
    const ret = arg0.elapsedTime;
    return ret;
};

export function __wbg_elapsedTime_6fb52e82a445ff1c(arg0) {
    const ret = arg0.elapsedTime;
    return ret;
};

export function __wbg_entries_4f2bb9b0d701c0f6(arg0) {
    const ret = Object.entries(arg0);
    return ret;
};

export function __wbg_entries_82bf0e755ef54a5f(arg0) {
    const ret = arg0.entries();
    return ret;
};

export function __wbg_error_483d659117b6f3f6(arg0, arg1, arg2, arg3) {
    console.error(arg0, arg1, arg2, arg3);
};

export function __wbg_error_7534b8e9a36f1ab4(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_error_bc396fc38839dd25(arg0, arg1) {
    console.error(arg0, arg1);
};

export function __wbg_error_fab41a42d22bf2bc(arg0) {
    console.error(arg0);
};

export function __wbg_files_576e546a364f9971(arg0) {
    const ret = arg0.files;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_files_95d9491da88a54b5(arg0) {
    const ret = arg0.files;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_focus_35fe945f7268dd62() { return handleError(function (arg0) {
    arg0.focus();
}, arguments) };

export function __wbg_force_82b5a32305812290(arg0) {
    const ret = arg0.force;
    return ret;
};

export function __wbg_forward_d4de62a4496ff3a2() { return handleError(function (arg0) {
    arg0.forward();
}, arguments) };

export function __wbg_getAttribute_3104455bb78f9b7b(arg0, arg1, arg2, arg3) {
    const ret = arg1.getAttribute(getStringFromWasm0(arg2, arg3));
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_getBoundingClientRect_05c4b9e3701bb372(arg0) {
    const ret = arg0.getBoundingClientRect();
    return ret;
};

export function __wbg_getElementById_dcc9f1f3cfdca0bc(arg0, arg1, arg2) {
    const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_getMetaContents_4d492f40d14ba619(arg0, arg1, arg2) {
    const ret = getMetaContents(getStringFromWasm0(arg1, arg2));
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_getNode_3019c6fd0554479b(arg0, arg1) {
    const ret = arg0.getNode(arg1 >>> 0);
    return ret;
};

export function __wbg_get_9901e5f7f90821fc(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_get_9aa3dff3f0266054(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return ret;
};

export function __wbg_get_bbccf8970793c087() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

export function __wbg_get_dfac72a5ffb577cc(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_getselectdata_750c8ebf72af63ae(arg0, arg1) {
    const ret = get_select_data(arg1);
    const ptr1 = passArrayJsValueToWasm0(ret, wasm.__wbindgen_malloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_hash_4227a319264c4ca1() { return handleError(function (arg0, arg1) {
    const ret = arg1.hash;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}, arguments) };

export function __wbg_height_08fd44318e18021d(arg0) {
    const ret = arg0.height;
    return ret;
};

export function __wbg_height_1cf1d938bdabff2a(arg0) {
    const ret = arg0.height;
    return ret;
};

export function __wbg_height_854c8d8584c709bc(arg0) {
    const ret = arg0.height;
    return ret;
};

export function __wbg_history_d719742cb5c67d99() { return handleError(function (arg0) {
    const ret = arg0.history;
    return ret;
}, arguments) };

export function __wbg_host_7131cd3aac9f8fd5() { return handleError(function (arg0, arg1) {
    const ret = arg1.host;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}, arguments) };

export function __wbg_identifier_14d5888db18610bb(arg0) {
    const ret = arg0.identifier;
    return ret;
};

export function __wbg_initialize_4d57f3e1cdf6f610(arg0, arg1, arg2) {
    arg0.initialize(arg1, arg2);
};

export function __wbg_inlineSize_60da5bea0a6275d2(arg0) {
    const ret = arg0.inlineSize;
    return ret;
};

export function __wbg_instanceof_ArrayBuffer_670ddde44cdb2602(arg0) {
    let result;
    try {
        result = arg0 instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_DragEvent_7074910fe5098d19(arg0) {
    let result;
    try {
        result = arg0 instanceof DragEvent;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_Element_0f1680908791f190(arg0) {
    let result;
    try {
        result = arg0 instanceof Element;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_HtmlElement_d94ed69c6883a691(arg0) {
    let result;
    try {
        result = arg0 instanceof HTMLElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_HtmlFormElement_6ccf78851bb8bb14(arg0) {
    let result;
    try {
        result = arg0 instanceof HTMLFormElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_HtmlInputElement_47b3e827f364773c(arg0) {
    let result;
    try {
        result = arg0 instanceof HTMLInputElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_HtmlSelectElement_3b6cae61035e1814(arg0) {
    let result;
    try {
        result = arg0 instanceof HTMLSelectElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_HtmlTextAreaElement_88347fc269bfb466(arg0) {
    let result;
    try {
        result = arg0 instanceof HTMLTextAreaElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_Node_7d77fe8c0da04c3a(arg0) {
    let result;
    try {
        result = arg0 instanceof Node;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_Uint8Array_28af5bc19d6acad8(arg0) {
    let result;
    try {
        result = arg0 instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_Window_d2514c6a7ee7ba60(arg0) {
    let result;
    try {
        result = arg0 instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_intersectionRatio_0473b7f3b4ca36eb(arg0) {
    const ret = arg0.intersectionRatio;
    return ret;
};

export function __wbg_intersectionRect_71f1c03fae274d24(arg0) {
    const ret = arg0.intersectionRect;
    return ret;
};

export function __wbg_isArray_1ba11a930108ec51(arg0) {
    const ret = Array.isArray(arg0);
    return ret;
};

export function __wbg_isComposing_34930e03980aa623(arg0) {
    const ret = arg0.isComposing;
    return ret;
};

export function __wbg_isIntersecting_03f2dfd4beb70720(arg0) {
    const ret = arg0.isIntersecting;
    return ret;
};

export function __wbg_isPrimary_d9a6de1204821aae(arg0) {
    const ret = arg0.isPrimary;
    return ret;
};

export function __wbg_isSafeInteger_12f5549b2fca23f4(arg0) {
    const ret = Number.isSafeInteger(arg0);
    return ret;
};

export function __wbg_item_8352cc6c60828e35(arg0, arg1) {
    const ret = arg0.item(arg1 >>> 0);
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_iterator_23604bb983791576() {
    const ret = Symbol.iterator;
    return ret;
};

export function __wbg_key_9a40d4f6defa675b(arg0, arg1) {
    const ret = arg1.key;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_left_d79d7167a89a5169(arg0) {
    const ret = arg0.left;
    return ret;
};

export function __wbg_length_15aa023b16db5413(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_length_65d1cd11729ced11(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_length_86e2f100fef1fecc(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_length_b4ca75fbd53c74dc(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_length_c4528fc455e58194(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_length_d65cf0786bfc5739(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_location_0d3ce589878cba8a(arg0) {
    const ret = arg0.location;
    return ret;
};

export function __wbg_location_b2ec7e36fec8a8ff(arg0) {
    const ret = arg0.location;
    return ret;
};

export function __wbg_log_0cc1b7768397bcfe(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_log_cb9e190acc5753fb(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.log(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_mark_7438147ce31e9d4b(arg0, arg1) {
    performance.mark(getStringFromWasm0(arg0, arg1));
};

export function __wbg_measure_fb7825c11612c823() { return handleError(function (arg0, arg1, arg2, arg3) {
    let deferred0_0;
    let deferred0_1;
    let deferred1_0;
    let deferred1_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        deferred1_0 = arg2;
        deferred1_1 = arg3;
        performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}, arguments) };

export function __wbg_metaKey_90fbd812345a7e0c(arg0) {
    const ret = arg0.metaKey;
    return ret;
};

export function __wbg_metaKey_a8404f6c100cb890(arg0) {
    const ret = arg0.metaKey;
    return ret;
};

export function __wbg_metaKey_de1f08a4d1e84bd1(arg0) {
    const ret = arg0.metaKey;
    return ret;
};

export function __wbg_name_37e12d7b980bc5bd(arg0, arg1) {
    const ret = arg1.name;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_new_254fa9eac11932ae() {
    const ret = new Array();
    return ret;
};

export function __wbg_new_3ff5b33b1ce712df(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
};

export function __wbg_new_688846f374351c92() {
    const ret = new Object();
    return ret;
};

export function __wbg_new_8a6f238a6ece86ea() {
    const ret = new Error();
    return ret;
};

export function __wbg_new_9b6c38191d7b9512() { return handleError(function (arg0, arg1) {
    const ret = new WebSocket(getStringFromWasm0(arg0, arg1));
    return ret;
}, arguments) };

export function __wbg_new_a01d9d610b795c1f() { return handleError(function () {
    const ret = new FileReader();
    return ret;
}, arguments) };

export function __wbg_new_bc96c6a1c0786643() {
    const ret = new Map();
    return ret;
};

export function __wbg_new_bdc34f8e8fb8b1e4(arg0) {
    const ret = new WebDioxusChannel(JSOwner.__wrap(arg0));
    return ret;
};

export function __wbg_new_f8510b5dc8ad6168(arg0) {
    const ret = new RawInterpreter(arg0 >>> 0);
    return ret;
};

export function __wbg_newnoargs_fd9e4bf8be2bc16d(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return ret;
};

export function __wbg_newwithargs_308e846d06f33aa0(arg0, arg1, arg2, arg3) {
    const ret = new Function(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
    return ret;
};

export function __wbg_next_01dd9234a5bf6d05() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments) };

export function __wbg_next_137428deb98342b0(arg0) {
    const ret = arg0.next;
    return ret;
};

export function __wbg_offsetX_2873be6a91890178(arg0) {
    const ret = arg0.offsetX;
    return ret;
};

export function __wbg_offsetY_04fbbed1bfcc85d1(arg0) {
    const ret = arg0.offsetY;
    return ret;
};

export function __wbg_ownerDocument_276071c53467c221(arg0) {
    const ret = arg0.ownerDocument;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_pageX_6d13bb07824f4544(arg0) {
    const ret = arg0.pageX;
    return ret;
};

export function __wbg_pageX_df74c43e31bf52e8(arg0) {
    const ret = arg0.pageX;
    return ret;
};

export function __wbg_pageY_5e7dfef64bdf2be8(arg0) {
    const ret = arg0.pageY;
    return ret;
};

export function __wbg_pageY_de9decbc3b027dd7(arg0) {
    const ret = arg0.pageY;
    return ret;
};

export function __wbg_parentElement_41c6f5f746ea74cf(arg0) {
    const ret = arg0.parentElement;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_pathname_97830455d4d265a8() { return handleError(function (arg0, arg1) {
    const ret = arg1.pathname;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}, arguments) };

export function __wbg_pointerId_85845d98372f1198(arg0) {
    const ret = arg0.pointerId;
    return ret;
};

export function __wbg_pointerType_4d6a147d076e7aae(arg0, arg1) {
    const ret = arg1.pointerType;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_pressure_c345c07c94ad38cf(arg0) {
    const ret = arg0.pressure;
    return ret;
};

export function __wbg_preventDefault_3c86e59772d015e6(arg0) {
    arg0.preventDefault();
};

export function __wbg_propertyName_a6c9b5e3273c260e(arg0, arg1) {
    const ret = arg1.propertyName;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_protocol_ceaedd334dc7dbaf() { return handleError(function (arg0, arg1) {
    const ret = arg1.protocol;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}, arguments) };

export function __wbg_pseudoElement_6d8c98bb69b10fd8(arg0, arg1) {
    const ret = arg1.pseudoElement;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_pseudoElement_f13d2cee967495a5(arg0, arg1) {
    const ret = arg1.pseudoElement;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_pushState_242f61fdcf188197() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    arg0.pushState(arg1, getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
}, arguments) };

export function __wbg_push_6edad0df4b546b2c(arg0, arg1) {
    const ret = arg0.push(arg1);
    return ret;
};

export function __wbg_querySelectorAll_2d037c571f099149() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
    return ret;
}, arguments) };

export function __wbg_queueMicrotask_2181040e064c0dc8(arg0) {
    queueMicrotask(arg0);
};

export function __wbg_queueMicrotask_ef9ac43769cbcc4f(arg0) {
    const ret = arg0.queueMicrotask;
    return ret;
};

export function __wbg_radiusX_22e15bbbc3829a00(arg0) {
    const ret = arg0.radiusX;
    return ret;
};

export function __wbg_radiusY_c559aca86d9063da(arg0) {
    const ret = arg0.radiusY;
    return ret;
};

export function __wbg_random_a435d21390634bdf() {
    const ret = Math.random();
    return ret;
};

export function __wbg_readAsArrayBuffer_db7f197b5b6b34cf() { return handleError(function (arg0, arg1) {
    arg0.readAsArrayBuffer(arg1);
}, arguments) };

export function __wbg_readAsText_9cd56925e58e0eab() { return handleError(function (arg0, arg1) {
    arg0.readAsText(arg1);
}, arguments) };

export function __wbg_reload_fc0bafbdf55c9e82() { return handleError(function (arg0) {
    arg0.reload();
}, arguments) };

export function __wbg_repeat_621b3806d8c52204(arg0) {
    const ret = arg0.repeat;
    return ret;
};

export function __wbg_replaceState_c99e45816817a26d() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    arg0.replaceState(arg1, getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
}, arguments) };

export function __wbg_requestAnimationFrame_169cbbda5861d9ca() { return handleError(function (arg0, arg1) {
    const ret = arg0.requestAnimationFrame(arg1);
    return ret;
}, arguments) };

export function __wbg_resolve_0bf7c44d641804f9(arg0) {
    const ret = Promise.resolve(arg0);
    return ret;
};

export function __wbg_result_b7f693658f393a91() { return handleError(function (arg0) {
    const ret = arg0.result;
    return ret;
}, arguments) };

export function __wbg_rootBounds_1d0268045373ce65(arg0) {
    const ret = arg0.rootBounds;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_rotationAngle_68fb2bcade37d390(arg0) {
    const ret = arg0.rotationAngle;
    return ret;
};

export function __wbg_run_039ff2fcfda5a2fd(arg0) {
    arg0.run();
};

export function __wbg_rustRecv_af81a17681ce3a96(arg0) {
    const ret = arg0.rustRecv();
    return ret;
};

export function __wbg_rustSend_d6c311e771149d6c(arg0, arg1) {
    arg0.rustSend(arg1);
};

export function __wbg_saveTemplate_1bff2747fd009e45(arg0, arg1, arg2, arg3) {
    var v0 = getArrayJsValueFromWasm0(arg1, arg2).slice();
    wasm.__wbindgen_free(arg1, arg2 * 4, 4);
    arg0.saveTemplate(v0, arg3);
};

export function __wbg_screenX_c56bfb67461b9296(arg0) {
    const ret = arg0.screenX;
    return ret;
};

export function __wbg_screenX_d9433abd043844ca(arg0) {
    const ret = arg0.screenX;
    return ret;
};

export function __wbg_screenY_41b8cda19b5f8f26(arg0) {
    const ret = arg0.screenY;
    return ret;
};

export function __wbg_screenY_f8a8d9911daacdfe(arg0) {
    const ret = arg0.screenY;
    return ret;
};

export function __wbg_scrollHeight_2a4648b5731c515c(arg0) {
    const ret = arg0.scrollHeight;
    return ret;
};

export function __wbg_scrollIntoView_ab0104255a2bac2b(arg0, arg1) {
    arg0.scrollIntoView(arg1);
};

export function __wbg_scrollLeft_f3a8f95470760df7(arg0) {
    const ret = arg0.scrollLeft;
    return ret;
};

export function __wbg_scrollTo_df6911be5d522b1f(arg0, arg1, arg2) {
    arg0.scrollTo(arg1, arg2);
};

export function __wbg_scrollTop_16b4c870cfff2996(arg0) {
    const ret = arg0.scrollTop;
    return ret;
};

export function __wbg_scrollWidth_8a6413fd3e55a5f2(arg0) {
    const ret = arg0.scrollWidth;
    return ret;
};

export function __wbg_scrollX_a7aa9e3d39d12f9f() { return handleError(function (arg0) {
    const ret = arg0.scrollX;
    return ret;
}, arguments) };

export function __wbg_scrollY_a855adb43646151e() { return handleError(function (arg0) {
    const ret = arg0.scrollY;
    return ret;
}, arguments) };

export function __wbg_search_feca3869d55ecd5c() { return handleError(function (arg0, arg1) {
    const ret = arg1.search;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}, arguments) };

export function __wbg_setAttributeInner_042ed9409bc5d08d(arg0, arg1, arg2, arg3, arg4, arg5) {
    setAttributeInner(arg0, getStringFromWasm0(arg1, arg2), arg3, arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
};

export function __wbg_setAttribute_148e0e65e20e5f27() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_setTimeout_8d2afdcdb34b4e5a() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.setTimeout(arg1, arg2);
    return ret;
}, arguments) };

export function __wbg_set_1d80752d0d5f0b21(arg0, arg1, arg2) {
    arg0[arg1 >>> 0] = arg2;
};

export function __wbg_set_23d69db4e5c66a6e(arg0, arg1, arg2) {
    arg0.set(arg1, arg2 >>> 0);
};

export function __wbg_set_3807d5f0bfc24aa7(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
};

export function __wbg_set_76818dc3c59a63d5(arg0, arg1, arg2) {
    const ret = arg0.set(arg1, arg2);
    return ret;
};

export function __wbg_setbehavior_e7cbaa29c624a8c5(arg0, arg1) {
    arg0.behavior = __wbindgen_enum_ScrollBehavior[arg1];
};

export function __wbg_sethref_959e239a845360e3() { return handleError(function (arg0, arg1, arg2) {
    arg0.href = getStringFromWasm0(arg1, arg2);
}, arguments) };

export function __wbg_setonclose_f9c609d8c9938fa5(arg0, arg1) {
    arg0.onclose = arg1;
};

export function __wbg_setonload_36cf7239551d2544(arg0, arg1) {
    arg0.onload = arg1;
};

export function __wbg_setonmessage_5e7ade2af360de9d(arg0, arg1) {
    arg0.onmessage = arg1;
};

export function __wbg_setonopen_54faa9e83483da1d(arg0, arg1) {
    arg0.onopen = arg1;
};

export function __wbg_setscrollRestoration_4a7c419d438ffcb7() { return handleError(function (arg0, arg1) {
    arg0.scrollRestoration = __wbindgen_enum_ScrollRestoration[arg1];
}, arguments) };

export function __wbg_shiftKey_0d6625838238aee8(arg0) {
    const ret = arg0.shiftKey;
    return ret;
};

export function __wbg_shiftKey_4b30f68655b97001(arg0) {
    const ret = arg0.shiftKey;
    return ret;
};

export function __wbg_shiftKey_64607b87e068c5fb(arg0) {
    const ret = arg0.shiftKey;
    return ret;
};

export function __wbg_size_5ead5cc358246113(arg0) {
    const ret = arg0.size;
    return ret;
};

export function __wbg_stack_0ed75d68575b0f3c(arg0, arg1) {
    const ret = arg1.stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_state_6de569648716f6fa() { return handleError(function (arg0) {
    const ret = arg0.state;
    return ret;
}, arguments) };

export function __wbg_static_accessor_GLOBAL_0be7472e492ad3e3() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_GLOBAL_THIS_1a6eb482d12c9bfb() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_SELF_1dc398a895c82351() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_WINDOW_ae1c80c7eea8d64a() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_stringify_f4f701bc34ceda61() { return handleError(function (arg0) {
    const ret = JSON.stringify(arg0);
    return ret;
}, arguments) };

export function __wbg_tangentialPressure_b1e2b9c3954f8c3b(arg0) {
    const ret = arg0.tangentialPressure;
    return ret;
};

export function __wbg_targetTouches_537b29eeaa72dc28(arg0) {
    const ret = arg0.targetTouches;
    return ret;
};

export function __wbg_target_a8fe593e7ee79c21(arg0) {
    const ret = arg0.target;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_textContent_593cb1d610df6a86(arg0, arg1) {
    const ret = arg1.textContent;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_then_0438fad860fe38e1(arg0, arg1) {
    const ret = arg0.then(arg1);
    return ret;
};

export function __wbg_then_0ffafeddf0e182a4(arg0, arg1, arg2) {
    const ret = arg0.then(arg1, arg2);
    return ret;
};

export function __wbg_tiltX_e10af55299c8ea3b(arg0) {
    const ret = arg0.tiltX;
    return ret;
};

export function __wbg_tiltY_d4653dc7c317ac5a(arg0) {
    const ret = arg0.tiltY;
    return ret;
};

export function __wbg_time_aef98df881293163(arg0) {
    const ret = arg0.time;
    return ret;
};

export function __wbg_top_640e0509d882f0ee(arg0) {
    const ret = arg0.top;
    return ret;
};

export function __wbg_touches_464d67ccc79e7632(arg0) {
    const ret = arg0.touches;
    return ret;
};

export function __wbg_twist_6436484255d17682(arg0) {
    const ret = arg0.twist;
    return ret;
};

export function __wbg_type_754e197ffe996ff1(arg0, arg1) {
    const ret = arg1.type;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_type_9d616bf03ad5a308(arg0, arg1) {
    const ret = arg1.type;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_updatememory_217ca207d383cda5(arg0, arg1) {
    arg0.update_memory(arg1);
};

export function __wbg_value_2adb5f0602e19ca9(arg0, arg1) {
    const ret = arg1.value;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_value_47fde8ea2d9fdcd5(arg0, arg1) {
    const ret = arg1.value;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_value_4c32fd138a88eee2(arg0) {
    const ret = arg0.value;
    return ret;
};

export function __wbg_value_a8b8b65bc31190d6(arg0, arg1) {
    const ret = arg1.value;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_weak_cfb42702fa4177b5(arg0) {
    const ret = arg0.weak();
    return ret;
};

export function __wbg_width_0d7b0b5ad3c2009f(arg0) {
    const ret = arg0.width;
    return ret;
};

export function __wbg_width_36ca6e422d0da22f(arg0) {
    const ret = arg0.width;
    return ret;
};

export function __wbg_width_8ae4f29ab9ee6f63(arg0) {
    const ret = arg0.width;
    return ret;
};

export function __wbg_x_81e0469be7b2266c(arg0) {
    const ret = arg0.x;
    return ret;
};

export function __wbg_y_e33ef4dbf3ba066d(arg0) {
    const ret = arg0.y;
    return ret;
};

export function __wbindgen_bigint_from_i64(arg0) {
    const ret = arg0;
    return ret;
};

export function __wbindgen_bigint_from_u64(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return ret;
};

export function __wbindgen_bigint_get_as_i64(arg0, arg1) {
    const v = arg1;
    const ret = typeof(v) === 'bigint' ? v : undefined;
    getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

export function __wbindgen_boolean_get(arg0) {
    const v = arg0;
    const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
    return ret;
};

export function __wbindgen_cb_drop(arg0) {
    const obj = arg0.original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

export function __wbindgen_closure_wrapper2789(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 411, __wbg_adapter_50);
    return ret;
};

export function __wbindgen_closure_wrapper2791(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 417, __wbg_adapter_53);
    return ret;
};

export function __wbindgen_closure_wrapper2793(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 409, __wbg_adapter_56);
    return ret;
};

export function __wbindgen_closure_wrapper2795(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 415, __wbg_adapter_59);
    return ret;
};

export function __wbindgen_closure_wrapper2797(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 413, __wbg_adapter_62);
    return ret;
};

export function __wbindgen_closure_wrapper4597(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 501, __wbg_adapter_65);
    return ret;
};

export function __wbindgen_debug_string(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbindgen_error_new(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

export function __wbindgen_in(arg0, arg1) {
    const ret = arg0 in arg1;
    return ret;
};

export function __wbindgen_init_externref_table() {
    const table = wasm.__wbindgen_export_4;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

export function __wbindgen_is_bigint(arg0) {
    const ret = typeof(arg0) === 'bigint';
    return ret;
};

export function __wbindgen_is_function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

export function __wbindgen_is_object(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

export function __wbindgen_is_string(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

export function __wbindgen_is_undefined(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

export function __wbindgen_jsval_eq(arg0, arg1) {
    const ret = arg0 === arg1;
    return ret;
};

export function __wbindgen_jsval_loose_eq(arg0, arg1) {
    const ret = arg0 == arg1;
    return ret;
};

export function __wbindgen_memory() {
    const ret = wasm.memory;
    return ret;
};

export function __wbindgen_number_get(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return ret;
};

export function __wbindgen_string_get(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

