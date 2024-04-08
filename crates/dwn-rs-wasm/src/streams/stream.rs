use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{pin_mut, StreamExt};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio_stream::Stream;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::AbortController;

use crate::streams::sys::make_readable;

use super::sys::Readable;

#[derive(Clone, Debug)]
/// StreamReadable is a wrapper around a readable-stream Readable Stream. This is commonly used
/// in browsers and in Node to read data from a stream, using the Node-compatiable API.
///
/// The stream is used to read data from the JavaScript stream, and return items as a JsValue.
pub struct StreamReadable {
    readable: Readable,
}

/// IntoStream is the the implementation for tokio::Stream, for the StreamReadable stream. This
/// can be used in Rust to read data from the JsvaScript stream, and return items as a JsValue.
pub struct IntoStream {
    inner: StreamReadable,
    data_rx: UnboundedReceiver<JsValue>,
    done_rx: UnboundedReceiver<()>,
    done: bool,
    data_cb: Closure<dyn FnMut(JsValue)>,
    end_cb: Closure<dyn FnMut()>,
}

impl StreamReadable {
    pub fn new(r: Readable) -> Self {
        Self {
            readable: r.clone(),
        }
    }

    pub fn as_raw(&self) -> &Readable {
        &self.readable
    }

    /// from_stream creates a new StreamReadable from a Rust Stream. This function will return a
    /// new StreamReadable, and the Readable (accessible as as_raw) will stream data to the
    /// JavaScript stream, as JsValues.
    pub async fn from_stream<St>(stream: St) -> Self
    where
        St: Stream<Item = Result<JsValue, JsValue>>,
    {
        pin_mut!(stream);
        let (data_tx, mut data_rx) = unbounded_channel::<JsValue>();
        let controller = AbortController::new().unwrap();

        while let Some(item) = stream.next().await {
            let data_tx = data_tx.clone();
            let controller = controller.clone();
            spawn_local(async move {
                match item {
                    Ok(i) => {
                        data_tx.send(i).unwrap();
                    }
                    Err(e) => {
                        if e.is_null() {
                            data_tx.send(JsValue::NULL).unwrap();
                        } else {
                            controller.abort();
                        }
                    }
                }
            });
        }

        let newr = make_readable(
            // TODO: the closure should take a `size` argument, and properly buffer the data
            Closure::wrap(Box::new(move |_size| -> JsValue {
                match data_rx.blocking_recv() {
                    Some(d) => d,
                    None => JsValue::NULL,
                }
            }) as Box<dyn FnMut(JsValue) -> JsValue>)
            .into_js_value(),
            controller.signal(),
        );

        Self::new(newr)
    }

    /// into_stream creates a new Stream from the StreamReadable stream. This function locks the StreamReadable in
    /// JavaScript, and attaches the handlers for data and end events. It then returns a new Stream
    /// from the locked data, and passes the values through unbounded channels.
    pub fn into_stream(self) -> IntoStream {
        IntoStream::new(self)
    }
}

impl IntoStream {
    pub fn new(r: StreamReadable) -> Self {
        let readable = r.as_raw();
        let (data_tx, data_rx) = unbounded_channel::<JsValue>();
        let (done_tx, done_rx) = unbounded_channel::<()>();

        let data_cb = Closure::wrap(Box::new(move |d| {
            data_tx.send(d).unwrap();
        }) as Box<dyn FnMut(JsValue)>);
        readable.on("data", data_cb.as_ref().unchecked_ref());

        let end_cb = Closure::wrap(Box::new(move || {
            done_tx.send(()).unwrap();
        }) as Box<dyn FnMut()>);
        readable.on("end", end_cb.as_ref().unchecked_ref());

        Self {
            inner: r,
            data_rx,
            done_rx,
            done: false,
            data_cb,
            end_cb,
        }
    }
}

impl Stream for IntoStream {
    type Item = JsValue;

    // poll_next is the main function that drives the stream. It is called by the runtime to
    // read the data in the Readable, and return it as a JsValue.
    fn poll_next<'c>(self: Pin<&mut Self>, cx: &mut Context<'c>) -> Poll<Option<Self::Item>> {
        let state = self.get_mut();
        let data_rx = state.data_rx.poll_recv(cx);
        let done_rx = state.done_rx.poll_recv(cx);

        // If we end, but the stream still has data left, we need to keep polling until the data is
        // done.
        let poll = match state.done && data_rx.is_pending() {
            false => data_rx,
            true => Poll::Ready(None),
        };

        // If we've recieved the done signal, and the data_rx is no longer ready, end the stream.
        if done_rx.is_ready() {
            state.done = true;
        };

        poll
    }
}

impl Drop for IntoStream {
    fn drop(&mut self) {
        // Drop the event listeners so that we don't try to call them after the stream is dropped.
        self.inner
            .readable
            .off("data", self.data_cb.as_ref().unchecked_ref());
        self.inner
            .readable
            .off("end", self.end_cb.as_ref().unchecked_ref());
    }
}

// The following unsafe implementations are required to ensure the vector data can be sent
// between WASM calls, where threads are not allowed.
// TODO: we should find a better way to do this, and remove this unsafe impl. (e.g. for WASIX
// environments that do support threads.)
unsafe impl Send for IntoStream {}
