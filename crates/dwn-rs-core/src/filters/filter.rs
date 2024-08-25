use std::ops::Bound;

use serde::{Deserialize, Serialize};

use crate::value::Value;

pub type RangeFilter<T> = (Bound<T>, Bound<T>);

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Filter<T> {
    Equal(T),
    #[serde(with = "range_filter_serializer")]
    Range((Bound<T>, Bound<T>)),
    OneOf(Vec<T>),
    #[serde(with = "prefix_filter_serializer")]
    Prefix(T),
}

pub mod prefix_filter_serializer {
    use serde::ser::{Serialize, SerializeMap, Serializer};

    pub fn serialize<S, T>(prefix_filter: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("prefix", prefix_filter)?;
        map.end()
    }

    #[cfg(test)]
    mod test {
        use super::super::Filter;

        struct Test {
            prefix_filter: Filter<String>,
            json: serde_json::Value,
        }

        #[test]
        fn test_serialize_prefix_filter() {
            let tests = vec![
                Test {
                    prefix_filter: Filter::Prefix("test".to_string()),
                    json: serde_json::json!({
                        "prefix": "test",
                    }),
                },
                Test {
                    prefix_filter: Filter::Prefix("".to_string()),
                    json: serde_json::json!({
                        "prefix": "",
                    }),
                },
            ];

            for test in tests {
                assert_eq!(serde_json::to_value(test.prefix_filter).unwrap(), test.json);
            }
        }
    }
}

pub mod range_filter_serializer {
    use super::*;
    use serde::ser::{Serialize, SerializeMap, Serializer};

    pub fn serialize<S, T>(
        range_filter: &(Bound<T>, Bound<T>),
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let mut map = serializer.serialize_map(Some(2))?;

        if let Bound::Included(ref v) = range_filter.0 {
            map.serialize_entry("gte", v)?;
        } else if let Bound::Excluded(ref v) = range_filter.0 {
            map.serialize_entry("gt", v)?;
        }

        if let Bound::Included(ref v) = range_filter.1 {
            map.serialize_entry("lte", v)?;
        } else if let Bound::Excluded(ref v) = range_filter.1 {
            map.serialize_entry("lt", v)?;
        }

        map.end()
    }

    pub fn serialize_optional<S, T>(
        range_filter: &Option<(Bound<T>, Bound<T>)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match range_filter {
            Some(range) => serialize(range, serializer),
            None => serializer.serialize_none(),
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use serde_json::json;

        #[test]
        fn test_serialize_range_filter() {
            struct Test {
                range_filter: Filter<i32>,
                json: serde_json::Value,
            }

            let tests = vec![
                Test {
                    range_filter: Filter::Range((Bound::Included(1), Bound::Excluded(2))),
                    json: json!({
                        "gte": 1,
                        "lt": 2,
                    }),
                },
                Test {
                    range_filter: Filter::Range((Bound::<i32>::Unbounded, Bound::Excluded(2))),
                    json: json!({
                        "lt": 2,
                    }),
                },
                Test {
                    range_filter: Filter::Range((Bound::Included(1), Bound::<i32>::Unbounded)),
                    json: json!({
                        "gte": 1,
                    }),
                },
            ];

            for test in tests {
                assert_eq!(serde_json::to_value(test.range_filter).unwrap(), test.json);
            }
        }

        #[test]
        fn test_serialize_optional_range_filter() {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum SomeFilter {
                #[serde(serialize_with = "serialize_optional")]
                MaybeRange(Option<RangeFilter<i32>>),
            }

            struct Test {
                range_filter: SomeFilter,
                json: serde_json::Value,
            }

            let tests = vec![
                Test {
                    range_filter: SomeFilter::MaybeRange(Some((
                        Bound::Included(1),
                        Bound::Excluded(2),
                    ))),
                    json: json!({
                        "gte": 1,
                        "lt": 2,
                    }),
                },
                Test {
                    range_filter: SomeFilter::MaybeRange(Some((
                        Bound::<i32>::Unbounded,
                        Bound::Excluded(2),
                    ))),
                    json: json!({
                        "lt": 2,
                    }),
                },
                Test {
                    range_filter: SomeFilter::MaybeRange(Some((
                        Bound::Included(1),
                        Bound::<i32>::Unbounded,
                    ))),
                    json: json!({
                        "gte": 1,
                    }),
                },
                Test {
                    range_filter: SomeFilter::MaybeRange(None),
                    json: json!(null),
                },
            ];

            for test in tests {
                assert_eq!(serde_json::to_value(test.range_filter).unwrap(), test.json);
            }
        }
    }
}

struct FilterVisitor;

impl<'de> serde::de::Visitor<'de> for FilterVisitor {
    type Value = Filter<Value>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expected a value, or a JSON object with eq, range, or oneOf")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Filter<Value>, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut range = (Bound::Unbounded, Bound::Unbounded);
        let mut prefix_value: Option<Value> = None;
        let mut has_range_key = false;

        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            match key.as_str() {
                "lt" | "lte" | "gt" | "gte" => {
                    has_range_key = true;

                    match key.as_str() {
                        "lt" => match range.1 {
                            Bound::Unbounded => range.1 = Bound::Excluded(value),
                            _ => return Err(serde::de::Error::custom("multiple upper bounds")),
                        },
                        "lte" => match range.1 {
                            Bound::Unbounded => range.1 = Bound::Included(value),
                            _ => return Err(serde::de::Error::custom("multiple upper bounds")),
                        },
                        "gt" => match range.0 {
                            Bound::Unbounded => range.0 = Bound::Excluded(value),
                            _ => return Err(serde::de::Error::custom("multiple lower bounds")),
                        },
                        "gte" => match range.0 {
                            Bound::Unbounded => range.0 = Bound::Included(value),
                            _ => return Err(serde::de::Error::custom("multiple lower bounds")),
                        },
                        _ => {} // Already checked by outer match
                    }
                }
                "prefix" => {
                    if has_range_key {
                        return Err(serde::de::Error::custom(
                            "cannot provide both 'prefix' and range keys",
                        ));
                    }
                    prefix_value = Some(value);
                }
                _ => return Err(serde::de::Error::custom("invalid range key")),
            }
        }

