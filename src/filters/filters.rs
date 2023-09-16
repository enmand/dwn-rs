use from_variants::FromVariants;
use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Filters {
    pub(crate) filters: BTreeMap<String, Filter>,
}

impl From<BTreeMap<String, Filter>> for Filters {
    fn from(filters: BTreeMap<String, Filter>) -> Self {
        Self { filters }
    }
}

impl<const N: usize, S, T> From<[(S, T); N]> for Filters
where
    S: Into<String> + Clone,
    T: Into<Filter> + Clone,
{
    fn from(filters: [(S, T); N]) -> Self {
        Self {
            filters: filters
                .iter()
                .map(|(k, v)| ((k.clone().into(), v.clone().into())))
                .collect::<BTreeMap<String, Filter>>(),
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
    lt: Option<RangeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gt: Option<RangeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lte: Option<RangeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gte: Option<RangeValue>,
}

impl RangeFilter {
    pub fn range_with(&self, key: &String) -> String {
        let mut s = String::new();

        if let Some(lt) = &self.lt {
            s.push_str(&format!("{} < {}", key, lt));
        } else if let Some(gt) = &self.gt {
            s.push_str(&format!("{} > {}", key, gt));
        }

        if let Some(lte) = &self.lte {
            if self.gt.is_some() || self.lt.is_some() {
                s.push_str(" AND ");
            }

            s.push_str(&format!("{} <= {}", key, lte));
        }

        if let Some(gte) = &self.gte {
            if self.gt.is_some() || self.lt.is_some() {
                s.push_str(" AND ");
            }
            s.push_str(&format!("{} >= {}", key, gte));
        }
    }
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
                if chrono::DateTime::parse_from_rfc3339(s).is_ok() {
                    write!(f, "\"{}\"", s)
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
