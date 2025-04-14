use std::collections::BTreeMap;

use cid::Cid;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::Value;

use super::{Filter, RangeFilter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[skip_serializing_none]
pub struct Records {
    pub author: Option<Vec<String>>,
    pub attester: Option<String>,
    pub recipient: Option<Vec<String>>,
    pub protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    pub protocol_path: Option<String>,
    pub published: Option<bool>,

    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    pub schema: Option<String>,
    pub tags: Option<BTreeMap<String, Filter<Value>>>,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "dataFormat")]
    pub data_format: Option<String>,
    #[serde(rename = "dataSize")]
    pub data_size: Option<RangeFilter<u64>>,
    #[serde(rename = "dataCid")]
    pub data_cid: Option<Cid>,
    #[serde(rename = "dateCreated")]
    pub date_created: Option<RangeFilter<String>>,
    #[serde(rename = "datePublished")]
    pub date_published: Option<RangeFilter<String>>,
    #[serde(rename = "dateUpdated")]
    pub date_updated: Option<RangeFilter<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[skip_serializing_none()]
pub struct Messages {
    pub interface: Option<String>,
    pub method: Option<String>,
    pub protocol: Option<String>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}
