use dwn_rs_stores::PutDataResults;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::ser::serializer;

#[wasm_bindgen(typescript_custom_section)]
const DATAPUT_IMPORT: &'static str =
    r#"import { DataStorePutResult } from "@tbd54566975/dwn-sdk-js";"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "DataStorePutResult")]
    pub type DataStorePutResult;

    #[wasm_bindgen(typescript_type = "DataStoreGetResult")]
    pub type DataStoreGetResult;
}

impl From<PutDataResults> for DataStorePutResult {
    fn from(value: PutDataResults) -> Self {
        if let Ok(d) = value.serialize(&serializer()) {
            return d.into();
        }

        wasm_bindgen::JsValue::undefined().into()
    }
}
