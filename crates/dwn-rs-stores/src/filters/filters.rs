use std::{collections::BTreeMap, fmt::Display, ops::Bound};

use serde::{ser::SerializeMap, Deserialize, Serialize};

use dwn_rs_core::value::Value;

// FilterKey represents the key-type to filter over. Currently, this can be an index or a tag.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterKey {
    Index(String),
    Tag(String),
}

impl Display for FilterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FilterKey::Index(s) => write!(f, "{}", s),
            FilterKey::Tag(s) => write!(f, "{}", s),
        }
    }
}

impl FilterKey {
    pub fn alias(&self, alias: &str) -> Alias {
        match self {
            FilterKey::Index(s) => (FilterKey::Index(s.clone()), format!("{}_{}", s, alias)),
            FilterKey::Tag(s) => (FilterKey::Tag(s.clone()), format!("{}_{}", s, alias)),
        }
    }

    pub fn count_set(&self, i: usize) -> Alias {
        self.alias(&i.to_string())
    }
}

// ValueFilter is a helper type that represents the filter types, and the application of that
// filter itself.
pub type ValueFilter<K> = BTreeMap<K, Filter>;

// FilterSet is a set of fitlers across indexes and tags. Multiple filters can be applied.
pub type Set<K> = Vec<ValueFilter<K>>;

// FilterKey represents the key-type to filter over. Currently, this can be an index or a tag.
pub type Alias = (FilterKey, String);

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Filters {
    pub(crate) set: Set<FilterKey>,
}

impl From<Filters> for Set<Alias> {
    fn from(filters: Filters) -> Self {
        filters
            .set
            .iter()
            .enumerate()
            .map(|(i, k)| {
                k.iter()
                    .map(|(k, v)| (k.count_set(i), v.clone()))
                    .collect::<ValueFilter<Alias>>()
            })
            .collect::<Set<Alias>>()
    }
}

impl From<Set<FilterKey>> for Filters {
    fn from(set: Set<FilterKey>) -> Self {
        Self { set }
    }
}

impl From<ValueFilter<FilterKey>> for Filters {
    fn from(filter: ValueFilter<FilterKey>) -> Self {
        Self { set: vec![filter] }
    }
}

impl PartialEq for Filters {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

impl<const N: usize, const M: usize, T> From<[[(FilterKey, T); N]; M]> for Filters
where
    T: Into<Filter> + Clone,
{
    fn from(filters: [[(FilterKey, T); N]; M]) -> Self {
        Self {
            set: filters
                .iter()
                .map(|f| {
                    f.iter()
                        .map(|(k, v)| (k.clone(), v.clone().into()))
                        .collect::<ValueFilter<FilterKey>>()
                })
                .collect::<Vec<ValueFilter<FilterKey>>>(),
        }
    }
}

impl IntoIterator for Filters {
    type Item = ValueFilter<FilterKey>;
    type IntoIter = std::vec::IntoIter<ValueFilter<FilterKey>>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.into_iter()
    }
}

impl FromIterator<Filters> for Filters {
    fn from_iter<I: IntoIterator<Item = Filters>>(iter: I) -> Self {
        let mut set = Vec::new();

        for filters in iter {
            set.extend(filters.set);
        }

        Self { set }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Equal(Value),
    Range(Bound<Value>, Bound<Value>),
    OneOf(Vec<Value>),
    Prefix(Value),
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
            Filter::Prefix(v) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("prefix", v)?;
                map.end()
            }
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

        let clean: Filters = vec![vec![(
            FilterKey::Index("foo".into()),
            Filter::Equal(Value::Number(1)),
        )]
        .into_iter()
        .collect()]
        .into();

        let tcs = vec![
            TestCase {
                input: vec![BTreeMap::from([(
                    FilterKey::Index("foo".into()),
                    Filter::Equal(Value::Number(1)),
                )])]
                .into(),
                output: clean.clone(),
            },
            TestCase {
                input: [[(
                    FilterKey::Index("foo".into()),
                    Filter::Equal(Value::Number(1)),
                )]]
                .into(),
                output: clean.clone(),
            },
        ];

        for tc in tcs {
            assert_eq!(tc.input, tc.output);
        }
    }

    #[test]
    fn test_filters_into_iter() {
        let filters: Filters = vec![
            vec![
                (
                    FilterKey::Index("foo".into()),
                    Filter::Equal(Value::Number(1)),
                ),
                (
                    FilterKey::Index("bar".into()),
                    Filter::Equal(Value::Number(2)),
                ),
            ]
            .into_iter()
            .collect(),
            vec![(
                FilterKey::Index("baz".into()),
                Filter::Equal(Value::Number(3)),
            )]
            .into_iter()
            .collect(),
        ]
        .into();

        let mut iter = filters.into_iter();

        assert_eq!(
            iter.next(),
            Some(
                vec![
                    (
                        FilterKey::Index("foo".into()),
                        Filter::Equal(Value::Number(1))
                    ),
                    (
                        FilterKey::Index("bar".into()),
                        Filter::Equal(Value::Number(2))
                    ),
                ]
                .into_iter()
                .collect()
            )
        );

        assert_eq!(
            iter.next(),
            Some(
                vec![(
                    FilterKey::Index("baz".into()),
                    Filter::Equal(Value::Number(3))
                )]
                .into_iter()
                .collect()
            )
        );

        assert_eq!(iter.next(), None);
    }
}
