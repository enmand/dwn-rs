use std::collections::BTreeMap;

use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, FromVariants)]
#[serde(untagged)]
pub enum IndexValue {
    Bool(bool),
    String(String),
    Number(i64),
    Float(f64),
    Map(BTreeMap<String, IndexValue>),
}

impl From<&str> for IndexValue {
    fn from(s: &str) -> Self {
        IndexValue::String(s.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Indexes {
    #[serde(flatten)]
    pub indexes: BTreeMap<String, IndexValue>,
}

impl From<BTreeMap<String, IndexValue>> for Indexes {
    fn from(indexes: BTreeMap<String, IndexValue>) -> Self {
        Self { indexes }
    }
}

impl From<Vec<(String, IndexValue)>> for Indexes {
    fn from(indexes: Vec<(String, IndexValue)>) -> Self {
        Self {
            indexes: indexes.into_iter().collect(),
        }
    }
}

impl<const N: usize> From<[(&str, IndexValue); N]> for Indexes {
    fn from(indexes: [(&str, IndexValue); N]) -> Self {
        Self {
            indexes: indexes
                .to_vec()
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        }
    }
}
