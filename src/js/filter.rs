use std::collections::HashMap;

use crate::{Filter as DBFilter, Filters};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const INDEX_MAP: &'static str = r#"import { Filter } from "@tbd54566975/dwn-sdk-js";

type IndexMap = {
    [key: string]: string | boolean;
};"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "Filter")]
    pub type Filter;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IndexMap")]
    pub type IndexMap;
}

impl From<&Filter> for Filters {
    fn from(value: &Filter) -> Self {
        match serde_wasm_bindgen::from_value::<HashMap<String, DBFilter>>(value.into()) {
            Ok(m) => {
                return m.into();
            }
            Err(_) => Filters::default(),
        }
    }
}

impl TryFrom<Filter> for Filters {
    type Error = JsError;

    fn try_from(value: Filter) -> Result<Self, Self::Error> {
        serde_wasm_bindgen::from_value::<HashMap<String, DBFilter>>(value.into())
            .map(|m| m.into())
            .map_err(Into::into)
    }
}

impl From<Filters> for Filter {
    fn from(value: Filters) -> Self {
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
