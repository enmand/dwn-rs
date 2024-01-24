use crate::{filters::Pagination, MessageSort, QueryReturn};

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const QUERY_TYPES: &'static str = r#"
export type QueryReturn = {
    messages: GenericMessage[];
    paginationMessageCid?: string;
};

import { MessageSort, Pagination } from "@tbd54566975/dwn-sdk-js";
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "QueryReturn")]
    pub type JSQueryReturn;

    #[wasm_bindgen(typescript_type = "MessageSort")]
    pub type JSMessageSort;

    #[derive(Debug)]
    #[wasm_bindgen(typescript_type = "Pagination")]
    pub type JSPagination;
}

impl From<QueryReturn> for JSQueryReturn {
    fn from(value: QueryReturn) -> Self {
        if let Ok(m) = value.serialize(&serializer()) {
            return m.into();
        }

        wasm_bindgen::JsValue::default().into()
    }
}

impl From<JSMessageSort> for MessageSort {
    fn from(value: JSMessageSort) -> Self {
        if value.is_undefined() {
            return MessageSort::default();
        }

        match serde_wasm_bindgen::from_value(value.into()) {
            Ok(m) => m,
            Err(_) => MessageSort::default(),
        }
    }
}

impl TryFrom<JSPagination> for Pagination {
    type Error = serde_wasm_bindgen::Error;
    fn try_from(value: JSPagination) -> Result<Self, serde_wasm_bindgen::Error> {
        if value.is_undefined() {
            return Ok(Pagination::default());
        }

        serde_wasm_bindgen::from_value(value.into())
    }
}

#[inline]
fn serializer() -> serde_wasm_bindgen::Serializer {
    serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true)
}
