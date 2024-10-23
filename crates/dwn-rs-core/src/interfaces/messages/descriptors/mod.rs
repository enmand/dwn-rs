pub mod general;
pub mod messages;
pub mod protocols;
pub mod records;

use std::fmt::Debug;

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
use serde::{de::DeserializeOwned, Serialize};

use super::fields::MessageFields;

pub const RECORDS: &str = "Records";
pub const PROTOCOLS: &str = "Protocols";
pub const MESSAGES: &str = "Messages";

pub const READ: &str = "Read";
pub const QUERY: &str = "Query";
pub const WRITE: &str = "Write";
pub const DELETE: &str = "Delete";
pub const SUBSCRIBE: &str = "Subscribe";
pub const CONFIGURE: &str = "Configure";

/// MessageDescriptor is a trait that all message descriptors must implement.
/// It provides the interface and method for the message descriptor. The generic `Descriptor`
/// implements this trait for use when the concrete type is not known. Concrete Descriptor types
/// implement this trait directly (or use the derive macro).
pub trait MessageDescriptor: Serialize + DeserializeOwned + PartialEq {
    type Fields: MessageFields
        + Serialize
        + DeserializeOwned
        + Debug
        + PartialEq
        + Send
        + Sync
        + Clone;

    fn interface(&self) -> &'static str;
    fn method(&self) -> &'static str;
}
