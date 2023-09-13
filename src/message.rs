use std::collections::BTreeMap;

use libipld_core::ipld::Ipld;
//use jose_jws::General as JWS;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use crate::Indexes;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct JWS {
    pub payload: String,
    pub signatures: Vec<SignatureEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<BTreeMap<String, Ipld>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SignatureEntry {
    pub protected: String,
    pub signature: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
}

// #[derive(Serialize, Deserialize, Debug, Default)]
// pub struct Message {
//     pub descriptor: Descriptor,
//     pub authorization: Option<JWS>,
//     #[serde(flatten)]
//     pub extra: BTreeMap<String, Ipld>,
// }
pub type Message = BTreeMap<String, Ipld>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    #[serde(rename = "dataSize")]
    pub data_size: u32,
    #[serde(rename = "messageTimestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Ipld>,
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
