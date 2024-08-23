use alloc::vec::Vec;
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

        serde_wasm_bindgen::from_value(value.into()).expect_throw("unable to deserialize message")
    }
}

impl From<Message> for GenericMessage {
    fn from(value: Message) -> Self {
        value
            .serialize(&serializer())
            .expect_throw("unable to serialize message")
            .into()
    }
}

impl From<&GenericMessageArray> for Vec<Message> {
    fn from(value: &GenericMessageArray) -> Self {
        serde_wasm_bindgen::from_value(value.into()).expect_throw("unable to deserialize messages")
    }
}

impl From<Vec<Message>> for GenericMessageArray {
    fn from(value: Vec<Message>) -> Self {
        value
            .serialize(&serializer())
            .expect_throw("unable to serialize messages")
            .into()
    }
}
