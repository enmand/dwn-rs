use std::collections::BTreeMap;

use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::Indexes;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct JWS {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    pub signature: Option<SignatureEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<SignatureEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<BTreeMap<String, Ipld>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    #[serde(rename = "dataSize", skip_serializing_if = "Option::is_none")]
    pub data_size: Option<u32>,
    #[serde(
        rename = "messageTimestamp",
        serialize_with = "serialize_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(
        rename = "dateCreated",
        serialize_with = "serialize_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(
        rename = "datePublished",
        serialize_with = "serialize_str",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_published: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<MessageFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

fn serialize_str<S>(
    date: &Option<chrono::DateTime<chrono::Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match date {
        Some(date) => {
            serializer.serialize_str(&date.to_rfc3339_opts(chrono::SecondsFormat::Micros, true))
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct MessageFilter {
    #[serde(rename = "dateCreated", skip_serializing_if = "Option::is_none")]
    pub date_created: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct DateRange {}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateEncodedMessage {
    pub(super) cid: String,
    pub(super) tenant: String,
    pub(super) encoded_message: Vec<u8>,
    #[serde(flatten)]
    pub(super) indexes: Indexes,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetEncodedMessage {
    pub(super) id: Thing,
    pub(super) cid: String,
    pub(super) tenant: String,
    pub(super) encoded_message: Vec<u8>,
}
