use std::collections::BTreeMap;

use from_variants::FromVariants;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, FromVariants)]
#[serde(untagged)]
pub enum IndexValue {
    Bool(bool),
    String(String),
    Number(i64),
    Float(f64),
    Map(BTreeMap<String, IndexValue>),
    Datetime(chrono::DateTime<chrono::Utc>),
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

// implement a custom Deserialize for IndexValue that will deserialize a string into a DateTime<Utc>
// if the passed string is a valid RFC3339 time. Otherwise, deserialize to the proper IndexValue
// variant.
impl<'de> serde::Deserialize<'de> for IndexValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IndexValueVisitor;

        impl<'de> serde::de::Visitor<'de> for IndexValueVisitor {
            type Value = IndexValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid RFC3339 datetime or a valid IndexValue variant")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match chrono::DateTime::parse_from_rfc3339(value) {
                    Ok(dt) => Ok(IndexValue::Datetime(dt.with_timezone(&chrono::Utc))),
                    Err(_) => Ok(IndexValue::String(value.to_string())),
                }
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(IndexValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(IndexValue::Number(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(IndexValue::Number(value as i64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(IndexValue::Float(value))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut values = BTreeMap::new();

                while let Some((key, value)) = map.next_entry::<String, IndexValue>()? {
                    values.insert(key, value);
                }

                Ok(IndexValue::Map(values))
            }
        }

        deserializer.deserialize_any(IndexValueVisitor)
    }
}
