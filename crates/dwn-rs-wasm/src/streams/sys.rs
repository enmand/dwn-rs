use js_sys::{AsyncIterator, Function, Iterator, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "events")]
extern "C" {
    #[wasm_bindgen(extends = Object, js_name = EventEmitter, typescript_type = "EventEmitter")]
    #[derive(Debug, Clone)]
    /// EventEmitters are objects that emit named events that cause Function objects ("listeners") to be called.a
    /// This is a very simple wrapper on the EventEmitter class in node.js (and provided by the `events module).
    pub type EventEmitter;

    #[wasm_bindgen(constructor)]
    pub fn new() -> EventEmitter;

    #[wasm_bindgen(method, js_name = on)]
    pub fn on(this: &EventEmitter, event: &str, callback: &Function);

    #[wasm_bindgen(method, js_name = once)]
    pub fn once(this: &EventEmitter, event: &str, callback: &Function);

    #[wasm_bindgen(method, js_name = off)]
    pub fn off(this: &EventEmitter, event: &str, callback: &Function);

    #[wasm_bindgen(method, js_name = emit)]
    pub fn emit(this: &EventEmitter, event: &str, args: JsValue);
}

// Basic bindings to the node.js Writable stream module.
#[wasm_bindgen(module = "readable-stream")]
extern "C" {
    #[wasm_bindgen(extends = EventEmitter, js_name = "Writable", typescript_type = "Writable")]
    #[derive(Debug, Clone)]
    pub type Writable;
}

// Bindings to the node.js Readable stream module.
#[wasm_bindgen(module = "readable-stream")]
extern "C" {
    #[wasm_bindgen(extends = EventEmitter, extends = AsyncIterator, extends = Iterator, js_name = "Readable", typescript_type = "Readable")]
    #[derive(Debug, Clone)]
    pub type Readable;

    #[wasm_bindgen(constructor)]
    pub fn new() -> Readable;

    #[wasm_bindgen(method, js_name = destroy)]
    pub fn destroy(this: &Readable, err: &str) -> bool;

    #[wasm_bindgen(method, js_name = isPaused)]
    pub fn is_paused(this: &Readable) -> bool;

    #[wasm_bindgen(method)]
    pub fn pause(this: &Readable);

    #[wasm_bindgen(method)]
    pub fn pipe(this: &Readable, dest: &Writable) -> Writable;

    #[wasm_bindgen(method)]
    pub fn read(this: &Readable, size: Option<usize>) -> JsValue;

    #[wasm_bindgen(method)]
    pub fn resume(this: &Readable);

    #[wasm_bindgen(method, js_name = setEncoding)]
    pub fn set_encoding(this: &Readable, encoding: &str);

    #[wasm_bindgen(method)]
    pub fn unpipe(this: &Readable, dest: Option<Writable>);

    #[wasm_bindgen(method)]
    pub fn unshift(this: &Readable, chunk: JsValue, encoding: Option<&str>);

    #[wasm_bindgen(method)]
    pub fn wrap(this: &Readable, stream: Readable) -> Readable;

    #[wasm_bindgen(method, getter)]
    pub fn closed(this: &Readable) -> bool;

    #[wasm_bindgen(method, getter)]
    pub fn destroyed(this: &Readable) -> bool;

    #[wasm_bindgen(method, getter)]
    pub fn readable(this: &Readable) -> bool;

    #[wasm_bindgen(method, getter, js_name = readableAborted)]
    pub fn readable_aborted(this: &Readable) -> bool;

    #[wasm_bindgen(method, getter, js_name = readableDidRead)]
    pub fn readable_did_read(this: &Readable) -> bool;

    #[wasm_bindgen(method, getter, js_name = readableEncoding)]
    pub fn readable_encoding(this: &Readable) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name = readableEnded)]
    pub fn readable_ended(this: &Readable) -> bool;

    #[wasm_bindgen(method, getter, js_name = errored)]
    pub fn errored(this: &Readable) -> bool;

    #[wasm_bindgen(method, getter, js_name = readableFlowing)]
    pub fn readable_flowing(this: &Readable) -> Option<bool>;

    #[wasm_bindgen(method, getter, js_name = readableHighWaterMark)]
    pub fn readable_high_water_mark(this: &Readable) -> f64;

    #[wasm_bindgen(method, getter, js_name = readableLength)]
    pub fn readable_length(this: &Readable) -> f64;

    #[wasm_bindgen(method, getter, js_name = readableObjectMode)]
    pub fn readable_object_mode(this: &Readable) -> bool;
}

// Bindings to the node.js Duplex stream module.
#[wasm_bindgen(module = "readable-stream")]
extern "C" {
    #[wasm_bindgen(extends = Readable, extends = Writable, js_name = "Duplex", typescript_type = "Duplex")]
    #[derive(Debug, Clone)]
    pub type Duplex;

    #[wasm_bindgen(constructor)]
    pub fn new() -> Duplex;

    #[wasm_bindgen(method, getter, js_name = allowHalfOpen)]
    pub fn allow_half_open(this: &Duplex) -> bool;

    #[wasm_bindgen(method, setter, js_name = allowHalfOpen)]
    pub fn set_allow_half_open(this: &Duplex, value: bool);
}

// Bindings to the node.js Transform stream module.
#[wasm_bindgen(module = "readable-stream")]
extern "C" {
    #[wasm_bindgen(extends = Readable, extends = Writable, js_name = "Transform", typescript_type = "Transform")]
    #[derive(Debug, Clone)]
    pub type Transform;
}

// Bindings to the node.js PassThrough stream module.
#[wasm_bindgen(module = "readable-stream")]
extern "C" {
    #[wasm_bindgen(extends = Transform, js_name = "PassThrough", typescript_type = "PassThrough")]
    #[derive(Debug, Clone)]
    pub type PassThrough;
}
