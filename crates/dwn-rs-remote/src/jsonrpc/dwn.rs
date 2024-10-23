use dwn_rs_core::{descriptors::MessageDescriptor, Message};
use serde::Serialize;

pub const PROCESS_MESSAGE: &str = "dwn.processMessage";

#[derive(Debug, Serialize)]
pub struct ProcessMessageParams<D: MessageDescriptor> {
    pub target: String,
    pub message: Message<D>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "encodedData")]
    pub encoded_data: Option<Vec<u8>>,
}
