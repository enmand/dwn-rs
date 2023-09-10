pub mod filter;
pub mod message;

pub use filter::*;
pub use message::*;

use crate::{Filters, IndexValue, Indexes};
use crate::{MessageStore, SurrealDB as RealSurreal};
use std::collections::HashMap;
use std::default::Default;
use thiserror::Error;
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

extern crate console_error_panic_hook;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Error, Debug)]
#[error(transparent)]
pub struct JSError(#[from] surrealdb::Error);

#[wasm_bindgen]
pub struct MessageStoreOptions {
    signal: Option<AbortSignal>,
}

#[wasm_bindgen]
impl MessageStoreOptions {
    pub fn new(signal: Option<AbortSignal>) -> Self {
        Self { signal }
    }
}

#[wasm_bindgen(js_name = SurrealDB)]
pub struct JSSurrealDB {
    store: RealSurreal,
}

#[wasm_bindgen(js_class = SurrealDB)]
impl JSSurrealDB {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            store: RealSurreal::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self, connstr: &str) -> Result<(), JsError> {
        self.store.connect(&connstr).await.map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {
        self.store.close().await;
    }

    #[wasm_bindgen]
    pub async fn put(
        &self,
        tenant: String,
        message: &GenericMessage,
        indexes: IndexMap,
        opts: Option<MessageStoreOptions>,
    ) -> Result<(), JsError> {
        // Convert IndexMap, which is an external TypeScript type, represented by a JSValue Object
        // into a HashMap<String, IndexValue> which is a Rust type.
        let indexes: Indexes =
            serde_wasm_bindgen::from_value::<HashMap<String, IndexValue>>(indexes.into())
                .unwrap()
                .into();

        let _ = self
            .store
            .put(&tenant, message.into(), indexes)
            .await
            .map_err(Into::<JsError>::into)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn get(
        &self,
        tenant: &str,
        cid: String,
        opts: Option<MessageStoreOptions>,
    ) -> Result<Option<GenericMessage>, JsError> {
        match self.store.get(tenant, cid).await {
            Ok(m) => match serde_wasm_bindgen::to_value(&m) {
                Ok(v) => Ok(Some(v.into())),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }
    }

    #[wasm_bindgen]
    pub async fn query(
        &self,
        tenant: &str,
        filter: Filter,
        opts: Option<MessageStoreOptions>,
    ) -> Result<GenericMessageArray, String> {
        let messages = self
            .store
            .query(tenant, Filters::default())
            .await
            .map_err(|e| e.to_string())?;

        Ok(messages.into())
    }

    #[wasm_bindgen]
    pub async fn delete(
        &self,
        tenant: &str,
        cid: String,
        opts: Option<MessageStoreOptions>,
    ) -> Result<(), JsError> {
        self.store.delete(tenant, cid).await.map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn clear(&self) -> Result<(), JsError> {
        self.store.clear().await.map_err(Into::into)
    }
}
