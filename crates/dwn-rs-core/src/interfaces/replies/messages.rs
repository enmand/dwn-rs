use cid::Cid;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{Cursor, Descriptor, Message};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ReadEntry {
    #[serde(rename = "messageCid")]
    pub cid: Cid,
    pub message: Option<Message<Descriptor>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Read {
    pub entry: Option<ReadEntry>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Query {
    pub entries: Option<Vec<Cid>>,
    pub cursor: Option<Cursor>,
}
