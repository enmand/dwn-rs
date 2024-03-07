use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct JWS {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<SignatureEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<BTreeMap<String, Ipld>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct SignatureEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Message {
    pub descriptor: Descriptor,
    #[serde(rename = "recordId", skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<JWS>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<JWS>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    #[serde(rename = "dataSize", skip_serializing_if = "Option::is_none")]
    pub data_size: Option<u32>,
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "crate::serde::serialize_optional_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(
        rename = "dateCreated",
        serialize_with = "crate::serde::serialize_optional_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_created: Option<DateTime<Utc>>,
    #[serde(
        rename = "datePublished",
        serialize_with = "crate::serde::serialize_optional_datetime",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_published: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<MessageFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<Ipld>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct MessageFilter {
    #[serde(rename = "dateCreated", skip_serializing_if = "Option::is_none")]
    pub date_created: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}
