use std::str::FromStr;

use crate::{CursorValue, MessageSort, MessageWatermark, NoSort};
use dwn_rs_core::{MapValue, Value};
use ipld_core::cid::Cid;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing, Value as SurrealValue};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateEncodedMessage {
    pub(super) cid: String,
    pub(super) tenant: String,
    pub(super) encoded_message: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) encoded_data: Option<Value>,
    #[serde(flatten)]
    pub(super) indexes: MapValue,
    pub(super) tags: MapValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetEncodedMessage {
    pub(super) id: Thing,
    pub(super) cid: String,
    pub(super) tenant: String,
    pub(super) encoded_message: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) encoded_data: Option<Value>,
    #[serde(flatten)]
    pub(super) indexes: MapValue,
}

impl CursorValue<MessageSort> for GetEncodedMessage {
    fn cursor_value(&self, sort: MessageSort) -> Value {
        match sort {
            MessageSort::DateCreated(_) => self.indexes.get("dateCreated").unwrap().clone(),
            MessageSort::DatePublished(_) => self.indexes.get("datePublished").unwrap().clone(),
            MessageSort::Timestamp(_) => self.indexes.get("messageTimestamp").unwrap().clone(),
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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateEvent {
    pub(super) cid: String,
    pub(super) watermark: Ulid,
    #[serde(flatten)]
    pub(super) indexes: MapValue,
    pub(super) tags: MapValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetEvent {
    pub(super) watermark: Ulid,
    pub(super) cid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Task<T: Serialize> {
    pub(super) id: Thing,
    pub(super) task: T,
    pub(super) timeout: Datetime,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateTask<T: Serialize> {
    pub(super) task: T,
    pub(super) timeout: SurrealValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ExtendTask {
    pub(super) timeout: SurrealValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ExtendedTask {
    pub(super) timeout: Datetime,
}

impl CursorValue<MessageWatermark> for GetEvent {
    fn cursor_value(&self, _: MessageWatermark) -> Value {
        Value::String(self.watermark.to_string())
    }

    fn cid(&self) -> Cid {
        Cid::from_str(&self.cid.to_string()).unwrap()
    }
}

impl<T> CursorValue<NoSort> for T
where
    T: Serialize + Sync + Send,
{
    fn cursor_value(&self, _: NoSort) -> Value {
        Value::Null
    }

    fn cid(&self) -> Cid {
        Cid::default()
    }
}
