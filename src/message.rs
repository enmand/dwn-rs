use std::collections::HashMap;

use jose_jws::General as JWS;
use serde::{Deserialize, Serialize};
use serde_cbor::Value;
use surrealdb::sql::Thing;

use crate::Indexes;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    pub descriptor: Descriptor,
    pub authroization: Option<JWS>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    pub timestamp: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateEncodedMessage {
    pub(super) encoded_message: Vec<u8>,
    #[serde(flatten)]
    pub(super) indexes: Indexes,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetEncodedMessage {
    pub(super) id: Thing,
    pub(super) encoded_message: Vec<u8>,
}
