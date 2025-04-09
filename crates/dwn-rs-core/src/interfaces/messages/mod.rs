pub mod descriptors;
pub mod fields;
pub mod protocols;

use std::collections::TryReserveError;

use crate::auth::{jws, JWS};
use crate::cid::generate_cid_from_serialized;
use crate::{auth::Authorization, interfaces::messages::descriptors::MessageParameters};
use cid::Cid;
pub use descriptors::Descriptor;
use descriptors::{MessageDescriptor, MessageValidator, RecordsWriteDescriptor, ValidationError};
pub use fields::Fields;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_ipld_dagcbor::EncodeError;
use ssi_jws::JwsSigner;

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

impl<D: MessageDescriptor> Message<D> {
    pub fn cid(&self) -> Result<Cid, EncodeError<TryReserveError>> {
        generate_cid_from_serialized(self)
    }

    pub async fn create<S: JwsSigner>(
        parameters: D::Parameters,
        signer: Option<S>,
    ) -> Result<Self, ValidationError> {
        let (descriptor, fields) = parameters.build()?;

        let delegated_grant = None;
        let permission_grant_id = None;
        let protocol_rule = None;

        if let Some(signer) = signer {
            let authorization = Self::create_authorization(
                &descriptor,
                signer,
                delegated_grant,
                permission_grant_id,
                protocol_rule,
            )
            .await?;
        }

        Ok(Self { descriptor, fields })
    }

    async fn create_authorization<S: JwsSigner>(
        descriptor: &D,
        signer: S,
        delegated_grant: Option<Message<RecordsWriteDescriptor>>,
        permission_grant_id: Option<String>,
        protocol_rule: Option<String>,
    ) -> Result<Authorization, ValidationError> {
        let delegated_grant_id: Option<Cid> = if let Some(delegated_grant) = delegated_grant.clone()
        {
            Some(delegated_grant.cid().map_err(|err| ValidationError {
                message: err.to_string(),
            })?)
        } else {
            None
        };

        let signature = Self::create_signature(
            descriptor,
            signer,
            delegated_grant_id,
            permission_grant_id,
            protocol_rule,
        )
        .await?;

        let mut authorization = Authorization {
            signature,
            ..Default::default()
        };

        if let Some(grant) = delegated_grant {
            authorization.author_delegated_grant = Some(Box::new(grant));
        }

        Ok(authorization)
    }

    async fn create_signature<S: JwsSigner>(
        descriptor: &D,
        signer: S,
        delegated_grant_id: Option<Cid>,
        permission_grant_id: Option<String>,
        protocol_rule: Option<String>,
    ) -> Result<JWS, ValidationError> {
        let descriptor_cid = descriptor.cid();

        let payload = jws::Payload {
            descriptor_cid,
            delegated_grant_id,
            permission_grant_id,
            protocol_rule,
        };

        let signature = jws::JWS::create(payload, Some(vec![signer]))
            .await
            .map_err(|e| ValidationError {
                message: e.to_string(),
            })?;

        Ok(signature)
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

    impl MessageParameters for TestParameters {}

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
