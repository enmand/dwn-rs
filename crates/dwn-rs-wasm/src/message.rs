use serde::Serialize;
use wasm_bindgen::prelude::*;

use dwn_rs_core::{Descriptor, Fields, GenericDescriptor, MapValue, Message};

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

impl From<&GenericMessage> for Message<GenericDescriptor, MapValue> {
    fn from(value: &GenericMessage) -> Self {
        if value.is_undefined() {
            return Message::default();
        }

        match serde_wasm_bindgen::from_value(value.into()) {
            Ok(m) => m,
            Err(_) => Message::default(),
        }
    }
}

impl<D, F> From<Message<D, F>> for GenericMessage
where
    D: Descriptor + Serialize,
    F: Fields + Serialize,
{
    fn from(value: Message<D, F>) -> Self {
        if value != Message::<D, F>::default() {
            if let Ok(m) = value.serialize(&serializer()) {
                return m.into();
            }
        }

        wasm_bindgen::JsValue::undefined().into()
    }
}

impl From<&GenericMessageArray> for Vec<Message<GenericDescriptor, MapValue>> {
    fn from(value: &GenericMessageArray) -> Self {
        if let Ok(m) = serde_wasm_bindgen::from_value(value.into()) {
            return m;
        }

        vec![]
    }
}

impl<D, F> From<Vec<Message<D, F>>> for GenericMessageArray
where
    D: Descriptor + Serialize,
    F: Fields + Serialize,
{
    fn from(value: Vec<Message<D, F>>) -> Self {
        if let Ok(m) = value.serialize(&serializer()) {
            return m.into();
        }

        wasm_bindgen::JsValue::default().into()
    }
}
