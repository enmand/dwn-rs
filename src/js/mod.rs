pub mod filter;
pub mod message;

pub use filter::*;
use js_sys::Reflect;
pub use message::*;

use crate::{IndexValue, Indexes, SurrealDBError};
use crate::{MessageStore, SurrealDB as RealSurreal};

use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

extern crate console_error_panic_hook;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

const INDEX_MAP: &'static str = r#"import { MessageStoreOptions } from "@tbd54566975/dwn-sdk-js";"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "MessageStoreOptions")]
    pub type MessageStoreOptions;
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
        options: Option<MessageStoreOptions>,
    ) -> Result<(), JsValue> {
        check_aborted(options)?;

        let indexes: Indexes =
            serde_wasm_bindgen::from_value::<BTreeMap<String, IndexValue>>(indexes.into())?.into();

        let _ = self
            .store
            .put(tenant.into(), message.into(), indexes)
            .await
            .map_err(Into::<JsValue>::into)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn get(
        &self,
        tenant: &str,
        cid: String,
        options: Option<MessageStoreOptions>,
    ) -> Result<GenericMessage, JsValue> {
        check_aborted(options)?;

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
        options: Option<MessageStoreOptions>,
    ) -> Result<GenericMessageArray, JsValue> {
        check_aborted(options)?;

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
        options: Option<MessageStoreOptions>,
    ) -> Result<(), JsValue> {
        check_aborted(options)?;

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

fn check_aborted(options: Option<MessageStoreOptions>) -> Result<(), JsValue> {
    if let Some(signal) = options {
        let sig = Reflect::get(&signal.into(), &JsValue::from_str("signal")).expect("signal");
        let sig = AbortSignal::from(sig);
        if sig.aborted() {
            let reason = Reflect::get(&sig.into(), &JsValue::from_str("reason")).expect("reason");

            return Err(reason);
        }
    }

    Ok(())
}
