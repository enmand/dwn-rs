use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{
    auth::{authorization::Authorization, jws::JWS},
    encryption::Encryption,
    Value,
};

use super::{descriptors::records::WriteDescriptor, Message};

/// MessageFields is a trait that all message fields must implement.
/// It provides the interface and method for the message fields. The generic `Fields`
/// implements this trait for use when the concrete type is not known.
pub trait MessageFields: Default {
    /// encoded_data returns the encoded data for the message fields (if any),
    /// and removes the encoded data fields from the Message Fields.
    fn encoded_data(&mut self) -> Option<Value> {
        None
    }

    // encode_data encodes the data for the message
    fn encode_data(&mut self, _data: Value) {
        // no-op
    }

    fn set_authorization(&mut self, mut _authorization: Authorization) {
        unimplemented!("set_authorization not implemented for this message fields type");
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Fields {
    Write(WriteFields),
    InitialWriteField(InitialWriteField),
    Authorization(Authorization),
}

impl Default for Fields {
    fn default() -> Self {
        Fields::Write(WriteFields {
            authorization: Authorization::default(),
            record_id: None,
            context_id: None,
            encryption: None,
            attestation: None,
            encoded_data: None,
        })
    }
}

impl MessageFields for Fields {
    fn encoded_data(&mut self) -> Option<Value> {
        match self {
            Fields::Write(encoded_write) => encoded_write.encoded_data.take().map(Value::String),
            _ => None,
        }
    }

    fn encode_data(&mut self, data: Value) {
        if let Fields::Write(encoded_write) = self {
            encoded_write.encoded_data = Some(data.to_string());
        }
    }

    fn set_authorization(&mut self, authorization: Authorization) {
        match self {
            Fields::Write(write_fields) => write_fields.authorization = authorization,
            Fields::InitialWriteField(initial_write_field) => {
                initial_write_field.write_fields.authorization = authorization
            }
            Fields::Authorization(auth) => *auth = authorization,
        }
    }
}

impl MessageFields for Option<Fields> {
    fn encoded_data(&mut self) -> Option<Value> {
        match self {
            Some(fields) => fields.encoded_data(),
            None => None,
        }
    }

    fn encode_data(&mut self, data: Value) {
        if let Some(fields) = self {
            fields.encode_data(data);
        }
    }

    fn set_authorization(&mut self, authorization: Authorization) {
        if let Some(fields) = self {
            fields.set_authorization(authorization);
        }
    }
}

// InitialWriteField represents the RecordsWrite interface method response that includes
// the `initialWrite` data field if the original record was not the initial write.
#[derive(Serialize, Default, Deserialize, Debug, PartialEq, Clone)]
pub struct InitialWriteField {
    #[serde(flatten)]
    pub write_fields: WriteFields,
    #[serde(rename = "initialWrite", skip_serializing_if = "Option::is_none")]
    pub initial_write: Option<Box<Message<WriteDescriptor>>>,
}

impl MessageFields for InitialWriteField {
    fn set_authorization(&mut self, authorization: Authorization) {
        self.write_fields.authorization = authorization;
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct WriteFields {
    pub authorization: Authorization,
    #[serde(rename = "recordId")]
    pub record_id: Option<String>,
    #[serde(rename = "contextId")]
    pub context_id: Option<String>,
    pub encryption: Option<Encryption>,
    pub attestation: Option<JWS>,
    #[serde(rename = "encodedData")]
    pub encoded_data: Option<String>,
}

impl MessageFields for WriteFields {
    fn encoded_data(&mut self) -> Option<Value> {
        Some(
            self.encoded_data
                .take()
                .map(Value::String)
                .unwrap_or(Value::Null),
        )
    }

    fn encode_data(&mut self, data: Value) {
        self.encoded_data = Some(data.to_string());
    }

    fn set_authorization(&mut self, authorization: Authorization) {
        self.authorization = authorization;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::jws::SignatureEntry,
        encryption::{
            DerivationScheme, KeyEncryption, KeyEncryptionAlgorithm,
            KeyEncryptionAlgorithmAsymmetric, KeyEncryptionAlgorithmSymmetric,
        },
    };

    use super::*;
    use serde_json::{self, json};
    use ssi_jwk::JWK;
    use tracing_test::traced_test;

    #[test]
    fn test_fields_serialization() {
        let jwk = JWK::generate_ed25519().unwrap();

        // Define your test cases as structs or tuples.
        struct TestCase {
            fields: Fields,
            expected_json: String,
        }

        // Populate the vector with the test cases.
        let tests = vec![TestCase {
            fields: Fields::Write(WriteFields {
                record_id: Some("record_id".to_string()),
                context_id: Some("context_id".to_string()),
                authorization: Authorization {
                    signature: JWS {
                        payload: Some("payload".to_string()),
                        signatures: Some(vec![SignatureEntry {
                            protected: Some("protected".to_string()),
                            signature: Some("signature".to_string()),
                            ..Default::default()
                        }]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                encryption: Some(Encryption {
                    algorithm: KeyEncryptionAlgorithm::Symmetric(
                        KeyEncryptionAlgorithmSymmetric::AES256GCM,
                    ),
                    initialization_vector: "initialization_vector".to_string(),
                    key_encryption: vec![KeyEncryption {
                        algorithm: KeyEncryptionAlgorithm::Asymmetric(
                            KeyEncryptionAlgorithmAsymmetric::EciesSecp256k1,
                        ),
                        root_key_id: "root_key_id".to_string(),
                        derivation_scheme: DerivationScheme::DataFormats,
                        derived_public_key: None,
                        encrypted_key: "encrypted_key".to_string(),
                        initialization_vector: "initialization_vector".to_string(),
                        ephemeral_public_key: jwk.clone(),
                        message_authentication_code: "message_authentication_code".to_string(),
                    }],
                }),
                attestation: Some(JWS {
                    payload: Some("payload".to_string()),
                    signatures: Some(vec![SignatureEntry {
                        protected: Some("protected".to_string()),
                        signature: Some("signature".to_string()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
                encoded_data: Some("encoded_data".to_string()),
            }),
            expected_json: format!(
                r#"{{
                    "recordId": "record_id",
                    "contextId": "context_id",
                    "encryption": {{
                        "algorithm": "A256GCM",
                        "initializationVector": "initialization_vector",
                        "keyEncryption": [
                            {{
                                "algorithm": "ECIES-ES256K",
                                "rootKeyId": "root_key_id",
                                "derivationScheme": "dataFormats",
                                "derivedPublicKey": null,
                                "encryptedKey": "encrypted_key",
                                "initializationVector": "initialization_vector",
                                "ephemeralPublicKey": {jwk},
                                "messageAuthenticationCode": "message_authentication_code"
                            }}
                        ]
                    }},
                    "authorization": {{
                        "signature": {{
                            "payload": "payload",
                            "signatures": [
                                {{
                                    "protected": "protected",
                                    "signature": "signature"
                                }}
                            ]
                        }}
                    }},
                    "attestation": {{
                        "payload": "payload",
                        "signatures": [
                            {{
                                "protected": "protected",
                                "signature": "signature"
                            }}
                        ]
                    }},
                    "encodedData": "encoded_data"
                }}"#
            ),
        }];

        for test in tests.iter() {
            let json = serde_json::to_value(&test.fields).unwrap();
            let expected_json: serde_json::Value =
                serde_json::from_str(&test.expected_json).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    #[test]
    fn test_fields_deserialization() {
        let jwk = JWK::generate_ed25519().unwrap();
        let json = &format!(
            r#"
            {{
                "recordId": "record_id",
                "contextId": "context_id",
                "encryption": {{
                    "algorithm": "A256GCM",
                    "initializationVector": "initialization_vector",
                    "keyEncryption": [
                        {{
                            "algorithm": "ECIES-ES256K",
                            "rootKeyId": "root_key_id",
                            "derivationScheme": "dataFormats",
                            "derivedPublicKey": null,
                            "encryptedKey": "encrypted_key",
                            "initializationVector": "initialization_vector",
                            "ephemeralPublicKey": {jwk},
                            "messageAuthenticationCode": "message_authentication_code"
                        }}
                    ]
                }},
                "authorization": {{
                    "signature": {{
                        "payload": "payload",
                        "signatures": [
                            {{
                                "protected": "protected",
                                "signature": "signature"
                            }}
                        ]
                    }}
                }},
                "attestation": {{
                    "payload": "payload",
                    "signatures": [
                        {{
                            "protected": "protected",
                            "signature": "signature"
                        }}
                    ]
                }},
                "encodedData": "encoded_data"
            }}
        "#
        );

        let fields: Fields = serde_json::from_str(json).unwrap();
        println!("{:?}", fields);
        match fields {
            Fields::Write(WriteFields {
                record_id,
                context_id,
                encryption,
                attestation,
                encoded_data,
                ..
            }) => {
                assert_eq!(record_id, Some("record_id".to_string()));
                assert_eq!(context_id, Some("context_id".to_string()));
                assert!(encryption.is_some());
                assert!(attestation.is_some());
                assert_eq!(encoded_data, Some("encoded_data".to_string()));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_fields_serialization_with_null_fields() {
        let fields = Fields::Write(WriteFields {
            record_id: Some("record_id".to_string()),
            context_id: None,
            authorization: Authorization::default(),
            encryption: None,
            attestation: None,
            encoded_data: None,
        });

        let json =
            serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&fields).unwrap())
                .unwrap();
        let expected = json!({"recordId":"record_id","authorization":{"signature":{}}});
        assert_eq!(expected, json);
    }

    #[test]
    #[traced_test]
    fn test_fields_deserialization_with_null_fields() {
        let json = r#"{"recordId":"test", "authorization": {"signature":{}}}"#;

        let fields: Fields = serde_json::from_str(json).unwrap();

        match fields {
            Fields::Write(WriteFields {
                record_id,
                context_id,
                encryption,
                attestation,
                encoded_data,
                ..
            }) => {
                assert_eq!(record_id, Some("test".to_string()));
                assert!(context_id.is_none());
                assert!(encryption.is_none());
                assert!(attestation.is_none());
                assert!(encoded_data.is_none());
            }
            _ => unreachable!(),
        }
    }
}
