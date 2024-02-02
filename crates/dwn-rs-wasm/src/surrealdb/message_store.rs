use std::collections::BTreeMap;

use dwn_rs_stores::{
    errors::MessageStoreError as StoreError,
    filters::value::Value,
    filters::{Indexes, QueryReturn},
    surrealdb::{SurrealDB, SurrealDBError},
    MessageStore,
};
use js_sys::Reflect;
use thiserror::Error;
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

use crate::filter::*;
use crate::message::*;
use crate::query::{JSMessageSort, JSPagination, JSQueryReturn};

#[derive(Error, Debug)]
enum MessageStoreError {
    #[error("Store error: {0}")]
    StoreError(#[from] StoreError),

    #[error("store connection failed: {0}")]
    ConnectionFailed(#[from] SurrealDBError),
}

impl From<MessageStoreError> for JsValue {
    fn from(e: MessageStoreError) -> Self {
        JsValue::from_str(&format!("{}", e))
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "MessageStoreOptions")]
    pub type MessageStoreOptions;
}

impl Default for SurrealMessageStore {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_name = SurrealMessageStore)]
pub struct SurrealMessageStore {
    store: SurrealDB,
}

#[wasm_bindgen(js_class = SurrealMessageStore)]
impl SurrealMessageStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            store: SurrealDB::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self, connstr: &str) -> Result<(), JsValue> {
        self.store
            .connect(connstr)
            .await
            .map_err(MessageStoreError::from)
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {
        self.store.close().await;
    }

    #[wasm_bindgen]
    pub async fn open(&mut self) -> Result<(), JsValue> {
        self.store
            .open()
            .await
            .map_err(MessageStoreError::from)
            .map_err(Into::into)
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
            serde_wasm_bindgen::from_value::<BTreeMap<String, Value>>(indexes.into())?.into();

        let _: Result<_, JsValue> = self
            .store
            .put(tenant, message.into(), indexes)
            .await
            .map_err(MessageStoreError::from)
            .map_err(Into::into);

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

        match self.store.get(tenant, cid).await {
            Ok(m) => Ok(m.into()),
            Err(_) => Ok(JsValue::undefined().into()),
        }
    }

    #[wasm_bindgen]
    pub async fn query(
        &self,
        tenant: &str,
        filter: &Filter,
        message_sort: Option<JSMessageSort>,
        pagination: Option<JSPagination>,
        options: Option<MessageStoreOptions>,
    ) -> Result<JSQueryReturn, JsValue> {
        check_aborted(options)?;

        let page = match pagination {
            Some(p) => Some(match p.try_into() {
                Ok(p) => p,
                Err(_) => {
                    return Ok(QueryReturn::default().into());
                }
            }),
            None => None,
        };

        Ok(self
            .store
            .query(
                tenant,
                filter.into(),
                message_sort.map(|sort| sort.into()),
                page,
            )
            .await
            .map_err(MessageStoreError::from)
            .map_err(Into::<JsValue>::into)?
            .into())
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
            .delete(tenant, cid)
            .await
            .map_err(MessageStoreError::from)
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn clear(&mut self) -> Result<(), JsValue> {
        self.store
            .clear()
            .await
            .map_err(MessageStoreError::from)
            .map_err(Into::into)
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
