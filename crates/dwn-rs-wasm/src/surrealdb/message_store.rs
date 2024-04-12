use dwn_rs_core::MapValue;
use dwn_rs_stores::{filters::Indexes, surrealdb::SurrealDB, MessageStore};
use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use web_sys::AbortSignal;

use crate::filter::*;
use crate::message::*;
use crate::query::{JSMessageSort, JSPagination, JSQueryReturn};

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
            .connect(connstr, dwn_rs_stores::surrealdb::Database::Messages)
            .await
            .map_err(Into::<JsError>::into)
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
            .map_err(Into::<JsError>::into)
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

        let indexes: Indexes = serde_wasm_bindgen::from_value::<MapValue>(indexes.into())?.into();

        let _: Result<_, JsError> = self
            .store
            .put(tenant, message.into(), indexes)
            .await
            .map_err(Into::<JsError>::into)
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

        Ok(match self.store.get(tenant, cid).await {
            Ok(v) => v.into(),
            Err(e) => match e {
                dwn_rs_stores::MessageStoreError::StoreError(
                    dwn_rs_stores::StoreError::NotFound,
                ) => return Ok(JsValue::undefined().into()),
                _ => return Err(Into::<JsError>::into(e).into()),
            },
        })
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
                Err(e) => return Err(Into::<JsError>::into(e).into()),
            }),
            None => None,
        };

        let out: JSQueryReturn = match self
            .store
            .query(
                tenant,
                filter.into(),
                message_sort.map(|sort| sort.into()),
                page,
            )
            .await
        {
            Ok(v) => v.into(),
            Err(e) => {
                web_sys::console::log_1(&e.to_string().into());
                return Err(Into::<JsError>::into(e).into());
            }
        };

        Ok(out)
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
            .map_err(Into::<JsError>::into)
            .map_err(Into::<JsValue>::into)
    }

    #[wasm_bindgen]
    pub async fn clear(&mut self) -> Result<(), JsValue> {
        self.store
            .clear()
            .await
            .map_err(Into::<JsError>::into)
            .map_err(Into::<JsValue>::into)
    }
}

fn check_aborted(options: Option<MessageStoreOptions>) -> Result<(), JsValue> {
    if let Some(signal) = options {
        let sig = Reflect::get(&signal.into(), &JsValue::from_str("signal")).expect("signal");
        let sig = AbortSignal::from(sig);
        if sig.aborted() {
            let reason =
                Reflect::get(&sig.into(), &JsValue::from_str("reason")).expect("has reason");

            return Err(reason);
        }
    }

    Ok(())
}
