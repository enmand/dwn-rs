use from_variants::FromVariants;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub struct Filters {
    pub(crate) filters: HashMap<String, Filter>,
}

impl From<HashMap<String, Filter>> for Filters {
    fn from(filters: HashMap<String, Filter>) -> Self {
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
                .collect::<HashMap<String, Filter>>(),
        }
    }
}

#[derive(Clone, Debug, FromVariants)]
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

#[derive(Clone, Debug, FromVariants)]
pub enum EqualFilter {
    String(String),
    Number(i64),
    Bool(bool),
}

impl Display for EqualFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EqualFilter::String(s) => write!(f, "{}", s),
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

#[derive(Clone, Debug)]
pub struct RangeFilter {
    lt: Option<LT>,
    gt: Option<GT>,
}

impl Display for RangeFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.lt, &self.gt) {
            (Some(lt), None) => write!(f, "{}", lt),
            (None, Some(gt)) => write!(f, "{}", gt),
            (Some(lt), Some(gt)) => write!(f, "{} x {}", lt, gt),
            (None, None) => write!(f, ""),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GT {
    GT(RangeValue),
    GTE(RangeValue),
}

impl Display for GT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GT::GT(v) => write!(f, "< {}", v),
            GT::GTE(v) => write!(f, "<= {}", v),
        }
    }
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

impl From<String> for GT {
    fn from(s: String) -> Self {
        GT::GT(s.into())
    }
}

impl From<GT> for RangeFilter {
    fn from(gt: GT) -> Self {
        Self {
            lt: None,
            gt: Some(gt),
        }
    }
}

impl From<GT> for Filter {
    fn from(gt: GT) -> Self {
        Filter::Range(RangeFilter::from(gt))
    }
}

#[derive(Clone, Debug)]
pub enum LT {
    LT(RangeValue),
    LTE(RangeValue),
}

impl Display for LT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LT::LT(v) => write!(f, "> {}", v),
            LT::LTE(v) => write!(f, ">= {}", v),
        }
    }
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

impl From<String> for LT {
    fn from(s: String) -> Self {
        LT::LT(s.into())
    }
}

impl From<LT> for RangeFilter {
    fn from(lt: LT) -> Self {
        Self {
            lt: Some(lt),
            gt: None,
        }
    }
}

impl From<LT> for Filter {
    fn from(lt: LT) -> Self {
        Filter::Range(RangeFilter::from(lt))
    }
}

#[derive(Clone, Debug, FromVariants)]
pub enum RangeValue {
    String(String),
    Number(i64),
}

impl Display for RangeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeValue::String(s) => write!(f, "{}", s),
            RangeValue::Number(i) => write!(f, "{}", i),
        }
    }
}

impl From<RangeValue> for Filter {
    fn from(v: RangeValue) -> Self {
        Filter::Range(RangeFilter {
            lt: None,
            gt: Some(GT::GT(v)),
        })
    }
}

impl From<&str> for RangeValue {
    fn from(s: &str) -> Self {
        RangeValue::String(s.into())
    }
}
