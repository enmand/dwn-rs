use std::ops::Bound;

use serde::{Deserialize, Serialize};

use crate::value::Value;

pub type RangeFilter<T> = (Bound<T>, Bound<T>);

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Filter<T> {
    Equal(T),
    #[serde(with = "range_filter_serializer")]
    Range((Bound<T>, Bound<T>)),
    OneOf(Vec<T>),
    Prefix(T),
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
