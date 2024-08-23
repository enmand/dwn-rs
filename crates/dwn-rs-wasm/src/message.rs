use alloc::{format, vec, vec::Vec};
use serde::Serialize;
use wasm_bindgen::{prelude::*, throw_str};

use dwn_rs_core::Message;

use crate::ser::serializer;

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
            throw_str("Message is undefined");
        }

        match serde_wasm_bindgen::from_value(value.into()) {
            Ok(m) => m,
            Err(e) => throw_str(&format!("unable to deserialize message: {:?}", e)),
        }
    }
}

impl From<Message> for GenericMessage {
    fn from(value: Message) -> Self {
        match value.serialize(&serializer()) {
            Ok(m) => m.into(),
            Err(e) => throw_str(&format!("unable to serialize message: {:?}", e)),
        }
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
        match value.serialize(&serializer()) {
            Ok(m) => m.into(),
            Err(e) => throw_str(&format!("unable to serialize messages: {:?}", e)),
        }
    }
}
