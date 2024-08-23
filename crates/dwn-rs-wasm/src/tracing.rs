use wasm_bindgen::{prelude::*, throw_str};

#[wasm_bindgen]
/// Initialize tracing with a console subscriber. This is useful for debugging
/// in the browser. By default, the subscriber will log all events at the `Error`
/// level. This can be adjusted by calling `set_tracing_level`, however you must
/// call this function before any tracing events are emitted.
pub fn init_tracing(level: TracingLevel) {
    let level: tracing::Level = level.into();
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
