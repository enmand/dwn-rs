use std::{collections::BTreeMap, default};

use serde::{Deserialize, Serialize};
use web5::dids::identifier::Identifier;

use crate::{value::Value, Filter, Interface, RangeFilter};

#[derive(Serialize, Deserialize, Debug)]
pub enum RecordsMethod {
    #[serde(rename = "read")]
    Read,
    #[serde(rename = "query")]
    Query,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadDescriptor {
    pub interface: Interface,
    pub method: RecordsMethod,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "recordId")]
    pub record_id: String,
}

impl default::Default for ReadDescriptor {
    fn default() -> Self {
        Self {
            interface: Interface::Records,
            method: RecordsMethod::Read,
            message_timestamp: chrono::Utc::now(),
            record_id: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryDescriptor {
    pub interface: Interface,
    pub method: RecordsMethod,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: QueryFilter,
    #[serde(rename = "dateSort")]
    pub date_sort: Option<DateSort>,
}

impl default::Default for QueryDescriptor {
    fn default() -> Self {
        Self {
            interface: Interface::Records,
            method: RecordsMethod::Query,
            message_timestamp: chrono::Utc::now(),
            filter: Default::default(),
            date_sort: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct QueryFilter {
    pub protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    pub protocol_path: Option<String>,
    #[serde(with = "crate::serde::identifier")]
    pub author: Option<Identifier>,
    #[serde(with = "crate::serde::identifier")]
    pub attester: Option<Identifier>,
    #[serde(with = "crate::serde::identifier")]
    pub recipient: Option<Identifier>,
    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    pub schema: Option<url::Url>,
    pub tags: Option<BTreeMap<String, Filter<Value>>>,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "dateCreated")]
    pub date_created: Option<RangeFilter<chrono::DateTime<chrono::Utc>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DateSort {
    #[serde(rename = "createdAscending")]
    CreatedAscending,
    #[serde(rename = "createdDescending")]
    CreatedDescending,
    #[serde(rename = "publishedAscending")]
    PublishedAscending,
    #[serde(rename = "publishedDescending")]
    PublishedDescending,
}

pub type RecordsRead = crate::Message<ReadDescriptor, ()>;
pub type RecordsQuery = crate::Message<QueryDescriptor, ()>;
