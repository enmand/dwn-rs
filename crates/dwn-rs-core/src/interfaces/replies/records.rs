use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    descriptors::{records::WriteDescriptor, DeleteDescriptor},
    Cursor, Message,
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReadEntry {
    #[serde(rename = "recordsWrite")]
    pub records_write: Option<Message<WriteDescriptor>>,
    #[serde(rename = "recordsDelete")]
    pub records_delete: Option<Message<DeleteDescriptor>>,
    #[serde(rename = "initialWrite")]
    pub initial_write: Option<Message<WriteDescriptor>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Read {
    pub entry: Option<ReadEntry>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryEntry {
    #[serde(rename = "initialWrite")]
    pub initial_write: Option<Message<WriteDescriptor>>,
    #[serde(rename = "encodedData")]
    pub encoded_data: Option<String>,
    #[serde(flatten)]
    pub message: Message<WriteDescriptor>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Query {
    pub entries: Option<Vec<QueryEntry>>,
    pub cursor: Option<Cursor>,
}
