use from_variants::FromVariants;
use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Filters {
    pub(crate) filters: Vec<BTreeMap<String, Filter>>,
}

impl From<Vec<BTreeMap<String, Filter>>> for Filters {
    fn from(filters: Vec<BTreeMap<String, Filter>>) -> Self {
        Self {
            filters,
            ..Default::default()
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
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromVariants)]
#[serde(untagged)]
pub enum Filter {
    Equal(EqualFilter),
    Range(RangeFilter),
    OneOf(OneOfFilter),
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

#[derive(Clone, Debug, Serialize, Deserialize, FromVariants)]
#[serde(untagged)]
pub enum EqualFilter {
    String(String),
    Number(i64),
    Bool(bool),
}

impl Display for EqualFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EqualFilter::String(s) => write!(f, "\"{}\"", s),
            EqualFilter::Number(i) => write!(f, "{}", i),
            EqualFilter::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl From<&str> for EqualFilter {
    fn from(s: &str) -> Self {
        EqualFilter::String(s.into())
    }
}

pub type OneOfFilter = Vec<EqualFilter>;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RangeFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<RangeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<RangeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<RangeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<RangeValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromVariants)]
#[serde(untagged)]
pub enum RangeValue {
    String(String),
    Number(i64),
}

impl Display for RangeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeValue::String(s) => {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    write!(f, "\"{}\"", dt)
                } else {
                    write!(f, "{}", s)
                }
            }
            RangeValue::Number(i) => write!(f, "{}", i),
        }
    }
}

impl From<&str> for RangeValue {
    fn from(s: &str) -> Self {
        RangeValue::String(s.into())
    }
}
