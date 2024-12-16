pub mod publickey;
pub(crate) mod secp256k1;
pub mod secretkey;
pub(crate) mod x25519;

use aes::cipher::{generic_array::GenericArray, ArrayLength};
use k256::sha2;
use ssi_jwk::JWK;
use thiserror::Error;

use super::HashAlgorithm;
#[derive(Error, Debug)]
pub enum ECIESError {
    #[error("Invalid HKDF key length: {0}")]
    InvalidHKDFKeyLength(hkdf::InvalidLength),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error getting SecretKey from bytes: {0}")]
    SecretKeyError(String),
    #[error("Error parsing private key: {0}")]
    PrivateKeyError(#[from] PrivateKeyError),
    #[error("Error parsing private key: {0}")]
    PublicKeyError(#[from] PublicKeyError),
    #[error("ECIES encryption error: {0}")]
    ECIESError(#[from] ECIESError),
    #[error("Error deriving key: unsupported hash algorithm: {0}")]
    UnsupportedHashAlgorithm(String),
    #[error("Error deriving key, bad key length: {0}")]
    DeriveKeyLengthError(hkdf::InvalidLength),
}

#[derive(Error, Debug)]
pub enum PublicKeyError {
    #[error("Error parsing JWK: {0}")]
    PublicKeyError(#[from] ssi_jwk::Error),
    #[error("Curve error: {0}")]
    CurveError(#[from] k256::elliptic_curve::Error),
    #[error("Unsupported Curve: {0}")]
    InvalidCurve(String),
    #[error("ECIES encryption error: {0}")]
    ECIESError(#[from] ECIESError),
    #[error("Error parsing public key. Invalid length provided")]
    InvalidKey,
}

#[derive(Error, Debug)]
pub enum PrivateKeyError {
    #[error("Error encoding key: {0}")]
    EncodeError(#[from] k256::pkcs8::der::Error),
    #[error("Error parsing private key: {0}")]
    PrivateKeyError(#[from] ssi_jwk::Error),
    #[error("Error parsing private key: {0}")]
    ParseError(#[from] ParseError),
    #[error("Error parsing private key. Invalid length provided")]
    InvalidKeyLength,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Error parsing secp256k1 private key: {0}")]
    Secp256k1(#[from] k256::elliptic_curve::Error),
    #[error("Error parsing x25519 private key: {0}")]
    X25519(String),
    #[error("Error parsing ed25519 private key: {0}")]
    Ed25519(#[from] ed25519_dalek::SignatureError),
}

const HKDF_KEY_LENGTH: usize = 32; // * 8 (without sign); // 32 bytes = 256 bits

trait SecretKeyTrait: Sized {
    type KeySize: ArrayLength<u8>;
    type PublicKey: PublicKeyTrait<KeySize = Self::KeySize, SecretKey = Self>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;
    fn to_bytes(&self) -> Vec<u8>;
    fn public_key(&self) -> Self::PublicKey;
    fn jwk(&self) -> Result<JWK, Error>;
    fn encapsulate(self, pk: Self::PublicKey) -> Result<GenericArray<u8, Self::KeySize>, Error>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error>;
}

trait PublicKeyTrait: Sized {
    type KeySize: ArrayLength<u8>;
    type SecretKey: SecretKeyTrait<KeySize = Self::KeySize, PublicKey = Self>;

    fn from_bytes(bytes: GenericArray<u8, Self::KeySize>) -> Result<Self, Error>;
    fn to_bytes(&self) -> GenericArray<u8, Self::KeySize>;
    fn jwk(&self) -> JWK;
    fn decapsulate(self, sk: Self::SecretKey) -> Result<GenericArray<u8, Self::KeySize>, Error>;
}

trait DeriveKey: SecretKeyTrait {
    fn derive_hkdf_key(
        &self,
        hash_algo: HashAlgorithm,
        salt: &[u8],
        info: &[u8],
    ) -> Result<Self, Error> {
        if hash_algo != crate::encryption::HashAlgorithm::SHA256 {
            return Err(Error::UnsupportedHashAlgorithm(
                "Unsupported hash algorithm".to_string(),
            ));
        }
        let mut okm: [u8; HKDF_KEY_LENGTH] = [0; HKDF_KEY_LENGTH];

        let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(Some(salt), &self.to_bytes());
        hkdf.expand(info, &mut okm)
            .map_err(ECIESError::InvalidHKDFKeyLength)?;

        Self::from_bytes(okm.as_slice())
    }
}
