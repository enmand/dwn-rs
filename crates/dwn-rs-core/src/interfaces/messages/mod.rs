use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{auth::JWS, MapValue, Value};

pub trait Descriptor: Default + PartialEq {}
pub trait Fields: Default + PartialEq {
    fn contains_key(&self, _key: &str) -> bool {
        false
    }
    fn insert(&mut self, _key: String, _value: Value) {}
    fn remove(&mut self, _key: &str) -> Option<Value> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct Message<D: Descriptor, F: Fields> {
    pub descriptor: D,
    #[serde(rename = "recordId", skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<JWS>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<JWS>,
    #[serde(flatten)]
    pub fields: F,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct GenericDescriptor {
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
    pub definition: Option<Value>,
    #[serde(flatten)]
    pub extra: MapValue,
}

impl Descriptor for GenericDescriptor {}
impl Fields for MapValue {
    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }

    fn remove(&mut self, key: &str) -> Option<Value> {
        self.remove(key)
    }

    fn insert(&mut self, key: String, value: Value) {
        self.insert(key, value);
    }
}

impl Descriptor for () {}

/// Interfaces represent the different Decentralized Web Node message interface types.
/// See <https://identity.foundation/decentralized-web-node/spec/#interfaces> for more information.
#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub enum Interface {
    #[default]
    Undefined,
    #[serde(rename = "records")]
    Records,
    #[serde(rename = "protocols")]
    Protocols,
    #[serde(rename = "permissions")]
    Permissions,
    #[serde(rename = "messages")]
    Messages,
    #[serde(rename = "snapshots")]
    Snapshots,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct MessageFilter {
    #[serde(rename = "dateCreated", skip_serializing_if = "Option::is_none")]
    pub date_created: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: MapValue,
}
