use std::cell::RefCell;

use wasm_bindgen::prelude::*;

thread_local! {
    pub static TRACING_LEVEL: RefCell<Option<tracing::Level>> = RefCell::new(None);
}

#[wasm_bindgen]
/// Initialize tracing with a console subscriber. This is useful for debugging
/// in the browser. By default, the subscriber will log all events at the `Error`
/// level. This can be adjusted by calling `set_tracing_level`, however you must
/// call this function before any tracing events are emitted.
pub fn init_tracing() {
    let level = TRACING_LEVEL.with(|l| l.borrow().unwrap_or(tracing::Level::ERROR));
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .without_time()
        .init();
}

#[wasm_bindgen]
/// Sets the global traving level. This must be called before the tracing initialization.
pub fn set_tracing_level(level: TracingLevel) {
    TRACING_LEVEL.with(|l| *l.borrow_mut() = Some(level.into()));
}

#[wasm_bindgen]
pub enum TracingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<TracingLevel> for tracing::Level {
    fn from(level: TracingLevel) -> Self {
        match level {
            TracingLevel::Trace => tracing::Level::TRACE,
            TracingLevel::Debug => tracing::Level::DEBUG,
            TracingLevel::Info => tracing::Level::INFO,
            TracingLevel::Warn => tracing::Level::WARN,
            TracingLevel::Error => tracing::Level::ERROR,
        }
    }
}
