use crate::{Filters, Message};

use crate::filters::errors;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Pagination {
    #[serde(rename = "messageCid")]
    pub message_cid: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(i8)]
pub enum SortDirection {
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MessageSort {
    #[serde(rename = "dateCreated")]
    pub date_created: Option<SortDirection>,
    #[serde(rename = "datePublished")]
    pub date_published: Option<SortDirection>,
    #[serde(rename = "messageTimestamp")]
    pub timestamp: Option<SortDirection>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct QueryReturn {
    pub messages: Vec<Message>,
    pub pagination_message_cid: Option<String>,
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
    async fn query(&self) -> Result<Vec<U>, errors::QueryError>;
}
