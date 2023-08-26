use std::{collections::TryReserveError, convert::Infallible, sync::Arc};

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use cid::multihash::{Code, MultihashDigest};
use jose_jws::General as JWS;
use serde::{Deserialize, Serialize, Serializer};
use surrealdb::engine::any::Any;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    pub descriptor: Descriptor,
    pub authroization: Option<JWS>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    pub timestamp: u64,
}

pub enum Filter {
    Property(String),
    Filter(/* filter types */),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Index {
    pub key: String,
    pub value: IndexValue,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IndexValue {
    Bool(bool),
    String(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EncodedMessage {
    // serialize as a string
    #[serde(
        serialize_with = "serialize_base64",
        deserialize_with = "deserialize_base64"
    )]
    pub(super) encoded_message: Vec<u8>,
    #[serde(serialize_with = "serialize_cid", deserialize_with = "deserialize_cid")]
    pub(super) cid: cid::Cid,
    pub(super) indexes: Vec<Index>,
}

fn deserialize_base64<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<u8>, D::Error> {
    let s = String::deserialize(deserializer)?;
    general_purpose::STANDARD
        .decode(s.as_bytes())
        .map_err(serde::de::Error::custom)
}

fn deserialize_cid<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<cid::Cid, D::Error> {
    use std::str::FromStr;
    let s = String::deserialize(deserializer)?;
    cid::Cid::from_str(&s).map_err(serde::de::Error::custom)
}

fn serialize_base64<T: Serializer>(bytes: &[u8], serializer: T) -> Result<T::Ok, T::Error> {
    serializer.serialize_str(&general_purpose::STANDARD.encode(bytes))
}

fn serialize_cid<T: Serializer>(cid: &cid::Cid, serializer: T) -> Result<T::Ok, T::Error> {
    serializer.serialize_str(&cid.to_string())
}
