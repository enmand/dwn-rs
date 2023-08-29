use std::collections::HashMap;

use jose_jws::General as JWS;
use serde::{Deserialize, Serialize};

use crate::IndexValue;

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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct EncodedMessage {
    pub(super) encoded_message: Vec<u8>,
    pub(super) indexes: HashMap<String, IndexValue>,
}
