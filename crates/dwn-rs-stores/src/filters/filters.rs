use std::{collections::BTreeMap, ops::Bound};

use serde::{ser::SerializeMap, Deserialize, Serialize};

use dwn_rs_core::value::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Filters {
    pub(crate) filters: Vec<BTreeMap<String, Filter>>,
}

impl From<Vec<BTreeMap<String, Filter>>> for Filters {
    fn from(filters: Vec<BTreeMap<String, Filter>>) -> Self {
        Self { filters }
    }
}

impl PartialEq for Filters {
    fn eq(&self, other: &Self) -> bool {
        self.filters.len() == other.filters.len()
            && self
                .filters
                .iter()
                .zip(other.filters.iter())
                .all(|(a, b)| a.len() == b.len())
    }
}

impl<const N: usize, const M: usize, S, T> From<[[(S, T); N]; M]> for Filters
where
    S: Into<String> + Clone,
    T: Into<Filter> + Clone,
{
    fn from(filters: [[(S, T); N]; M]) -> Self {
        Self {
            filters: filters
                .iter()
                .map(|f| {
                    f.iter()
                        .map(|(k, v)| (k.clone().into(), v.clone().into()))
                        .collect::<BTreeMap<String, Filter>>()
                })
                .collect::<Vec<BTreeMap<String, Filter>>>(),
        }
    }
}

impl IntoIterator for Filters {
    type Item = BTreeMap<String, Filter>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.filters.into_iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Equal(Value),
    Range(Bound<Value>, Bound<Value>),
    OneOf(Vec<Value>),
}

impl<'de> Deserialize<'de> for Filter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FilterVisitor;

        impl<'de> serde::de::Visitor<'de> for FilterVisitor {
            type Value = Filter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expected a value, or a JSON object with eq, range, or oneOf")
            }

            fn visit_str<E>(self, value: &str) -> Result<Filter, E>
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

            fn visit_i64<E>(self, value: i64) -> Result<Filter, E>
            where
                E: serde::de::Error,
            {
                Ok(Filter::Equal(Value::Number(value)))
            }

            fn visit_bool<E>(self, value: bool) -> Result<Filter, E>
            where
                E: serde::de::Error,
            {
                Ok(Filter::Equal(Value::Bool(value)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Filter, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut range = (Bound::Unbounded, Bound::Unbounded);

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "lt" => match range.1 {
                            Bound::Unbounded => range.1 = Bound::Excluded(map.next_value()?),
                            _ => return Err(serde::de::Error::custom("multiple upper bounds")),
                        },
                        "lte" => match range.1 {
                            Bound::Unbounded => range.1 = Bound::Included(map.next_value()?),
                            _ => return Err(serde::de::Error::custom("multiple upper bounds")),
                        },
                        "gt" => match range.0 {
                            Bound::Unbounded => range.0 = Bound::Excluded(map.next_value()?),
                            _ => return Err(serde::de::Error::custom("multiple lower bounds")),
                        },
                        "gte" => match range.0 {
                            Bound::Unbounded => range.0 = Bound::Included(map.next_value()?),
                            _ => return Err(serde::de::Error::custom("multiple lower bounds")),
                        },
                        _ => return Err(serde::de::Error::custom("invalid range key")),
                    }
                }

                Ok(Filter::Range(range.0, range.1))
            }
        }
        deserializer.deserialize_any(FilterVisitor)
    }
}

impl Serialize for Filter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Filter::Equal(v) => v.serialize(serializer),
            Filter::Range(beg, end) => {
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
        }
    }
}

impl From<&str> for Filter {
    fn from(s: &str) -> Self {
        Filter::Equal(s.into())
    }
}

impl From<String> for Filter {
    fn from(s: String) -> Self {
        Filter::Equal(s.into())
    }
}

impl From<i64> for Filter {
    fn from(i: i64) -> Self {
        Filter::Equal(i.into())
    }
}

impl From<bool> for Filter {
    fn from(b: bool) -> Self {
        Filter::Equal(b.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filters_from() {
        struct TestCase {
            input: Filters,
            output: Filters,
        }

        let clean = &Filters {
            filters: vec![vec![("foo".to_string(), Filter::Equal(Value::Number(1)))]
                .into_iter()
                .collect()],
        };

        let tcs = vec![
            TestCase {
                input: vec![BTreeMap::from([(
                    "foo".to_string(),
                    Filter::Equal(Value::Number(1)),
                )])]
                .into(),
                output: clean.clone(),
            },
            TestCase {
                input: [[("foo", Filter::Equal(Value::Number(1)))]].into(),
                output: clean.clone(),
            },
        ];

        for tc in tcs {
            assert_eq!(tc.input, tc.output);
        }
    }

    #[test]
    fn test_filters_into_iter() {
        let filters = Filters {
            filters: vec![
                vec![
                    ("foo".to_string(), Filter::Equal(Value::Number(1))),
                    ("bar".to_string(), Filter::Equal(Value::Number(2))),
                ]
                .into_iter()
                .collect(),
                vec![("baz".to_string(), Filter::Equal(Value::Number(3)))]
                    .into_iter()
                    .collect(),
            ],
        };

        let mut iter = filters.into_iter();

        assert_eq!(
            iter.next(),
            Some(
                vec![
                    ("foo".to_string(), Filter::Equal(Value::Number(1))),
                    ("bar".to_string(), Filter::Equal(Value::Number(2))),
                ]
                .into_iter()
                .collect()
            )
        );

        assert_eq!(
            iter.next(),
            Some(
                vec![("baz".to_string(), Filter::Equal(Value::Number(3)))]
                    .into_iter()
                    .collect()
            )
        );

        assert_eq!(iter.next(), None);
    }
}
