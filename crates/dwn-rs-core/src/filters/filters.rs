use std::ops::Bound;

use serde::{ser::SerializeMap, Deserialize, Serialize};

use crate::value::Value;

pub type RangeFilter<T> = (Bound<T>, Bound<T>);

#[derive(Clone, Debug, PartialEq)]
pub enum Filter<T> {
    Equal(T),
    Range(RangeFilter<T>),
    OneOf(Vec<T>),
    Prefix(T),
}

struct FilterVisitor;

impl<'de> serde::de::Visitor<'de> for FilterVisitor {
    type Value = Filter<Value>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expected a value, or a JSON object with eq, range, or oneOf")
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
                    return Ok(Filter::Equal(Value::Bool(
                        value.parse::<bool>().map_err(serde::de::Error::custom)?,
                    )));
                }

                Ok(Filter::Equal(Value::String(value.to_string())))
            }
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Filter<Value>, E>
    where
        E: serde::de::Error,
    {
        Ok(Filter::Equal(Value::Number(value)))
    }

    fn visit_bool<E>(self, value: bool) -> Result<Filter<Value>, E>
    where
        E: serde::de::Error,
    {
        Ok(Filter::Equal(Value::Bool(value)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Filter<Value>, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut range = (Bound::Unbounded, Bound::Unbounded);

        while let Some((key, value)) = map.next_entry::<String, Value>()? {
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
                "prefix" => {
                    return Ok(Filter::Prefix(value));
                }
                _ => return Err(serde::de::Error::custom("invalid range key")),
            }
        }

        Ok(Filter::Range((range.0, range.1)))
    }
}

impl<'de> Deserialize<'de> for Filter<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(FilterVisitor)
            .map(Filter::from)
    }
}

impl<'de> Deserialize<'de> for Filter<i64> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(FilterVisitor)
            .map(Filter::from)
    }
}

impl<'de> Deserialize<'de> for Filter<bool> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer
            .deserialize_any(FilterVisitor)
            .map(Filter::from)
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

impl<T> Serialize for Filter<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Filter::Equal(v) => v.serialize(serializer),
            Filter::Range((beg, end)) => {
                let mut map = serializer.serialize_map(Some(2))?;

                match beg {
                    Bound::Included(v) => {
                        map.serialize_entry("gte", v)?;
                    }
                    Bound::Excluded(v) => {
                        map.serialize_entry("gt", v)?;
                    }
                    _ => {}
                }

                match end {
                    Bound::Included(v) => {
                        map.serialize_entry("lte", v)?;
                    }
                    Bound::Excluded(v) => {
                        map.serialize_entry("lt", v)?;
                    }
                    _ => {}
                }

                map.end()
            }
            Filter::OneOf(v) => v.serialize(serializer),
            Filter::Prefix(v) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("prefix", v)?;
                map.end()
            }
        }
    }
}

impl From<Filter<Value>> for Filter<String> {
    fn from(f: Filter<Value>) -> Self {
        match f {
            Filter::Equal(v) => Filter::Equal(v.to_string()),
            Filter::Range((beg, end)) => Filter::Range((
                match beg {
                    Bound::Included(v) => Bound::Included(v.to_string()),
                    Bound::Excluded(v) => Bound::Excluded(v.to_string()),
                    _ => Bound::Unbounded,
                },
                match end {
                    Bound::Included(v) => Bound::Included(v.to_string()),
                    Bound::Excluded(v) => Bound::Excluded(v.to_string()),
                    _ => Bound::Unbounded,
                },
            )),
            Filter::OneOf(v) => Filter::OneOf(v.into_iter().map(|v| v.to_string()).collect()),
            Filter::Prefix(v) => Filter::Prefix(v.to_string()),
        }
    }
}

impl From<Filter<Value>> for Filter<i64> {
    fn from(f: Filter<Value>) -> Self {
        match f {
            Filter::Equal(v) => Filter::Equal(v.try_into().unwrap_or(0)),
            Filter::Range((beg, end)) => Filter::Range((
                match beg {
                    Bound::Included(v) => Bound::Included(v.try_into().unwrap_or(0)),
                    Bound::Excluded(v) => Bound::Excluded(v.try_into().unwrap_or(0)),
                    _ => Bound::Unbounded,
                },
                match end {
                    Bound::Included(v) => Bound::Included(v.try_into().unwrap_or(0)),
                    Bound::Excluded(v) => Bound::Excluded(v.try_into().unwrap_or(0)),
                    _ => Bound::Unbounded,
                },
            )),
            Filter::OneOf(v) => {
                Filter::OneOf(v.into_iter().map(|v| v.try_into().unwrap_or(0)).collect())
            }
            Filter::Prefix(v) => Filter::Prefix(v.try_into().unwrap_or(0)),
        }
    }
}

impl From<Filter<Value>> for Filter<bool> {
    fn from(value: Filter<Value>) -> Self {
        match value {
            Filter::Equal(v) => Filter::Equal(v.try_into().unwrap_or(false)),
            Filter::Range(_) => Filter::Equal(false),
            Filter::OneOf(v) => Filter::OneOf(
                v.into_iter()
                    .map(|v| v.try_into().unwrap_or(false))
                    .collect(),
            ),
            Filter::Prefix(_) => Filter::Equal(false),
        }
    }
}

impl From<&str> for Filter<String> {
    fn from(s: &str) -> Self {
        Filter::Equal(s.into())
    }
}

impl From<String> for Filter<String> {
    fn from(s: String) -> Self {
        Filter::Equal(s)
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
