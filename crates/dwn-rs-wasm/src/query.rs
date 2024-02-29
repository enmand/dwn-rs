use dwn_rs_core::Message;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use dwn_rs_stores::{Cursor, MessageSort, Pagination, QueryReturn};

use crate::ser::serializer;

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

    #[wasm_bindgen(typescript_type = "Pagination")]
    pub type JSPagination;

    #[wasm_bindgen(typescript_type = "PaginationCursor")]
    pub type JSPaginationCursor;
}

impl From<QueryReturn<Message>> for JSQueryReturn {
    fn from(value: QueryReturn<Message>) -> Self {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            messages: &'a [Message],
            cursor: Option<Cursor>,
        }

        let wrapper = Wrapper {
            messages: value.items.as_slice(),
            cursor: value.cursor,
        };

        if let Ok(m) = wrapper.serialize(&serializer()) {
            return m.into();
        }

        wasm_bindgen::JsValue::default().into()
    }
}

impl From<QueryReturn<String>> for JSQueryReturn {
    fn from(value: QueryReturn<String>) -> Self {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            events: &'a [String],
            cursor: Option<Cursor>,
        }

        let wrapper = Wrapper {
            events: value.items.as_slice(),
            cursor: value.cursor,
        };

        if let Ok(m) = wrapper.serialize(&serializer()) {
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

impl TryFrom<JSPaginationCursor> for Cursor {
    type Error = serde_wasm_bindgen::Error;
    fn try_from(value: JSPaginationCursor) -> Result<Self, serde_wasm_bindgen::Error> {
        if value.is_undefined() {
            return Ok(Cursor::default());
        }

        serde_wasm_bindgen::from_value(value.into())
    }
}
