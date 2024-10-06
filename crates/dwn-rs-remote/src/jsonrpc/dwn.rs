use dwn_rs_core::Message;
use serde::{Deserialize, Serialize};

pub const PROCESS_MESSAGE: &str = "dwn.processMessage";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProcessMessageParams {
    pub target: String,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none", rename = "encodedData")]
    pub encoded_data: Option<Vec<u8>>,
}
