use serde::{Deserialize, Serialize};

use crate::Fields;

use super::{
    super::descriptors::{
        ConfigureDescriptor, DeleteDescriptor, MessagesQueryDescriptor, MessagesReadDescriptor,
        MessagesSubscribeDescriptor, ProtocolQueryDescriptor, ReadDescriptor,
        RecordsQueryDescriptor, RecordsWriteDescriptor, SubscribeDescriptor,
    },
    MessageDescriptor, MessageValidator, ValidationError, CONFIGURE, DELETE, MESSAGES, PROTOCOLS,
    QUERY, READ, RECORDS, SUBSCRIBE, WRITE,
};

/// Interfaces represent the different Decentralized Web Node message interface types.
/// See <https://identity.foundation/decentralized-web-node/spec/#interfaces> for more information.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Descriptor {
    Records(Records),
    Protocols(Protocols),
    Messages(Messages),
}

impl MessageValidator for Descriptor {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Descriptor::Records(_) => Ok(()),
            Descriptor::Protocols(_) => Ok(()),
            Descriptor::Messages(_) => Ok(()),
        }
    }
}

impl MessageDescriptor for Descriptor {
    type Fields = Fields;
    type Parameters = ();

    fn interface(&self) -> &'static str {
        match self {
            Descriptor::Records(_) => RECORDS,
            Descriptor::Protocols(_) => PROTOCOLS,
            Descriptor::Messages(_) => MESSAGES,
        }
    }

    fn method(&self) -> &'static str {
        match self {
            Descriptor::Records(records) => records.method(),
            Descriptor::Protocols(protocols) => protocols.method(),
            Descriptor::Messages(messages) => messages.method(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Records {
    Read(ReadDescriptor),
    Query(RecordsQueryDescriptor),
    Write(RecordsWriteDescriptor),
    Delete(DeleteDescriptor),
    Subscribe(SubscribeDescriptor),
}

impl MessageValidator for Records {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Records::Read(_) => Ok(()),
            Records::Query(_) => Ok(()),
            Records::Write(_) => Ok(()),
            Records::Delete(_) => Ok(()),
            Records::Subscribe(_) => Ok(()),
        }
    }
}

impl MessageDescriptor for Records {
    type Fields = Fields;
    type Parameters = ();

    fn interface(&self) -> &'static str {
        RECORDS
    }

    fn method(&self) -> &'static str {
        match self {
            Records::Read(_) => READ,
            Records::Query(_) => QUERY,
            Records::Write(_) => WRITE,
            Records::Delete(_) => DELETE,
            Records::Subscribe(_) => SUBSCRIBE,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Protocols {
    Configure(ConfigureDescriptor),
    Query(ProtocolQueryDescriptor),
}

impl MessageValidator for Protocols {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Protocols::Configure(_) => Ok(()),
            Protocols::Query(_) => Ok(()),
        }
    }
}

impl MessageDescriptor for Protocols {
    type Fields = Fields;
    type Parameters = ();

    fn interface(&self) -> &'static str {
        PROTOCOLS
    }

    fn method(&self) -> &'static str {
        match self {
            Protocols::Configure(_) => CONFIGURE,
            Protocols::Query(_) => QUERY,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Messages {
    Read(MessagesReadDescriptor),
    Query(MessagesQueryDescriptor),
    Subscribe(MessagesSubscribeDescriptor),
}

impl MessageValidator for Messages {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Messages::Read(_) => Ok(()),
            Messages::Query(_) => Ok(()),
            Messages::Subscribe(_) => Ok(()),
        }
    }
}

impl MessageDescriptor for Messages {
    type Fields = Fields;
    type Parameters = ();

    fn interface(&self) -> &'static str {
        MESSAGES
    }

    fn method(&self) -> &'static str {
        match self {
            Messages::Read(_) => READ,
            Messages::Query(_) => QUERY,
            Messages::Subscribe(_) => SUBSCRIBE,
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::Filters;

    #[test]
    fn test_descriptor_serialize() {
        use super::*;

        let now = chrono::Utc::now();
        let desc = Descriptor::Records(Records::Read(ReadDescriptor {
            message_timestamp: now,
            filter: Filters::default(),
        }));
        let serialized = json!(&desc);
        let expected = json!({"interface": RECORDS,"method": READ, "messageTimestamp": now, "filter": Filters::default()});

        assert_eq!(serialized, expected);
    }
}
