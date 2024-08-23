use dwn_rs_core::{stores::ResumableTaskStore, Value};
use dwn_rs_stores::SurrealDB;
use wasm_bindgen::prelude::*;

use crate::task::{JsManagedResumableTask, JsManagedResumableTaskArray};

#[derive(Default)]
#[wasm_bindgen(js_name = SurrealResumableTaskStore)]
pub struct SurrealResumableTaskStore {
    store: SurrealDB,
}

#[wasm_bindgen(js_class = SurrealResumableTaskStore)]
impl SurrealResumableTaskStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SurrealResumableTaskStore {
        SurrealResumableTaskStore {
            store: SurrealDB::new(),
        }
    }

    #[wasm_bindgen]
    pub async fn connect(&mut self, connstr: &str) -> Result<(), JsValue> {
        self.store
            .connect(connstr)
            .await
            .map_err(JsError::from)
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn open(&mut self) -> Result<(), JsValue> {
        self.store
            .open()
            .await
            .map_err(JsError::from)
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn register(
        &self,
        task: JsValue,
        timeout: u32,
    ) -> Result<JsManagedResumableTask, JsValue> {
        let task: Value = serde_wasm_bindgen::from_value(task).map_err(JsValue::from)?;

        Ok(self
            .store
            .register(task, timeout as u64)
            .await
            .map_err(JsError::from)
            .map_err(JsValue::from)?
            .into())
    }

    #[wasm_bindgen]
    pub async fn grab(&self, count: u32) -> Result<JsManagedResumableTaskArray, JsValue> {
        Ok(self
            .store
            .grab::<Value>(count as u64)
            .await
            .map_err(JsError::from)
            .map_err(JsValue::from)?
            .into())
    }

    #[wasm_bindgen]
    pub async fn read(&self, id: &str) -> Result<Option<JsManagedResumableTask>, JsValue> {
        let t = self
            .store
            .read::<Value>(id)
            .await
            .map_err(JsError::from)
            .map_err(JsValue::from)?;

        match t {
            Some(t) => Ok(Some(t.into())),
            None => Ok(None),
        }
    }

    #[wasm_bindgen]
    pub async fn extend(&self, id: &str, timeout: u32) -> Result<(), JsValue> {
        self.store
            .extend(id, timeout as u64)
            .await
            .map_err(JsError::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen]
    pub async fn delete(&self, id: &str) -> Result<(), JsValue> {
        self.store
            .delete(id)
            .await
            .map_err(JsError::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen]
    pub async fn clear(&self) -> Result<(), JsValue> {
        self.store
            .clear()
            .await
            .map_err(JsError::from)
            .map_err(JsValue::from)
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {
        self.store.close().await;
    }
}
