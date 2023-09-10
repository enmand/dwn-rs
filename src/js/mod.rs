pub mod errors;
pub mod filter;
pub mod message;

pub use errors::*;
pub use filter::*;
pub use message::*;

use crate::{IndexValue, Indexes};
use crate::{MessageStore, SurrealDB as RealSurreal};

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

extern crate console_error_panic_hook;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

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
    pub async fn connect(&mut self, connstr: &str) -> Result<(), StoreError> {
        self.store
            .connect(&connstr)
            .await
            .map_err(Into::<StoreError>::into)
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
        _opts: Option<MessageStoreOptions>,
    ) -> Result<(), StoreError> {
        let indexes: Indexes =
            serde_wasm_bindgen::from_value::<HashMap<String, IndexValue>>(indexes.into())
                .unwrap()
                .into();

        let _ = self
            .store
            .put(&tenant, message.into(), indexes)
            .await
            .map_err(Into::<StoreError>::into)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn get(
        &self,
        tenant: &str,
        cid: String,
        _opts: Option<MessageStoreOptions>,
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
        filter: &Filter,
        _opts: Option<MessageStoreOptions>,
    ) -> Result<GenericMessageArray, String> {
        let messages = self
            .store
            .query(tenant, filter.into())
            .await
            .map_err(|e| e.to_string())?;

        Ok(messages.into())
    }

    #[wasm_bindgen]
    pub async fn delete(
        &self,
        tenant: &str,
        cid: String,
        _opts: Option<MessageStoreOptions>,
    ) -> Result<(), JsError> {
        self.store.delete(tenant, cid).await.map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn clear(&self) -> Result<(), JsError> {
        self.store.clear().await.map_err(Into::into)
    }
}
