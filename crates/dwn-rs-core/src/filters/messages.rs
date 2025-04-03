use std::collections::BTreeMap;

use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::Value;

use super::{Filter, RangeFilter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Records {
    author: Option<Vec<String>>,
    attester: Option<String>,
    recipient: Option<Vec<String>>,
    protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    protocol_path: Option<String>,
    published: Option<bool>,

    #[serde(rename = "contextId")]
    context_id: Option<String>,
    schema: Option<String>,
    tags: Option<BTreeMap<String, Filter<Value>>>,
    #[serde(rename = "recordId")]
    record_id: Option<String>,
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
    #[serde(rename = "dataFormat")]
    data_format: Option<String>,
    #[serde(rename = "dataSize")]
    data_size: Option<RangeFilter<u64>>,
    #[serde(rename = "dataCid")]
    data_cid: Option<Cid>,
    #[serde(rename = "dateCreated")]
    date_created: Option<RangeFilter<String>>,
    #[serde(rename = "datePublished")]
    date_published: Option<RangeFilter<String>>,
    #[serde(rename = "dateUpdated")]
    date_updated: Option<RangeFilter<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Messages {
    interface: Option<String>,
    method: Option<String>,
    protocol: Option<String>,
    #[serde(rename = "messageTimestamp")]
    message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}
