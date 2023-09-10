use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const INDEX_MAP: &'static str = r#"import { Filters } from "@tbd54566975/dwn-sdk-js/types/message-types";

type IndexMap = {
    [key: string]: string | boolean;
};"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js/types/message-types")]
extern "C" {
    #[wasm_bindgen(typescript_type = "Filter")]
    pub type Filter;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IndexMap")]
    pub type IndexMap;
}
