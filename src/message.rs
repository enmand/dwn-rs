use std::collections::HashMap;

use jose_jws::General as JWS;
use serde::{Deserialize, Serialize};

use crate::IndexValue;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    pub descriptor: Descriptor,
    pub authroization: Option<JWS>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct Filters {
    filters: HashMap<String, Filter>,
}

impl From<HashMap<String, Filter>> for Filters {
    fn from(filters: HashMap<String, Filter>) -> Self {
        Self { filters }
    }
}

impl<const N: usize> From<[(String, Filter); N]> for Filters {
    fn from(filters: [(String, Filter); N]) -> Self {
        Self {
            filters: HashMap::from(filters),
        }
    }
}

impl<const N: usize> From<[(&str, Filter); N]> for Filters {
    fn from(filters: [(&str, Filter); N]) -> Self {
        Self {
            filters: filters
                .to_vec()
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect::<HashMap<String, Filter>>(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Filter {
    Equal(EqualFilter),
    Range(RangeFilter),
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

#[derive(Clone, Debug)]
pub enum EqualFilter {
    String(String),
    Number(i64),
    Bool(bool),
}
impl From<EqualFilter> for Filter {
    fn from(f: EqualFilter) -> Self {
        Filter::Equal(f)
    }
}

impl From<String> for EqualFilter {
    fn from(s: String) -> Self {
        EqualFilter::String(s)
    }
}

impl From<&str> for EqualFilter {
    fn from(s: &str) -> Self {
        EqualFilter::String(s.into())
    }
}

impl From<i64> for EqualFilter {
    fn from(i: i64) -> Self {
        EqualFilter::Number(i)
    }
}

impl From<bool> for EqualFilter {
    fn from(b: bool) -> Self {
        EqualFilter::Bool(b)
    }
}

pub type OneOfFilter = Vec<EqualFilter>;

#[derive(Clone, Debug)]
pub struct RangeFilter {
    lt: Option<LT>,
    gt: Option<GT>,
}

impl From<RangeFilter> for Filter {
    fn from(f: RangeFilter) -> Self {
        Filter::Range(f)
    }
}

#[derive(Clone, Debug)]
pub enum GT {
    GT(RangeValue),
    GTE(RangeValue),
}

impl From<RangeValue> for GT {
    fn from(v: RangeValue) -> Self {
        GT::GT(v)
    }
}

impl From<i64> for GT {
    fn from(i: i64) -> Self {
        GT::GT(i.into())
    }
}

#[derive(Clone, Debug)]
pub enum LT {
    LT(RangeValue),
    LTE(RangeValue),
}

impl From<RangeValue> for LT {
    fn from(v: RangeValue) -> Self {
        LT::LT(v)
    }
}

impl From<i64> for LT {
    fn from(i: i64) -> Self {
        LT::LT(i.into())
    }
}

#[derive(Clone, Debug)]
pub enum RangeValue {
    String(String),
    Number(i64),
}

impl From<RangeValue> for Filter {
    fn from(v: RangeValue) -> Self {
        Filter::Range(RangeFilter {
            lt: None,
            gt: Some(GT::GT(v)),
        })
    }
}

impl From<String> for RangeValue {
    fn from(s: String) -> Self {
        RangeValue::String(s)
    }
}

impl From<&str> for RangeValue {
    fn from(s: &str) -> Self {
        RangeValue::String(s.into())
    }
}

impl From<i64> for RangeValue {
    fn from(i: i64) -> Self {
        RangeValue::Number(i)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EncodedMessage {
    pub(super) encoded_message: Vec<u8>,
    pub(super) indexes: HashMap<String, IndexValue>,
}
