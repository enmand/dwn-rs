use alloc::string::{String, ToString};
use wasm_bindgen::{prelude::*, throw_str};

use dwn_rs_core::{
    filters::{FilterKey, FilterSet, Filters, ValueFilter},
    value::MapValue,
};

use serde::Serialize;

#[wasm_bindgen(typescript_custom_section)]
const INDEX_MAP: &'static str = r#"import { Filter } from "@tbd54566975/dwn-sdk-js";"#;

#[wasm_bindgen(module = "@tbd54566975/dwn-sdk-js")]
extern "C" {
    #[wasm_bindgen(typescript_type = "Filter[]")]
    pub type Filter;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "KeyValues")]
    pub type IndexMap;
}

impl From<Filter> for Filters {
    fn from(value: Filter) -> Self {
        match serde_wasm_bindgen::from_value::<FilterSet<String>>(value.into()) {
            Ok(m) => m
                .into_iter()
                .map(|f: ValueFilter<String>| {
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
        match serde_wasm_bindgen::from_value::<MapValue>(value.into()) {
            Ok(m) => m.into_iter().fold(
                (MapValue::new(), MapValue::new()),
                |(mut indexes, mut tags), (k, v)| {
                    if let Some(tag) = k.strip_prefix("tag.") {
                        tags.insert(tag.to_string(), v);
                    } else {
                        indexes.insert(k, v);
                    }

                    (indexes, tags)
                },
            ),
            Err(e) => throw_str(&format!("unable to deserialize indexes: {:?}", e)),
        }
    }
}

impl From<IndexMap> for MapValue {
    fn from(value: IndexMap) -> Self {
        match serde_wasm_bindgen::from_value::<MapValue>(value.into()) {
            Ok(m) => m,
            Err(e) => throw_str(&format!("unable to deserialize indexes: {:?}", e)),
        }
    }
}

impl From<MapValue> for IndexMap {
    fn from(value: MapValue) -> Self {
        match value.serialize(&crate::ser::serializer()) {
            Ok(m) => m.into(),
            Err(e) => throw_str(&format!("unable to serialize indexes: {:?}", e)),
        }
    }
}

impl std::fmt::Debug for IndexMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexMap").finish()
    }
}
