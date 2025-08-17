use std::future::Future;

use crate::value::Value;
use ipld_core::cid::Cid;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{serde_as, DisplayFromStr};

use crate::filters::{errors, Filters};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Pagination {
    pub cursor: Option<Cursor>,
    pub limit: Option<u64>,
}

impl Pagination {
    pub fn new(cursor: Option<Cursor>, limit: Option<u64>) -> Self {
        Self { cursor, limit }
    }

    pub fn with_limit(limit: u64) -> Self {
        Self {
            cursor: None,
            limit: Some(limit),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Cursor {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "messageCid")]
    pub cursor: Cid,
    pub value: Option<Value>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone, Default)]
#[repr(i8)]
pub enum SortDirection {
    #[default]
    Ascending = 1,
    Descending = -1,
}

impl SortDirection {
    pub fn to_bool(&self) -> bool {
        match self {
            SortDirection::Ascending => true,
            SortDirection::Descending => false,
        }
    }
}

/// Directional is a trait that allows for the retrieval of the direction of a type.
pub trait Directional {
    fn get_direction(&self) -> &SortDirection;
}

/// Ordorable is a trait that allows for the conversion of a type into an Order.
pub trait Ordorable {
    fn to_order<'a>(self) -> Vec<(&'a str, bool)>;
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct NoSort;

impl Default for NoSort {
    fn default() -> Self {
        Self
    }
}

impl Directional for NoSort {
    fn get_direction(&self) -> &SortDirection {
        &SortDirection::Ascending
    }
}

impl Ordorable for NoSort {
    fn to_order<'a>(self) -> Vec<(&'a str, bool)> {
        vec![]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MessageSort {
    #[serde(rename = "dateCreated")]
    DateCreated(SortDirection),
    #[serde(rename = "datePublished")]
    DatePublished(SortDirection),
    #[serde(rename = "messageTimestamp")]
    Timestamp(SortDirection),
}

impl Default for MessageSort {
    fn default() -> Self {
        Self::Timestamp(SortDirection::default())
    }
}

impl Directional for MessageSort {
    fn get_direction(&self) -> &SortDirection {
        match self {
            MessageSort::DateCreated(direction) => direction,
            MessageSort::DatePublished(direction) => direction,
            MessageSort::Timestamp(direction) => direction,
        }
    }
}

impl Ordorable for MessageSort {
    fn to_order<'a>(self) -> Vec<(&'a str, bool)> {
        match self {
            MessageSort::DateCreated(direction) => {
                vec![
                    ("dateCreated", direction.to_bool()),
                    ("cid", direction.to_bool()),
                ]
            }

            MessageSort::DatePublished(direction) => {
                vec![
                    ("datePublished", direction.to_bool()),
                    ("cid", direction.to_bool()),
                ]
            }
            MessageSort::Timestamp(direction) => {
                vec![
                    ("messageTimestamp", direction.to_bool()),
                    ("cid", direction.to_bool()),
                ]
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct MessageWatermark {
    direction: SortDirection,
}

impl Directional for MessageWatermark {
    fn get_direction(&self) -> &SortDirection {
        &self.direction
    }
}

impl Ordorable for MessageWatermark {
    fn to_order<'a, 's>(self) -> Vec<(&'a str, bool)> {
        vec![("watermark", self.direction.to_bool())]
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct QueryReturn<T> {
    pub items: Vec<T>,
    pub cursor: Option<Cursor>,
}

// Trait for implementing Filters
pub trait Query<U, T>
where
    U: DeserializeOwned,
    T: Directional,
{
    fn from<S>(&mut self, table: S) -> &mut Self
    where
        S: Into<String>;
    fn filter(&mut self, filters: &Filters) -> Result<&mut Self, errors::FilterError>;
    fn page(&mut self, pagination: Option<&Pagination>) -> &mut Self;
    fn always_cursor(&mut self) -> &mut Self;
    fn sort(&mut self, sort: Option<T>) -> &mut Self;
    fn query(
        &self,
    ) -> impl Future<Output = Result<(Vec<U>, Option<crate::Cursor>), errors::QueryError>> + Send;
}
