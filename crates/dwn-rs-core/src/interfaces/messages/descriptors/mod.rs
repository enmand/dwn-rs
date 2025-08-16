pub mod general;
pub mod messages;
pub mod protocols;
pub mod records;

pub use general::*;

use crate::cid::generate_cid_from_serialized;
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

use super::{fields::MessageFields, Fields, Message};
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

pub trait MessageParameters {
    type Descriptor: MessageDescriptor;
    type Fields: MessageFields;

    #[allow(async_fn_in_trait)]
    async fn build(&self) -> Result<(Self::Descriptor, Option<Self::Fields>), ValidationError> {
        Err(ValidationError {
            message: String::from("not implemented"),
        })
    }

    fn delegated_grant(&self) -> Option<Message<RecordsWriteDescriptor>> {
        None
    }

    fn permission_grant_id(&self) -> Option<String> {
        None
    }

    fn protocol_rule(&self) -> Option<String> {
        None
    }
}

impl MessageParameters for () {
    type Descriptor = Descriptor;
    type Fields = Fields;
}

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
