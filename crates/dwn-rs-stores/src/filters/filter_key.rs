use std::{collections::BTreeMap, fmt::Display};

use dwn_rs_core::{Filter, Value};
use serde::{Deserialize, Serialize};

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
            FilterKey::Index(s) => (FilterKey::Index(s.clone()), format!("{}_{}_idx", s, alias)),
            FilterKey::Tag(s) => (FilterKey::Tag(s.clone()), format!("{}_{}_tag", s, alias)),
        }
    }

    pub fn count_set(&self, i: usize) -> Alias {
        self.alias(&i.to_string())
    }
}

// FilterKey represents the key-type to filter over. Currently, this can be an index or a tag.
pub type Alias = (FilterKey, String);

// ValueFilter is a helper type that represents the filter types, and the application of that
// filter itself.
pub type ValueFilter<K> = BTreeMap<K, Filter<Value>>;

// FilterSet is a set of fitlers across indexes and tags. Multiple filters can be applied.
pub type FilterSet<K> = Vec<ValueFilter<K>>;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Filters {
    pub(crate) set: FilterSet<FilterKey>,
}

impl From<Filters> for FilterSet<Alias> {
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
            .collect::<FilterSet<Alias>>()
    }
}

impl From<FilterSet<FilterKey>> for Filters {
    fn from(set: FilterSet<FilterKey>) -> Self {
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
    T: Into<Filter<Value>> + Clone,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filters_from() {
        struct TestCase {
            input: ValueFilter<FilterKey>,
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
                input: vec![(
                    FilterKey::Index("foo".into()),
                    Filter::Equal(Value::Number(1)),
                )]
                .into_iter()
                .collect(),
                output: clean.clone(),
            },
            TestCase {
                input: vec![(
                    FilterKey::Index("foo".into()),
                    Filter::Equal(Value::Number(1)),
                )]
                .into_iter()
                .collect(),
                output: clean.clone(),
            },
            TestCase {
                input: vec![(
                    FilterKey::Index("foo".into()),
                    Filter::Equal(Value::Number(1)),
                )]
                .into_iter()
                .collect(),
                output: clean.clone(),
            },
        ];

        for tc in tcs {
            assert_eq!(Into::<Filters>::into(tc.input), tc.output);
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
