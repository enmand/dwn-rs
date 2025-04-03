pub mod descriptors;
pub mod fields;
pub mod protocols;

pub use descriptors::Descriptor;
use descriptors::{MessageDescriptor, MessageValidator, ValidationError};
pub use fields::Fields;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct Message<D: MessageDescriptor + DeserializeOwned> {
    pub descriptor: D,
    pub fields: D::Fields,
}

impl<D: MessageDescriptor + MessageValidator> Message<D> {
    pub fn new(descriptor: D, fields: D::Fields) -> Result<Self, ValidationError> {
        descriptor.validate()?;

        Ok(Self { descriptor, fields })
    }
}

impl<D> Serialize for Message<D>
where
    D: MessageDescriptor + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        #[derive(Serialize)]
        struct TempMessage<'a, D: MessageDescriptor> {
            descriptor: &'a D,
            #[serde(flatten)]
            other: &'a D::Fields,
        }

        let temp_message = TempMessage {
            descriptor: &self.descriptor,
            other: &self.fields,
        };

        temp_message.serialize(serializer)
    }
}

// This is a custom deserializer for the Message struct. It is necessary because the Message
// struct has a generic type parameter that is not known at compile time. This deserializer
// is the generalized version, which can deserialize any descriptor type. Individual
// Descriptors types implement their own deserializers via. the `MessageDescriptor` trait
// derivation.
impl<'de> Deserialize<'de> for Message<Descriptor> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TempMessage {
            descriptor: Descriptor,
            #[serde(flatten)]
            other: Fields,
        }

        let temp_message = TempMessage::deserialize(deserializer)?;

        Ok(Self {
            descriptor: temp_message.descriptor,
            fields: temp_message.other,
        })
    }
}

#[cfg(test)]
mod test {

    use chrono::Utc;
    use descriptors::{ReadDescriptor, Records};
    use dwn_rs_message_derive::descriptor;
    use fields::MessageFields;
    use serde_json::json;

    use crate::{auth::Authorization, Filters};

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestParameters {}

    const INTERFACE: &str = "interface";
    const METHOD: &str = "method";
    #[descriptor(interface = INTERFACE, method = METHOD, fields = TestFields, parameters = TestParameters)]
    struct TestDescriptor {
        data: String,
    }

    impl MessageValidator for TestDescriptor {
        fn validate(&self) -> Result<(), ValidationError> {
            if self.data.is_empty() {
                return Err(ValidationError {
                    message: "data".to_string(),
                });
            }
            Ok(())
        }
    }

    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    struct TestFields {
        field1: String,
        field2: i32,
    }
    impl MessageFields for TestFields {}

    #[test]
    fn test_message_serialize() {
        let desc = TestDescriptor {
            data: "test".to_string(),
        };
        let fields = TestFields {
            field1: "test".to_string(),
            field2: 42,
        };

        let message = Message::new(desc, fields).unwrap();

        let serialized = serde_json::to_string(&message).unwrap();
        let expected = r#"{"descriptor":{"data":"test","interface":"interface","method":"method"},"field1":"test","field2":42}"#;

        assert_eq!(serialized, expected);

        let now = Utc::now();

        let desc = Descriptor::Records(Records::Read(ReadDescriptor {
            message_timestamp: now,
            filter: Filters::default(),
        }));
        let fields = Fields::Authorization(Authorization {
            ..Default::default()
        });

        let message = Message::new(desc, fields).unwrap();
        let serialized = json!(&message);
        let expected = json!({
                "descriptor": {
                    "messageTimestamp": now,
                    "filter": Filters::default(),
                    "interface":"Records","method":"Read"
                },
                "signature":{}
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_message_deserialize() {
        let serialized = r#"{"descriptor":{"data":"test","interface":"interface","method":"method"},"field1":"test","field2":42}"#;

        let message: Message<TestDescriptor> = serde_json::from_str(serialized).unwrap();

        let descriptor = TestDescriptor {
            data: "test".to_string(),
        };

        let fields = TestFields {
            field1: "test".to_string(),
            field2: 42,
        };

        let expected = Message::new(descriptor, fields).unwrap();

        assert_eq!(message, expected);
    }
}
