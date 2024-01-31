pub mod filter;
pub mod message;
pub mod query;

pub use filter::*;
use js_sys::Reflect;
pub use message::*;

use std::collections::BTreeMap;

use thiserror::Error;
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

use self::query::{JSMessageSort, JSPagination, JSQueryReturn};
use dwn_rs_messagestore::surrealdb::SurrealDB as RealSurreal;
use dwn_rs_messagestore::SurrealDBError;
use dwn_rs_stores::{
    errors::MessageStoreError as StoreError,
    filters::{Indexes, QueryReturn},
    value::Value,
    MessageStore,
};

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
    pub async fn connect(&mut self, connstr: &str) -> Result<(), JsValue> {
        self.store
            .connect(&connstr)
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
            .put(tenant.into(), message.into(), indexes)
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
                tenant.into(),
                filter.into(),
                match message_sort {
                    Some(sort) => Some(sort.into()),
                    None => None,
                },
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
            .delete(tenant.into(), cid)
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
