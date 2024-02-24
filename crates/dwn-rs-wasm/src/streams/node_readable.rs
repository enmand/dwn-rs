use std::{
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
};

use futures_util::StreamExt;
use tokio_stream::Stream;
use wasm_bindgen::prelude::*;
use wasm_streams::readable::IntoStream;

// Bindings to the node.js Readable stream module. We use `node:stream` instead of `readable-stream`
// because `readable-stream` does not implement the conversion to Web Streams.
#[wasm_bindgen(module = "node:stream")]
extern "C" {
    #[wasm_bindgen(extends = ::js_sys::Object, js_name = "Readable", typescript_type = "Readable")]
    #[derive(Clone, Debug)]
    pub type Readable;

    #[wasm_bindgen(constructor)]
    pub fn new() -> Readable;

    #[wasm_bindgen(static_method_of = Readable, js_name = "toWeb")]
    pub fn to_web(this: Readable) -> web_sys::ReadableStream;

    #[wasm_bindgen(static_method_of = Readable, js_name = "fromWeb")]
    pub fn from_web(this: web_sys::ReadableStream) -> Readable;
}

/// NodeReadable is a wrapper around a node.js Readable stream. It implements the `Stream` trait
/// from the `futures` crate, allowing it to be used with async/await. It converts the values from
/// JavaScript (as a Uint8Arary) to Rust (as a Vec<u8>).
pub struct NodeReadable<'a> {
    inner: Mutex<IntoStream<'a>>,
}

impl<'a> NodeReadable<'a> {
    /// New creates a new NodeReadable from a node.js Readable stream.
    pub fn new(r: Readable) -> Self {
        Self {
            inner: Mutex::new(
                wasm_streams::ReadableStream::from_raw(Readable::to_web(r).dyn_into().unwrap())
                    .into_stream(),
            ),
        }
    }
}

impl<'a> Stream for NodeReadable<'a> {
    type Item = Vec<u8>;

    fn poll_next<'c>(self: Pin<&mut Self>, cx: &mut Context<'c>) -> Poll<Option<Self::Item>> {
        match self.inner.lock() {
            Ok(mut i) => match i.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(i))) => {
                    // our sub-stream is ready with data, convert it to a Vec<u8>
                    std::task::Poll::Ready(Some(js_sys::Uint8Array::new(&i).to_vec()))
                }
                Poll::Ready(Some(Err(_))) => std::task::Poll::Ready(None),
                Poll::Pending => std::task::Poll::Pending,
                Poll::Ready(None) => std::task::Poll::Ready(None),
            },
            Err(_) => Poll::Ready(None),
        }
    }
}

unsafe impl Send for NodeReadable<'_> {}
