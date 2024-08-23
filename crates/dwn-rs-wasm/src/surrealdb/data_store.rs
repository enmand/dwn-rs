use futures_util::StreamExt;
use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

use dwn_rs_core::stores::DataStore;
use dwn_rs_stores::surrealdb::SurrealDB;

use crate::{
    data::{DataStoreGetResult, DataStorePutResult},
    streams::{stream::StreamReadable, sys::Readable},
};

#[wasm_bindgen(js_name = SurrealDataStore)]
pub struct SurrealDataStore {
    store: SurrealDB,
}

impl Default for SurrealDataStore {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(js_class = SurrealDataStore)]
impl SurrealDataStore {
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
            .map_err(Into::<JsError>::into)
            .map_err(Into::into)
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
        record_id: &str,
        cid: &str,
        value: Readable,
    ) -> Result<DataStorePutResult, JsValue> {
        let readable = StreamReadable::new(value).into_stream().map(|r| {
            let val = serde_wasm_bindgen::to_value(&r).unwrap();
            js_sys::Uint8Array::new(&val).to_vec()
        });

        match self
            .store
            .put(tenant, record_id, cid, readable)
            .await
            .map_err(Into::<JsError>::into)
        {
            Ok(d) => Ok(d.into()),
            Err(e) => Err(e.into()),
        }
    }

    #[wasm_bindgen]
    pub async fn get(
        &self,
        tenant: &str,
        record_id: &str,
        cid: &str,
    ) -> Result<Option<DataStoreGetResult>, JsValue> {
        let v = match self.store.get(tenant, record_id, cid).await {
            Ok(d) => d,
            Err(_) => return Ok(None),
        };

        let size = v.size;
        let reader = v.data.map(|r| Some(serde_bytes::ByteBuf::from(r)));

        let obj: DataStoreGetResult = JsCast::unchecked_into(Object::new());
        Reflect::set(&obj, &"dataSize".into(), &size.into())?;
        Reflect::set(
            &obj,
            &"dataStream".into(),
            StreamReadable::from_stream(reader).await?.as_raw(),
        )?;

        Ok(Some(obj))
    }

    #[wasm_bindgen]
    pub async fn close(&mut self) {
        self.store.close().await;
    }

    #[wasm_bindgen]
    pub async fn clear(&mut self) -> Result<(), JsValue> {
        self.store
            .clear()
            .await
            .map_err(Into::<JsError>::into)
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn delete(
        &mut self,
        tenant: &str,
        record_id: &str,
        cid: &str,
    ) -> Result<(), JsValue> {
        self.store
            .delete(tenant, record_id, cid)
            .await
            .map_err(Into::<JsError>::into)?;

        Ok(())
    }
}
