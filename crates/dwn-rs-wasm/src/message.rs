use alloc::string::ToString;
use alloc::vec::Vec;
use serde::Serialize;
use wasm_bindgen::{prelude::*, throw_str};

use dwn_rs_core::{Descriptor, Message};

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

impl From<&GenericMessage> for Message<Descriptor> {
    fn from(value: &GenericMessage) -> Self {
        if value.is_undefined() {
            throw_str("Message is undefined");
        }

        let t = serde_wasm_bindgen::from_value(value.into());

        match t {
            Ok(m) => m,
            Err(e) => throw_str(e.to_string().as_str()),
        }
    }
}

impl From<Message<Descriptor>> for GenericMessage {
    fn from(value: Message<Descriptor>) -> Self {
        value
            .serialize(&serializer())
            .expect_throw("unable to serialize message")
            .into()
    }
}

impl From<&GenericMessageArray> for Vec<Message<Descriptor>> {
    fn from(value: &GenericMessageArray) -> Self {
        serde_wasm_bindgen::from_value(value.into()).expect_throw("unable to deserialize messages")
    }
}

impl From<Vec<Message<Descriptor>>> for GenericMessageArray {
    fn from(value: Vec<Message<Descriptor>>) -> Self {
        value
            .serialize(&serializer())
            .expect_throw("unable to serialize messages")
            .into()
    }
}
