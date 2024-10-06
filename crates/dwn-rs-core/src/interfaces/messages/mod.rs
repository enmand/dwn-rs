pub mod descriptors;
pub mod fields;

pub use descriptors::Descriptor;
pub use fields::Fields;

use fields::InitialWriteField;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{Cursor, SubscriptionID};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Message {
    pub descriptor: Descriptor,
    #[serde(flatten)]
    pub fields: Fields, // Fields should be an Enum representing possible fields<D-s>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ResponseEntries {
    Message(Message),
    String(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Record {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReadReplyEntry {
    pub cid: cid::Cid,
    message: Message,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Status {
    pub code: i32,
    pub detail: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Response {
    pub status: Status,
    pub entries: Option<Vec<ResponseEntries>>,
    pub entry: Option<ReadReplyEntry>,
    pub record: Option<InitialWriteField>,
    pub cursor: Option<Cursor>,
    pub subscription: Option<SubscriptionID>,
}
