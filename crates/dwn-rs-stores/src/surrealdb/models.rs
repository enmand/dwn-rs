use std::str::FromStr;

use crate::{CursorValue, Indexes, MessageSort, Value};
use libipld_core::cid::Cid;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateEncodedMessage {
    pub(super) cid: String,
    pub(super) tenant: String,
    pub(super) encoded_message: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) encoded_data: Option<Ipld>,
    #[serde(flatten)]
    pub(super) indexes: Indexes,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetEncodedMessage {
    pub(super) id: Thing,
    pub(super) cid: String,
    pub(super) tenant: String,
    pub(super) encoded_message: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) encoded_data: Option<Ipld>,
    #[serde(flatten)]
    pub(super) indexes: Indexes,
}

impl CursorValue<MessageSort> for GetEncodedMessage {
    fn cursor_value(&self, sort: MessageSort) -> &Value {
        match sort {
            MessageSort::DateCreated(_) => self.indexes.indexes.get("dateCreated").unwrap(),
            MessageSort::DatePublished(_) => self.indexes.indexes.get("datePublished").unwrap(),
            MessageSort::Timestamp(_) => self.indexes.indexes.get("messageTimestamp").unwrap(),
        }
    }

    fn cid(&self) -> Cid {
        Cid::from_str(&self.cid).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateData {
    pub(super) cid: String,
    pub(super) data: Vec<u8>,
    pub(super) tenant: String,
    pub(super) record_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetData {
    pub(super) id: Thing,
    pub(super) cid: String,
    pub(super) data: Vec<u8>,
    pub(super) tenant: String,
    pub(super) record_id: String,
}
