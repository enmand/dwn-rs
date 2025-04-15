pub mod asymmetric;
pub mod errors;
pub mod hd_keys;
pub mod symmetric;

pub use asymmetric::secretkey::SecretKey;
pub use errors::Error;
pub use hd_keys::{DerivedPrivateJWK, HashAlgorithm};

use serde::{Deserialize, Serialize};
use ssi_jwk::JWK;

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
#[serde(untagged)]
pub enum KeyEncryptionAlgorithm {
    Asymmetric(KeyEncryptionAlgorithmAsymmetric),
    Symmetric(KeyEncryptionAlgorithmSymmetric),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum KeyEncryptionAlgorithmAsymmetric {
    #[serde(rename = "ECIES-ES256K")]
    EciesSecp256k1,
    #[serde(rename = "X25519")]
    X25519,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum KeyEncryptionAlgorithmSymmetric {
    #[serde(rename = "A256GCM")] // dwn-sdk-js does aes-gcm but calls the property aes-ctr
    AES256CTR,
    #[serde(rename = "A256GCM")]
    AES256GCM,
    #[serde(rename = "XSalsa20-Poly1305")]
    XSalsa20Poly1305,
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
    pub algorithm: KeyEncryptionAlgorithm,
    #[serde(rename = "initializationVector")]
    pub initialization_vector: String,
    #[serde(rename = "keyEncryption")]
    pub key_encryption: Vec<KeyEncryption>,
}
