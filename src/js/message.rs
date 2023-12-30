use crate::Message;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const MESSAGE_IMPORT: &'static str = r#"import { GenericMessage } from "@tbd54566975/dwn-sdk-js";"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "GenericMessage")]
    pub type GenericMessage;

    #[wasm_bindgen(typescript_type = "GenericMessage[]")]
    pub type GenericMessageArray;
}

impl From<&GenericMessage> for Message {
    fn from(value: &GenericMessage) -> Self {
        if value.is_undefined() {
            return Message::default();
        }

        match serde_wasm_bindgen::from_value(value.into()) {
            Ok(m) => m,
            Err(e) => Message::default(),
        }
    }
}

impl From<Message> for GenericMessage {
    fn from(value: Message) -> Self {
        if value != Message::default() {
            if let Ok(m) = value.serialize(&serializer()) {
                return m.into();
            }
        }

        wasm_bindgen::JsValue::undefined().into()
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
