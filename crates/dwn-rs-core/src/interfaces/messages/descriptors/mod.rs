pub mod messages;
pub mod protocols;
pub mod records;

pub use messages::{
    QueryDescriptor as MessagesQueryDescriptor, ReadDescriptor as MessagesReadDescriptor,
    SubscribeDescriptor as MessagesSubscribeDescriptor,
};
pub use protocols::{ConfigureDescriptor, QueryDescriptor as ProtocolQueryDescriptor};
pub use records::{
    DeleteDescriptor, QueryDescriptor as RecordsQueryDescriptor, ReadDescriptor,
    SubscribeDescriptor, WriteDescriptor as RecordsWriteDescriptor,
};

use serde::{Deserialize, Serialize};

/// Interfaces represent the different Decentralized Web Node message interface types.
/// See <https://identity.foundation/decentralized-web-node/spec/#interfaces> for more information.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "interface")]
pub enum Descriptor {
    Records(Records),
    Protocols(Protocols),
    Messages(Messages),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "method")]
pub enum Records {
    Read(ReadDescriptor),
    Query(RecordsQueryDescriptor),
    Write(RecordsWriteDescriptor),
    Delete(DeleteDescriptor),
    Subscribe(SubscribeDescriptor),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "method")]
pub enum Protocols {
    Configure(ConfigureDescriptor),
    Query(ProtocolQueryDescriptor),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Messages {
    Read(MessagesReadDescriptor),
    Query(MessagesQueryDescriptor),
    Subscribe(MessagesSubscribeDescriptor),
}
