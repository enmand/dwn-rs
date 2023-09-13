use crate::Message;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const INDEX_MAP: &'static str = r#"import { GenericMessage } from "@tbd54566975/dwn-sdk-js";"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "GenericMessage")]
    pub type GenericMessage;

    #[wasm_bindgen(typescript_type = "GenericMessage[]")]
    pub type GenericMessageArray;
}

impl From<&GenericMessage> for Message {
    fn from(value: &GenericMessage) -> Self {
        if let Ok(m) = serde_wasm_bindgen::from_value(value.into()) {
            return m;
        }

        Message::default()
    }
}

impl TryFrom<GenericMessage> for Message {
    type Error = serde_wasm_bindgen::Error;

    fn try_from(value: GenericMessage) -> Result<Self, Self::Error> {
        serde_wasm_bindgen::from_value(value.into())
    }
}

impl From<Message> for GenericMessage {
    fn from(value: Message) -> Self {
        if let Ok(m) = value.serialize(&serializer()) {
            return m.into();
        }

        wasm_bindgen::JsValue::default().into()
    }
}

impl From<&GenericMessageArray> for Vec<Message> {
    fn from(value: &GenericMessageArray) -> Self {
        if let Ok(m) = serde_wasm_bindgen::from_value(value.into()) {
            return m;
        }

        vec![]
    }
}

impl From<Vec<Message>> for GenericMessageArray {
    fn from(value: Vec<Message>) -> Self {
        if let Ok(m) = value.serialize(&serializer()) {
            return m.into();
        }

        wasm_bindgen::JsValue::default().into()
    }
}

#[inline]
fn serializer() -> serde_wasm_bindgen::Serializer {
    serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true)
}
