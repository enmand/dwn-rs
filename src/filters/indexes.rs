use std::collections::BTreeMap;

use super::value::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Indexes {
    #[serde(flatten)]
    pub indexes: BTreeMap<String, Value>,
}

impl From<BTreeMap<String, Value>> for Indexes {
    fn from(indexes: BTreeMap<String, Value>) -> Self {
        Self { indexes }
    }
}

impl From<Vec<(String, Value)>> for Indexes {
    fn from(indexes: Vec<(String, Value)>) -> Self {
        Self {
            indexes: indexes.into_iter().collect(),
        }
    }
}

impl<const N: usize> From<[(&str, Value); N]> for Indexes {
    fn from(indexes: [(&str, Value); N]) -> Self {
        Self {
            indexes: indexes
                .to_vec()
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        }
    }
}
