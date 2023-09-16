pub mod filter;
pub mod message;

pub use filter::*;
pub use message::*;

use crate::{IndexValue, Indexes};
use crate::{IndexValue, Indexes, SurrealDBError};
use crate::{MessageStore, SurrealDB as RealSurreal};

use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

extern crate console_error_panic_hook;

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

impl From<SurrealDBError> for JsValue {
    fn from(err: SurrealDBError) -> Self {
        JsValue::from_str(&format!("{:?}", err.to_string()))
    }
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
    pub async fn connect(&mut self, connstr: &str) -> Result<(), JsValue> {
        self.store.connect(&connstr).await.map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {
        self.store.close().await;
    }

    #[wasm_bindgen]
    pub async fn open(&mut self) -> Result<(), JsValue> {
        self.store.open().await.map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn put(
        &self,
        tenant: &str,
        message: &GenericMessage,
        indexes: IndexMap,
        _opts: Option<MessageStoreOptions>,
    ) -> Result<(), JsValue> {
        let indexes: Indexes =
            serde_wasm_bindgen::from_value::<BTreeMap<String, IndexValue>>(indexes.into())
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
        _opts: Option<MessageStoreOptions>,
    ) -> Result<GenericMessage, JsValue> {
        match self.store.get(tenant.into(), cid).await {
            Ok(m) => Ok(m.into()),
            Err(_) => Ok(JsValue::undefined().into()),
        }
    }

    #[wasm_bindgen]
    pub async fn query(
        &self,
        tenant: &str,
        filter: &Filter,
        _opts: Option<MessageStoreOptions>,
    ) -> Result<GenericMessageArray, JsValue> {
        let messages = self
            .store
            .query(tenant.into(), filter.into())
            .await
            .map_err(Into::<JsValue>::into)?;

        Ok(messages.into())
    }

    #[wasm_bindgen]
    pub async fn delete(
        &self,
        tenant: &str,
        cid: String,
        _opts: Option<MessageStoreOptions>,
    ) -> Result<(), JsValue> {

        self.store
            .delete(tenant.into(), cid)
            .await
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn clear(&mut self) -> Result<(), JsValue> {
        self.store.clear().await.map_err(Into::into)
    }
}
