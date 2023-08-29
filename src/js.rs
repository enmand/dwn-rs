use crate::Message;
use crate::SurrealDB as RealSurreal;
//#[cfg(target_arch = "wasm32")]
//use async_trait::async_trait;
use std::default::Default;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use web_sys::AbortSignal;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js/types")]
extern "C" {
    pub type GenericMessage;
}

impl From<GenericMessage> for Message {
    fn from(value: GenericMessage) -> Self {
        if let Ok(m) = serde_wasm_bindgen::from_value(value.into()) {
            return m;
        }

        Message::default()
    }
}

impl From<Message> for GenericMessage {
    fn from(value: Message) -> Self {
        if let Ok(m) = serde_wasm_bindgen::to_value(&value) {
            return m.into();
        }

        wasm_bindgen::JsValue::default().into()
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct MessageStoreOptions {
    signal: Option<AbortSignal>,
    _marker: std::marker::PhantomPinned,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl MessageStoreOptions {
    pub fn new(signal: Option<AbortSignal>) -> Self {
        Self {
            signal,
            _marker: std::marker::PhantomPinned,
        }
    }
}

#[wasm_bindgen]
pub struct JSSurrealDB {
    store: RealSurreal,
}

#[wasm_bindgen]
//#[cfg(target_arch = "wasm32")]
impl JSSurrealDB {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            store: RealSurreal::new(),
        }
    }

    //pub async fn connect(&self, connstr: String) -> Result<(), SurrealDBError> {}

    async fn close() {}

    //#[cfg(target_arch = "wasm32")]
    //async fn put(&self, message: &Message, indexes: Vec<Index>, opts: Option<MessageStoreOptions>) {}

    //#[cfg(target_arch = "wasm32")]
    //async fn get(&self, cid: String, opts: Option<MessageStoreOptions>) -> Message {
    //    Mexc sage::default()
    //}

    //#[cfg(target_arch = "wasm32")]
    //async fn query(&self, filter: Vec<Filter>, opts: Option<MessageStoreOptions>) -> Vec<Message> {
    //    vec![]
    //}

    //#[cfg(target_arch = "wasm32")]
    //async fn delete(&self, cid: String, opts: Option<MessageStoreOptions>) {}

    async fn clear(&self) {}
}
