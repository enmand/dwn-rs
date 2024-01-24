use crate::{Filters, Message, SOrders};

use crate::filters::errors;
use async_trait::async_trait;
use cid::Cid;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{serde_as, DisplayFromStr};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Pagination {
    pub cursor: Option<Cursor>,
    pub limit: Option<u32>,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Cursor {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "messageCid")]
    pub cursor: Cid,
    pub value: Option<crate::value::Value>,
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

impl MessageSort {
    pub fn get_direction(&self) -> SortDirection {
        match self {
            MessageSort::DateCreated(direction) => direction.clone(),
            MessageSort::DatePublished(direction) => direction.clone(),
            MessageSort::Timestamp(direction) => direction.clone(),
        }
    }

    pub fn to_order(&self) -> SOrders {
        match self {
            MessageSort::DateCreated(direction) => {
                SOrders::new().push(("dateCreated", direction.to_bool()))
            }
            MessageSort::DatePublished(direction) => {
                SOrders::new().push(("datePublished", direction.to_bool()))
            }
            MessageSort::Timestamp(direction) => {
                SOrders::new().push(("messageTimestamp", direction.to_bool()))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct QueryReturn {
    pub messages: Vec<Message>,
    pub cursor: Option<Cursor>,
}

// Trait for implementing Filters
#[async_trait]
pub trait Query<U>
where
    U: DeserializeOwned,
{
    fn from<S>(&mut self, table: S) -> &mut Self
    where
        S: Into<String>;
    fn filter(&mut self, filters: &Filters) -> Result<&mut Self, errors::FilterError>;
    fn page(&mut self, pagination: Option<Pagination>) -> &mut Self;
    fn sort(&mut self, sort: Option<MessageSort>) -> &mut Self;
    async fn query(&self) -> Result<(Vec<U>, Option<crate::Cursor>), errors::QueryError>;
}

pub trait CursorValue {
    fn cid(&self) -> Cid;
    fn cursor_value(&self, sort: MessageSort) -> &crate::value::Value;
}
