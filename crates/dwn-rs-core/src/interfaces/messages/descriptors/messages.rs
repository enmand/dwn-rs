use cid::Cid;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReadDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "messageCid", skip_serializing_if = "Option::is_none")]
    pub message_cid: Option<Cid>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<crate::Filters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<crate::Cursor>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum QueryInterfaces {
    Protocols,
    Read,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum QueryMethods {
    Configure,
    Delete,
    Write,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryMessageTimestamp {
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryFilter {
    pub interface: Option<QueryInterfaces>,
    pub method: Option<QueryMethods>,
    pub protocol: Option<String>,
    #[serde(rename = "messageTimestamp")]
    pub message_timestamp: Option<QueryMessageTimestamp>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SubscribeDescriptor {
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::ser::serialize_datetime"
    )]
    pub message_timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<crate::Filters>,
}
