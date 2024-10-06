use serde::{Deserialize, Serialize};
use ssi_jwk::JWK;

use crate::auth::{
    authorization::{Authorization, AuthorizationDelegatedGrant, AuthorizationOwner},
    jws::JWS,
};

use super::Message;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Fields {
    EncodedWrite(EncodedWriteField),
    Write(WriteFields),
    InitialWriteField(InitialWriteField),
    Authorization(Authorization),
    AuthorizationDelegatedGrant(AuthorizationDelegatedGrantFields),
}

/// ReadFields are the message fields for the RecordsRead interface method.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct AuthorizationDelegatedGrantFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<AuthorizationDelegatedGrant>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
pub struct WriteFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<AuthorizationOwner>,
    #[serde(rename = "recordId", skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<Encryption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<JWS>,
}
/// EncodedWriteField represents the RecordsWrite interface method for writing a record to
/// the DWN using the `encodedData` field for records data that is encoded in messages directly.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct EncodedWriteField {
    #[serde(flatten)]
    pub write_fields: WriteFields,
    #[serde(rename = "encodedData", skip_serializing_if = "Option::is_none")]
    pub encoded_data: Option<String>,
}

// InitialWriteField represents the RecordsWrite interface method response that includes
// the `initialWrite` data field if the original record was not the initial write.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct InitialWriteField {
    #[serde(flatten)]
    pub write_fields: EncodedWriteField,
    #[serde(rename = "initialWrite", skip_serializing_if = "Option::is_none")]
    pub initial_write: Option<Box<Message>>,
}

/// EncryptionAlgorithm represents the encryption algorithm used for encrypting records. Currently
/// A256CTR is the only supported algorithm.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum EncryptionAlgorithm {
    A256CTR,
}

// DerivationScheme represents the derivation scheme used for deriving keys for encryption.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum DerivationScheme {
    #[serde(rename = "dataFormats")]
    DataFormats,
    #[serde(rename = "protocolContext")]
    ProtocolContext,
    #[serde(rename = "protocolPath")]
    ProtocolPath,
    #[serde(rename = "schemas")]
    Schemas,
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
    use crate::{auth::jws::SignatureEntry, descriptors::Records, Descriptor, Message};

    use super::*;
    use serde_json;
    use ssi_jwk::JWK;

    #[test]
    fn test_fields_serialization() {
        use serde_json::json;

        let jwk = JWK::generate_ed25519().unwrap();

        // Define your test cases as structs or tuples.
        struct TestCase {
            fields: Fields,
            expected_json: String,
        }

        // Populate the vector with the test cases.
        let tests = vec![
            TestCase {
                fields: Fields::EncodedWrite(EncodedWriteField {
                    write_fields: WriteFields {
                        record_id: Some("record_id".to_string()),
                        context_id: Some("context_id".to_string()),
                        authorization: None,
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
                                message_authentication_code: "message_authentication_code"
                                    .to_string(),
                            }],
                        }),
                        attestation: Some(JWS {
                            payload: Some("payload".to_string()),
                            signatures: Some(vec![SignatureEntry {
                                payload: Some("payload".to_string()),
                                protected: Some("protected".to_string()),
                                signature: Some("signature".to_string()),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        }),
                    },
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
                    "attestation": {{
                        "payload": "payload",
                        "signatures": [
                            {{
                                "payload": "payload",
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
                        author_delegated_grant: Some(Box::new(Message {
                            descriptor: Descriptor::Records(Records::Write(
                                crate::descriptors::RecordsWriteDescriptor::default(),
                            )),
                            fields: Fields::Write(WriteFields::default()),
                        })),
                    }),
                }),
                expected_json: json!({
                        "authorization": {
                            "signature": JWS::default(),
                            "authorDelegatedGrant": {
                                "descriptor": Descriptor::Records(Records::Write(
                                    crate::descriptors::RecordsWriteDescriptor::default()
                                )),
                                // there are no Fields on this request for test serialization
                            }
                        },
                    }
                )
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
                "attestation": {{
                    "payload": "payload",
                    "signatures": [
                        {{
                            "payload": "payload",
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
        match fields {
            Fields::EncodedWrite(EncodedWriteField {
                write_fields,
                encoded_data,
            }) => {
                assert_eq!(write_fields.record_id, Some("record_id".to_string()));
                assert_eq!(write_fields.context_id, Some("context_id".to_string()));
                assert!(write_fields.authorization.is_none());
                assert!(write_fields.encryption.is_some());
                assert!(write_fields.attestation.is_some());
                assert_eq!(encoded_data, Some("encoded_data".to_string()));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_fields_serialization_with_null_fields() {
        let fields = Fields::EncodedWrite(EncodedWriteField {
            write_fields: WriteFields {
                record_id: None,
                context_id: None,
                authorization: None,
                encryption: None,
                attestation: None,
            },
            encoded_data: None,
        });

        let json = serde_json::to_string(&fields).unwrap();
        assert_eq!("{}", json);
    }

    #[test]
    fn test_fields_deserialization_with_null_fields() {
        let json = "{}";

        let fields: Fields = serde_json::from_str(json).unwrap();
        match fields {
            Fields::EncodedWrite(EncodedWriteField {
                write_fields,
                encoded_data,
            }) => {
                assert!(write_fields.record_id.is_none());
                assert!(write_fields.context_id.is_none());
                assert!(write_fields.authorization.is_none());
                assert!(write_fields.encryption.is_none());
                assert!(write_fields.attestation.is_none());
                assert!(encoded_data.is_none());
            }
            _ => unreachable!(),
        }
    }
}
