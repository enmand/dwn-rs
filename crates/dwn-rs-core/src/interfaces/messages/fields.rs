use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use ssi_jwk::JWK;

use crate::{
    auth::{
        authorization::{Authorization, AuthorizationDelegatedGrant, AuthorizationOwner},
        jws::JWS,
    },
    encryption::DerivationScheme,
    Value,
};

use super::{descriptors::records::WriteDescriptor, Message};

/// MessageFields is a trait that all message fields must implement.
/// It provides the interface and method for the message fields. The generic `Fields`
/// implements this trait for use when the concrete type is not known.
pub trait MessageFields {
    /// encoded_data returns the encoded data for the message fields (if any),
    /// and removes the encoded data fields from the Message Fields.
    fn encoded_data(&mut self) -> Option<Value> {
        None
    }

    // encode_data encodes the data for the message
    fn encode_data(&mut self, _data: Value) {
        // no-op
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Fields {
    Write(WriteFields),
    InitialWriteField(InitialWriteField),
    Authorization(Authorization),
    AuthorizationDelegatedGrant(AuthorizationDelegatedGrantFields),
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
}

/// ReadFields are the message fields for the RecordsRead interface method.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct AuthorizationDelegatedGrantFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<AuthorizationDelegatedGrant>,
}

impl MessageFields for AuthorizationDelegatedGrantFields {}

// InitialWriteField represents the RecordsWrite interface method response that includes
// the `initialWrite` data field if the original record was not the initial write.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct InitialWriteField {
    #[serde(flatten)]
    pub write_fields: WriteFields,
    #[serde(rename = "initialWrite", skip_serializing_if = "Option::is_none")]
    pub initial_write: Option<Box<Message<WriteDescriptor>>>,
}

impl MessageFields for InitialWriteField {}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct WriteFields {
    pub authorization: AuthorizationOwner,
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
}

/// EncryptionAlgorithm represents the encryption algorithm used for encrypting records. Currently
/// A256CTR is the only supported algorithm.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum EncryptionAlgorithm {
    A256CTR,
}

/// KeyEncryptionAlgorithm represents the key encryption algorithm used for encrypting keys.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum KeyEncryptionAlgorithm {
    #[serde(rename = "ECIES-ES256K")]
    #[allow(non_camel_case_types)]
    EciesEs256k,
}

/// KeyEncryption represents the key encryption used for encrypting keys.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct KeyEncryption {
    pub algorithm: KeyEncryptionAlgorithm,
    #[serde(rename = "rootKeyId")]
    pub root_key_id: String,
    #[serde(rename = "derivationScheme")]
    pub derivation_scheme: DerivationScheme,
    #[serde(rename = "derivedPublicKey")]
    pub derived_public_key: Option<JWK>,
    #[serde(rename = "encryptedKey")]
    pub encrypted_key: String,
    #[serde(rename = "initializationVector")]
    pub initialization_vector: String,
    #[serde(rename = "ephemeralPublicKey")]
    pub ephemeral_public_key: JWK,
    #[serde(rename = "messageAuthenticationCode")]
    pub message_authentication_code: String,
}

/// Encryption represents the encryption used for encrypting records.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Encryption {
    pub algorithm: EncryptionAlgorithm,
    #[serde(rename = "initializationVector")]
    pub initialization_vector: String,
    #[serde(rename = "keyEncryption")]
    pub key_encryption: Vec<KeyEncryption>,
}

#[cfg(test)]
mod tests {
    use crate::descriptors::{RecordsWriteDescriptor, RECORDS, WRITE};
    use crate::{auth::jws::SignatureEntry, Message};

    use super::*;
    use serde_json::{self, json};
    use ssi_jwk::JWK;
    use tracing_test::traced_test;

    #[test]
    fn test_fields_serialization() {
        use serde_json::json;

        let jwk = JWK::generate_ed25519().unwrap();
        let now = chrono::Utc::now();

        // Define your test cases as structs or tuples.
        struct TestCase {
            fields: Fields,
            expected_json: String,
        }

        // Populate the vector with the test cases.
        let tests = vec![
            TestCase {
                fields: Fields::Write(WriteFields {
                    record_id: Some("record_id".to_string()),
                    context_id: Some("context_id".to_string()),
                    authorization: AuthorizationOwner {
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
                        algorithm: EncryptionAlgorithm::A256CTR,
                        initialization_vector: "initialization_vector".to_string(),
                        key_encryption: vec![KeyEncryption {
                            algorithm: KeyEncryptionAlgorithm::EciesEs256k,
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
                        "algorithm": "A256CTR",
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
            },
            TestCase {
                fields: Fields::AuthorizationDelegatedGrant(AuthorizationDelegatedGrantFields {
                    authorization: Some(AuthorizationDelegatedGrant {
                        // fill in all the fields with fake details
                        signature: JWS::default(),
                        author_delegated_grant: Some(Box::new(Message::<RecordsWriteDescriptor> {
                            descriptor: crate::descriptors::RecordsWriteDescriptor {
                                data_cid: "data_cid".to_string(),
                                data_size: 0,
                                date_created: now,
                                message_timestamp: now,
                                data_format: "data_format".to_string(),
                                protocol: None,
                                recipient: None,
                                schema: None,
                                tags: None,
                                protocol_path: None,
                                parent_id: None,
                                published: None,
                                date_published: None,
                            },
                            fields: WriteFields {
                                record_id: Some("record".to_string()),
                                context_id: Some("context".to_string()),
                                authorization: AuthorizationOwner::default(),
                                encryption: None,
                                attestation: None,
                                encoded_data: None,
                            },
                        })),
                    }),
                }),
                expected_json: json!({
                    "authorization": {
                        "signature": JWS::default(),
                        "authorDelegatedGrant": {
                            "descriptor": json!({
                                "interface": RECORDS,
                                "method": WRITE,
                                "dataCid": "data_cid",
                                "dataSize": 0,
                                "dateCreated": now.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                                "messageTimestamp": now.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                                "dataFormat": "data_format",
                            }),
                            "authorization": {
                                "signature": JWS::default(),
                            },
                            "recordId": "record",
                            "contextId": "context",
                        },
                    },
                })
                .to_string(),
            },
        ];

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
                    "algorithm": "A256CTR",
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
            authorization: AuthorizationOwner::default(),
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
