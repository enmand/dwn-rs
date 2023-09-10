use crate::SurrealDBError;
use thiserror::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=StoreError)]
#[derive(Error, Debug)]
#[error(transparent)]
pub struct StoreError(#[from] JSSurrealError);

impl From<SurrealDBError> for StoreError {
    fn from(e: SurrealDBError) -> Self {
        Self(JSSurrealError::JSError(e))
    }
}

impl From<serde_wasm_bindgen::Error> for StoreError {
    fn from(e: serde_wasm_bindgen::Error) -> Self {
        Self(JSSurrealError::EncodingError(e))
    }
}

#[derive(Error, Debug)]
enum JSSurrealError {
    #[error("Aborted")]
    Aborted,
    #[error("JSError: {0}")]
    JSError(#[from] SurrealDBError),
    #[error("Encoding error: {0}")]
    EncodingError(#[from] serde_wasm_bindgen::Error),
}