        if let Some(value) = prefix_value {
            return Ok(Filter::Prefix(value));
        }

        Ok(Filter::Range((range.0, range.1)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Filter<Value>, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Filter::OneOf(values))
    }

    fn visit_str<E>(self, value: &str) -> Result<Filter<Value>, E>
    where
        E: serde::de::Error,
    {
        match chrono::DateTime::parse_from_rfc3339(value) {
            Ok(dt) => Ok(Filter::Equal(Value::String(
                dt.with_timezone(&chrono::Utc).to_rfc3339(),
            ))),
            Err(_) => {
                if value == "true" || value == "false" {
                    Ok(Filter::Equal(Value::Bool(
                        value.parse::<bool>().map_err(serde::de::Error::custom)?,
                    )))
                } else {
                    Ok(Filter::Equal(Value::String(value.to_string())))
                }
            }
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Filter<Value>, E>
    where
        E: serde::de::Error,
    {
        Ok(Filter::Equal(Value::Number(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Filter<Value>, E>
    where
        E: serde::de::Error,
    {
        Ok(Filter::Equal(Value::Number(value as i64)))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Filter::Equal(Value::Number(v as i64)))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Filter::Equal(Value::Number(v as i64)))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Filter<Value>, E>
    where
        E: serde::de::Error,
    {
        Ok(Filter::Equal(Value::Bool(value)))
    }
}

impl<'de> Deserialize<'de> for Filter<Value> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(FilterVisitor)
    }
}

impl<T: Default + TryFrom<Value>> From<Filter<Value>> for Filter<T> {
    fn from(f: Filter<Value>) -> Self {
        match f {
            Filter::Equal(v) => Filter::Equal(v.try_into().unwrap_or_default()),
            Filter::Range((beg, end)) => Filter::Range((
                match beg {
                    Bound::Included(v) => Bound::Included(v.try_into().unwrap_or_default()),
                    Bound::Excluded(v) => Bound::Excluded(v.try_into().unwrap_or_default()),
                    _ => Bound::Unbounded,
                },
                match end {
                    Bound::Included(v) => Bound::Included(v.try_into().unwrap_or_default()),
                    Bound::Excluded(v) => Bound::Excluded(v.try_into().unwrap_or_default()),
                    _ => Bound::Unbounded,
                },
            )),
            Filter::OneOf(v) => Filter::OneOf(
                v.into_iter()
                    .map(|v| v.try_into().unwrap_or_default())
                    .collect(),
            ),
            Filter::Prefix(v) => Filter::Prefix(v.try_into().unwrap_or_default()),
        }
    }
}

impl<T: Into<String>> From<T> for Filter<String> {
    fn from(value: T) -> Self {
        Filter::Equal(value.into())
    }
}

impl From<i64> for Filter<i64> {
    fn from(i: i64) -> Self {
        Filter::Equal(i)
    }
}

impl From<bool> for Filter<bool> {
    fn from(b: bool) -> Self {
        Filter::Equal(b)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_filter() {
        struct Test {
            filter: Filter<Value>,
            json: serde_json::Value,
        }

        let tests = vec![
            Test {
                filter: Filter::Equal(Value::String("test".to_string())),
                json: json!("test"),
            },
            Test {
                filter: Filter::Equal(Value::Number(42)),
                json: json!(42),
            },
            Test {
                filter: Filter::Equal(Value::Bool(true)),
                json: json!(true),
            },
            Test {
                filter: Filter::Range((
                    Bound::Included(Value::Number(1)),
                    Bound::Excluded(Value::Number(2)),
                )),
                json: json!({
                    "gte": 1,
                    "lt": 2,
                }),
            },
            Test {
                filter: Filter::OneOf(vec![Value::String("test".to_string()), Value::Number(42)]),
                json: json!(["test", 42]),
            },
            Test {
                filter: Filter::Prefix(Value::String("test".to_string())),
                json: json!({
                    "prefix": "test",
                }),
            },
        ];

        for test in tests {
            assert_eq!(serde_json::to_value(&test.filter).unwrap(), test.json);
        }
    }

    #[test]
    fn test_deserialize_filter() {
        struct Test {
            json: serde_json::Value,
            filter: Filter<Value>,
        }

        let tests = vec![
            Test {
                json: json!("test"),
                filter: Filter::Equal(Value::String("test".to_string())),
            },
            Test {
                json: json!(42),
                filter: Filter::Equal(Value::Number(42)),
            },
            Test {
                json: json!(true),
                filter: Filter::Equal(Value::Bool(true)),
            },
            Test {
                json: json!({
                    "gte": 1,
                    "lt": 2,
                }),
                filter: Filter::Range((
                    Bound::Included(Value::Number(1)),
                    Bound::Excluded(Value::Number(2)),
                )),
            },
            Test {
                json: json!(["test", "ing"]),
                filter: Filter::OneOf(vec![
                    Value::String("test".to_string()),
                    Value::String("ing".to_string()),
                ]),
            },
            Test {
                json: json!({
                    "prefix": "test",
                }),
                filter: Filter::Prefix(Value::String("test".to_string())),
            },
        ];

        for test in tests {
            assert_eq!(
                serde_json::from_value::<Filter<Value>>(test.json.clone()).unwrap(),
                test.filter
            );
        }

        let invalid_tests = vec![json!(42.0), json!(null), json!({ "invalid": "test" })];

        for test in invalid_tests {
            assert!(serde_json::from_value::<Filter<Value>>(test).is_err());
        }

        let invalid_range_tests = vec![
            json!({ "lt": 2, "prefix": "test" }),
            json!({ "bad": false }),
        ];

        for test in invalid_range_tests {
            assert!(serde_json::from_value::<Filter<Value>>(test).is_err());
        }
    }

    #[test]
    fn test_filter_into() {
        let filter = Filter::Equal(Value::String("test".to_string()));
        assert_eq!(
            Filter::<String>::from(filter),
            Filter::Equal("test".to_string())
        );

        let filter = Filter::Equal(Value::Number(42));
        assert_eq!(Filter::<i64>::from(filter), Filter::Equal(42));

        let filter = Filter::Equal(Value::Bool(true));
        assert_eq!(Filter::<bool>::from(filter), Filter::Equal(true));

        let filter = Filter::Range((
            Bound::Included(Value::Number(1)),
            Bound::Excluded(Value::Number(2)),
        ));
        assert_eq!(
            Filter::<i64>::from(filter),
            Filter::Range((Bound::Included(1), Bound::Excluded(2)))
        );

        let filter = Filter::OneOf(vec![
            Value::String("test".to_string()),
            Value::String(42.to_string()),
        ]);
        assert_eq!(
            Filter::<String>::from(filter),
            Filter::OneOf(vec!["test".to_string(), 42.to_string()])
        );

        let filter = Filter::Prefix(Value::String("test".to_string()));
        assert_eq!(
            Filter::<String>::from(filter),
            Filter::Prefix("test".to_string())
        );
    }
}
