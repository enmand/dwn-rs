use dwn_rs_stores::{
    surrealdb::SurrealDB, Cursor, EventLog, EventLogError, QueryReturn, StoreError, SurrealDBError,
};
use js_sys::Array;
use wasm_bindgen::prelude::*;

use thiserror::Error;

use crate::{
    filter::{Filter, IndexMap},
    query::{JSPaginationCursor, JSQueryReturn},
};

#[derive(Error, Debug)]
enum SurrealEventLogError {
    #[error("Store error: {0}")]
    StoreError(#[from] StoreError),

    #[error("store connection failed: {0}")]
    ConnectionFailed(#[from] SurrealDBError),
}

#[wasm_bindgen(js_name = SurrealEventLog)]
#[derive(Default)]
pub struct SurrealEventLog {
    store: SurrealDB,
}

#[wasm_bindgen(js_class = SurrealEventLog)]
impl SurrealEventLog {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();

        Self {
            store: SurrealDB::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self, connstr: &str) -> Result<(), JsError> {
        self.store
            .connect(connstr)
            .await
            .map_err(SurrealDBError::from)
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {
        self.store.close().await;
    }

    #[wasm_bindgen]
    pub async fn open(&mut self) -> Result<(), JsError> {
        self.store
            .open()
            .await
            .map_err(EventLogError::from)
            .map_err(Into::<JsError>::into)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn append(&self, tenant: &str, cid: &str, indexes: IndexMap) -> Result<(), JsError> {
        let (indexes, _) = indexes.into();
        self.store
            .append(tenant, cid.to_string(), indexes)
            .await
            .map_err(EventLogError::from)
            .map_err(Into::<JsError>::into)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = getEvents)]
    pub async fn get_events(
        &self,
        tenant: &str,
        cursor: Option<JSPaginationCursor>,
    ) -> Result<JSQueryReturn, JsError> {
        self.query_events(tenant, Filter::from(JsValue::from(Array::new())), cursor)
            .await
    }

    #[wasm_bindgen(js_name = queryEvents)]
    pub async fn query_events(
        &self,
        tenant: &str,
        filter: Filter,
        cursor: Option<JSPaginationCursor>,
    ) -> Result<JSQueryReturn, JsError> {
        let cursor = match cursor {
            Some(c) => Some(match TryInto::<Cursor>::try_into(c) {
                Ok(c) => c,
                Err(_) => {
                    return Ok(QueryReturn::<String>::default().into());
                }
            }),
            None => None,
        };

        let res: JSQueryReturn = self
            .store
            .query_events(tenant, filter.into(), cursor)
            .await
            .map_err(EventLogError::from)?
            .into();

        Ok(res)
    }

    #[wasm_bindgen(js_name = deleteEventsByCid)]
    pub async fn delete(&self, tenant: &str, cids: Vec<String>) -> Result<(), JsError> {
        self.store
            .delete(
                tenant,
                cids.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            )
            .await
            .map_err(EventLogError::from)?;

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn clear(&self) -> Result<(), JsError> {
        self.store.clear().await.map_err(EventLogError::from)?;
        Ok(())
    }
}
