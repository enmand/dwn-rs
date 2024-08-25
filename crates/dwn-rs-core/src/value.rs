extern crate derive_more;
use derive_more::{Display, From, TryInto};

use std::collections::BTreeMap;

use chrono::{DateTime, SecondsFormat, Utc};
use ipld_core::cid::Cid;
use serde::ser::{SerializeMap, SerializeSeq};

/// Value represents a JSON-like value, that can be serialized and deserialized and
/// used in DWN to represent data frmo various sources, such as Messages. Values
/// are typed.
#[derive(Debug, Clone, From, TryInto, Display, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    String(String),
    Number(i64),
    Float(f64),
    Cid(Cid),
    #[display("{:?}", "_0")]
    Map(MapValue),
    #[display("{:?}", "_0")]
    Array(Vec<Value>),
    DateTime(DateTime<Utc>),
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_none(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::String(s) => serializer.serialize_str(s),
            Value::Cid(c) => serializer.serialize_str(&c.to_string()),
            Value::Number(n) => serializer.serialize_i64(*n),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::Map(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::DateTime(dt) => {
                serializer.serialize_str(&dt.to_rfc3339_opts(SecondsFormat::Micros, true))
            }
            Value::Array(a) => {
                let mut map = serializer.serialize_seq(Some(a.len()))?;
                for v in a.iter() {
                    map.serialize_element(v)?;
                }
                map.end()
            }
        }
    }
}

/// MapValue is a type alias for a BTreeMap of String to Value, used for map values
/// such as indexes.
pub type MapValue = BTreeMap<String, Value>;

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IndexValueVisitor;

        impl<'de> serde::de::Visitor<'de> for IndexValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid RFC3339 datetime or a valid Value variant")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match DateTime::parse_from_rfc3339(value) {
                    Ok(dt) => Ok(Value::DateTime(dt.with_timezone(&chrono::Utc))),
                    Err(_) => match ipld_core::cid::Cid::try_from(value) {
                        Ok(cid) => Ok(Value::Cid(cid)),
                        Err(_) => {
                            if value == "true" {
                                return Ok(Value::Bool(true));
                            } else if value == "false" {
                                return Ok(Value::Bool(false));
                            }

                            Ok(Value::String(value.to_string()))
                        }
                    },
                }
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Number(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Number(value as i64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Float(value))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut values = MapValue::default();

                while let Some((key, value)) = map.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Value::Map(values))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut values = Vec::new();

                while let Some(value) = seq.next_element::<Value>()? {
                    values.push(value);
                }

                Ok(Value::Array(values))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Null)
            }
        }

        deserializer.deserialize_any(IndexValueVisitor)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use chrono::{DateTime, SecondsFormat, Utc};
    use serde_json::json;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_value_serde() {
        struct TestCase {
            value: Value,
            json: serde_json::Value,
        }

        let message_timestamp = DateTime::from_str(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Micros, true)
                .as_str(),
        );

        let cases = vec![
            TestCase {
                value: Value::Null,
                json: json!(null),
            },
            TestCase {
                value: Value::Bool(true),
                json: json!(true),
            },
            TestCase {
                value: Value::Bool(false),
                json: json!(false),
            },
            TestCase {
                value: Value::String("foo".to_string()),
                json: json!("foo"),
            },
            TestCase {
                value: Value::Number(1),
                json: json!(1),
            },
            TestCase {
                value: Value::Float(1.0),
                json: json!(1.0),
            },
            TestCase {
                value: Value::DateTime(message_timestamp.unwrap()),
                json: json!(message_timestamp
                    .unwrap()
                    .to_rfc3339_opts(SecondsFormat::Micros, true)),
            },
            TestCase {
                value: Value::Cid(Cid::new_v1(0x71, cid::multihash::Multihash::default())),
                json: json!(Cid::new_v1(0x71, cid::multihash::Multihash::default()).to_string()),
            },
            TestCase {
                value: Value::Map(MapValue::default()),
                json: json!({}),
            },
            TestCase {
                value: Value::Array(vec![]),
                json: json!([]),
            },
        ];

        for tc in cases {
            tracing::debug!(?tc.value);
            assert_eq!(serde_json::to_value(&tc.value).unwrap(), tc.json);
            assert_eq!(serde_json::from_value::<Value>(tc.json).unwrap(), tc.value);
        }
    }

    #[test]
    #[traced_test]
    fn test_value_map_serde() {
        let map = MapValue::default();
        let json = json!({});
        assert_eq!(serde_json::to_value(&map).unwrap(), json);
        assert_eq!(serde_json::from_value::<MapValue>(json).unwrap(), map);
    }
}
