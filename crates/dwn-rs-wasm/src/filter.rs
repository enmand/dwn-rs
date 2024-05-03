use std::collections::BTreeMap;

use dwn_rs_core::MapValue;
use wasm_bindgen::prelude::*;

use dwn_rs_stores::{FilterKey, Filters, ValueFilter};

#[wasm_bindgen(typescript_custom_section)]
const INDEX_MAP: &'static str = r#"import { Filter } from "@tbd54566975/dwn-sdk-js";

type IndexMap = {
    [key: string]: string | boolean;
};"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "Filter[]")]
    pub type Filter;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IndexMap")]
    pub type IndexMap;
}

impl From<Filter> for Filters {
    fn from(value: Filter) -> Self {
        match serde_wasm_bindgen::from_value::<Vec<BTreeMap<String, dwn_rs_stores::Filter>>>(
            value.into(),
        ) {
            Ok(m) => m
                .into_iter()
                .map(|f: BTreeMap<String, dwn_rs_stores::Filter>| {
                    let fs = f
                        .into_iter()
                        .fold(ValueFilter::default(), |mut filters, (k, v)| {
                            if let Some(tag) = k.strip_prefix("tag.") {
                                filters.insert(FilterKey::Tag(tag.to_string()), v);
                            } else {
                                filters.insert(FilterKey::Index(k), v);
                            }

                            filters
                        });
                    Into::<Filters>::into(fs)
                })
                .collect(),
            Err(err) => panic!("{}", err),
        }
    }
}

impl From<IndexMap> for (MapValue, MapValue) {
    fn from(value: IndexMap) -> Self {
        if let Ok(m) = serde_wasm_bindgen::from_value::<MapValue>(value.into()) {
            return m.into_iter().fold(
                (MapValue::new(), MapValue::new()),
                |(mut indexes, mut tags), (k, v)| {
                    if let Some(tag) = k.strip_prefix("tag.") {
                        tags.insert(tag.to_string(), v);
                    } else {
                        indexes.insert(k, v);
                    }

                    (indexes, tags)
                },
            );
        }

        (MapValue::default(), MapValue::default())
    }
}
