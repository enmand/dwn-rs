pub mod general;
pub mod messages;
pub mod protocols;
pub mod records;

pub use general::*;

pub use messages::{
    QueryDescriptor as MessagesQueryDescriptor, ReadDescriptor as MessagesReadDescriptor,
    SubscribeDescriptor as MessagesSubscribeDescriptor,
};
pub use protocols::{ConfigureDescriptor, QueryDescriptor as ProtocolQueryDescriptor};
pub use records::{
    DeleteDescriptor, QueryDescriptor as RecordsQueryDescriptor, ReadDescriptor,
    SubscribeDescriptor, WriteDescriptor as RecordsWriteDescriptor,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::cid::generate_cid_from_serialized;

use super::fields::MessageFields;
use thiserror::Error;

pub const RECORDS: &str = "Records";
pub const PROTOCOLS: &str = "Protocols";
pub const MESSAGES: &str = "Messages";

pub const READ: &str = "Read";
pub const QUERY: &str = "Query";
pub const WRITE: &str = "Write";
pub const DELETE: &str = "Delete";
pub const SUBSCRIBE: &str = "Subscribe";
pub const CONFIGURE: &str = "Configure";

/// ValidationError represents an error that occurs during validation of a message descriptor.
#[derive(Serialize, Deserialize, Error, Debug)]
#[error("Validation error: {message}")]
pub struct ValidationError {
    /// The error message.
    pub message: String,
}

pub(crate) trait MessageParameters {
    fn build<D: MessageDescriptor, F: MessageFields>(&self) -> Result<(D, F), ValidationError> {
        return Err(ValidationError {
            message: String::from("not implemented"),
        });
    }
}

impl MessageParameters for () {}

/// MessageDescriptor is a trait that all message descriptors must implement.
/// It provides the interface and method for the message descriptor. The generic `Descriptor`
/// implements this trait for use when the concrete type is not known. Concrete Descriptor types
/// implement this trait directly (or use the derive macro).
pub trait MessageDescriptor: Serialize + DeserializeOwned + PartialEq {
    type Fields: MessageFields
        + Serialize
        + DeserializeOwned
        + std::fmt::Debug
        + PartialEq
        + Send
        + Sync
        + Clone;

    type Parameters: MessageParameters + Send + Sync;

    fn interface(&self) -> &'static str;
    fn method(&self) -> &'static str;
    fn cid(&self) -> cid::Cid {
        generate_cid_from_serialized(self)
            .expect("Failed to generate CID from serialized message descriptor")
    }
}

pub trait MessageValidator {
    fn validate(&self) -> Result<(), ValidationError> {
        Err(ValidationError {
            message: String::from("not implemented"),
        })
    }
}
