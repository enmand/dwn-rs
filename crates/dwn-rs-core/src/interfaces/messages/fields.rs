use serde::{Deserialize, Serialize};
use ssi_jwk::JWK;

use crate::auth::{
    authorization::{Authorization, AuthorizationDelegatedGrant, AuthorizationOwner},
    jws::JWS,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Fields {
    EncodedWrite(EncodedWriteField),
    Write(WriteFields),
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
    ECIES_ES256K,
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
