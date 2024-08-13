use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ssi_dids_core::DIDBuf;

use crate::{
    filter::range_filter_serializer, value::Value, Filter, MapValue, Pagination, RangeFilter,
};

/// ReadDescriptor represents the RecordsRead interface method for reading a given
/// record by ID.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct ReadDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "recordId")]
    pub record_id: String,
}

// QueryDescriptor represents the RecordsQuery interface method for querying records.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct QueryDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: crate::Filters, // QueryFilter,
    pub pagination: Option<Pagination>,
    #[serde(rename = "dateSort")]
    pub date_sort: Option<DateSort>,
}

/// QueryFilter represents the filter criteria for querying records in the DWN. Filters exist
/// for various elements of message properties, such as `protocol`, `author`, `attester`,
/// `recipient`. Records can be filtered by `tags`.
/// TODO: Is this necessary? Can we just use the `Filters` type?
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct QueryFilter {
    pub protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    pub protocol_path: Option<String>,
    pub author: Option<DIDBuf>,
    pub attester: Option<DIDBuf>,
    pub recipient: Option<DIDBuf>,
    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    pub schema: Option<url::Url>,
    pub tags: Option<BTreeMap<String, Filter<Value>>>,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(
        rename = "dateCreated",
        serialize_with = "range_filter_serializer::serialize_optional"
    )]
    pub date_created: Option<RangeFilter<chrono::DateTime<chrono::Utc>>>,
}

/// DataSort represents Records ordering for queries.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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

/// WriteDescriptor represents the RecordsWrite interface method for writing a record to the DWN.
/// It can be represented with either no additional fields (`()`), or additional descriptor fields,
/// as in the case for `encodedData`.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct WriteDescriptor {
    pub protocol: Option<String>,
    #[serde(rename = "protocolPath")]
    pub protocol_path: Option<String>,
    pub recipient: Option<String>,
    pub schema: Option<String>,
    pub tags: Option<MapValue>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "dataCid")]
    pub data_cid: String,
    #[serde(rename = "dataSize")]
    pub data_size: u64,
    #[serde(
        rename = "dateCreated",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub date_created: chrono::DateTime<chrono::Utc>,
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub published: Option<bool>,
    #[serde(
        rename = "datePublished",
        serialize_with = "crate::ser::serialize_optional_datetime"
    )]
    pub date_published: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "dataFormat")]
    pub data_format: String,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct SubscribeDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    pub filter: crate::Filters, //QueryFilter,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DeleteDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "recordId")]
    pub record_id: String,
    pub prune: bool,
}
