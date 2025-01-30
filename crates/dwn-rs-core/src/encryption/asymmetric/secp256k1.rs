use std::fmt::Debug;

use aes::cipher::generic_array::GenericArray;
use k256::sha2;
use ssi_jwk::{secp256k1_parse_private, JWK};
use tracing::error;
use typenum::{U32, U33};

use crate::encryption::symmetric::{self, Encryption};

use super::{
    DeriveKey, ECIESError, Error, ParseError, PrivateKeyError, PublicKeyError, PublicKeyTrait,
    SecretKeyTrait,
};

pub struct PublicKey {
    pub pk: k256::PublicKey,
}

impl PublicKeyTrait for PublicKey {
    type KeySize = U33;
    type SecretKey = SecretKey;
    type SymmetricEncryption = symmetric::aead::AES256GCM;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let pk = k256::PublicKey::from_sec1_bytes(bytes).map_err(PublicKeyError::CurveError)?;
        Ok(Self { pk })
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.pk.to_sec1_bytes().to_vec()
    }

    fn jwk(&self) -> JWK {
        self.pk.into()
    }

    fn decapsulate(
        &self,
        sk: Self::SecretKey,
    ) -> Result<
        GenericArray<
            u8,
            <<Self as PublicKeyTrait>::SymmetricEncryption as symmetric::Encryption>::KeySize,
        >,
        Error,
    > {
        sk.encapsulate(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SecretKey {
    sk: k256::SecretKey,
}

impl DeriveKey for SecretKey {}

impl SecretKeyTrait for SecretKey {
    type KeySize = U32;
    type PublicKey = PublicKey;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let sk: k256::SecretKey = k256::SecretKey::from_slice(bytes).map_err(|e| {
            error!("Error parsing secp256k1 private key: {:?}", e);
            PrivateKeyError::ParseError(ParseError::Secp256k1(e))
        })?;
        Ok(SecretKey { sk })
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.sk.to_bytes().to_vec()
    }

    fn public_key(&self) -> Self::PublicKey {
        let pk = self.sk.public_key();
        PublicKey { pk }
    }

    fn jwk(&self) -> Result<JWK, Error> {
        let mut jwk: JWK = self.sk.public_key().into();
        let pjwk = secp256k1_parse_private(
            &self
                .sk
                .to_sec1_der()
                .map_err(PrivateKeyError::EncodeError)?,
        )
        .map_err(PrivateKeyError::PrivateKeyError)?;
        jwk.params = pjwk.params.clone();

        Ok(jwk)
    }

    fn encapsulate(
        &self,
        pk: &Self::PublicKey,
    ) -> Result<
        GenericArray<u8, <<Self::PublicKey as PublicKeyTrait>::SymmetricEncryption as symmetric::Encryption>::KeySize>,
        Error,
    >{
        let mut okm: GenericArray<
            u8,
            <<Self::PublicKey as PublicKeyTrait>::SymmetricEncryption as Encryption>::KeySize,
        > = GenericArray::default();

        k256::ecdh::diffie_hellman(self.sk.to_nonzero_scalar(), pk.pk.as_affine())
            .extract::<sha2::Sha256>(None)
            .expand(&[], &mut okm)
            .map_err(ECIESError::InvalidHKDFKeyLength)?;

        Ok(okm)
    }

    fn generate_keypair() -> (Self, Self::PublicKey) {
        let sk = k256::SecretKey::random(&mut rand::thread_rng());
        let pk = sk.public_key();
        (sk.into(), pk.into())
    }
}

impl From<k256::PublicKey> for PublicKey {
    fn from(pk: k256::PublicKey) -> Self {
        PublicKey { pk }
    }
}

impl From<k256::SecretKey> for SecretKey {
    fn from(sk: k256::SecretKey) -> Self {
        SecretKey { sk }
    }
}

#[cfg(test)]
mod tests {
    use crate::encryption::HashAlgorithm;

    use super::*;
    use k256::SecretKey as K256SecretKey;
    use rand::thread_rng;

    // Helper function to generate a random SecretKey
    fn generate_random_secret_key() -> SecretKey {
        let sk = K256SecretKey::random(&mut thread_rng());
        SecretKey { sk }
    }

    #[test]
    fn test_secret_key_from_bytes() {
        // Generate a random secret key
        let sk = generate_random_secret_key();
        let sk_bytes = sk.to_bytes();

        // Test parsing from bytes
        let parsed_sk = SecretKey::from_bytes(&sk_bytes).unwrap();
        assert_eq!(sk.to_bytes(), parsed_sk.to_bytes());
    }

    #[test]
    fn test_secret_key_to_bytes() {
        // Generate a random secret key
        let sk = generate_random_secret_key();
        let sk_bytes = sk.to_bytes();

        // Ensure the bytes are not all zeros
        assert_ne!(sk_bytes, vec![0u8; 32]);
    }

    #[test]
    fn test_secret_key_public_key() {
        // Generate a random secret key
        let sk = generate_random_secret_key();
        let pk = sk.public_key();

        // Ensure the public key is derived correctly
        let expected_pk_bytes = sk.sk.public_key().to_sec1_bytes();
        assert_eq!(pk.to_bytes().as_slice(), &expected_pk_bytes[..33]);
    }

    #[test]
    fn test_secret_key_jwk() {
        // Generate a random secret key
        let sk = generate_random_secret_key();
        let jwk = sk.jwk().unwrap();

        // Ensure the JWK contains the correct curve and key type
        match jwk.params {
            ssi_jwk::Params::EC(ec) => {
                assert_eq!(ec.curve, Some("secp256k1".to_string()));
            }
            _ => panic!("Invalid JWK params"),
        }
    }

    #[test]
    fn test_public_key_from_bytes() {
        // Generate a random secret key and its corresponding public key
        let sk = generate_random_secret_key();
        let pk = sk.public_key();
        let pk_bytes = pk.to_bytes();

        // Test parsing from bytes
        let parsed_pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        assert_eq!(pk.to_bytes(), parsed_pk.to_bytes());
    }

    #[test]
    fn test_public_key_to_bytes() {
        // Generate a random secret key and its corresponding public key
        let sk = generate_random_secret_key();
        let pk = sk.public_key();
        let pk_bytes = pk.to_bytes();

        // Ensure the bytes are not all zeros
        assert_ne!(pk_bytes, vec![0u8; 33]);
    }

    #[test]
    fn test_public_key_jwk() {
        // Generate a random secret key and its corresponding public key
        let sk = generate_random_secret_key();
        let pk = sk.public_key();
        let jwk = pk.jwk();

        // Ensure the JWK contains the correct curve and key type
        match jwk.params {
            ssi_jwk::Params::EC(ec) => {
                assert_eq!(ec.curve, Some("secp256k1".to_string()));
            }
            _ => panic!("Invalid JWK params"),
        }
    }

    #[test]
    fn test_encapsulate_decapsulate() {
        // Generate two key pairs
        let sk1 = generate_random_secret_key();
        let pk1 = sk1.public_key();
        let sk2 = generate_random_secret_key();
        let pk2 = sk2.public_key();

        // Encapsulate using sk1 and pk2
        let shared_secret1 = sk1.encapsulate(&pk2).unwrap();

        // Encapsulate using sk2 and pk1
        let shared_secret2 = sk2.encapsulate(&pk1).unwrap();

        // Ensure the shared secrets match
        assert_eq!(shared_secret1, shared_secret2);
    }

    #[test]
    fn test_generate_keypair() {
        // Generate a key pair
        let (sk, pk) = SecretKey::generate_keypair();

        // Ensure the secret key and public key are consistent
        let derived_pk = sk.public_key();
        assert_eq!(pk.to_bytes(), derived_pk.to_bytes());
    }

    #[test]
    fn test_invalid_secret_key_from_bytes() {
        // Test parsing an invalid secret key (wrong length)
        let invalid_bytes = vec![0u8; 31]; // 31 bytes instead of 32
        let result = SecretKey::from_bytes(&invalid_bytes);
        assert!(result.is_err());

        // Test parsing an invalid secret key (zero key)
        let zero_bytes = vec![0u8; 32];
        let result = SecretKey::from_bytes(&zero_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_public_key_from_bytes() {
        // Test parsing an invalid public key (wrong length)
        let invalid_bytes = vec![0u8; 31]; // 31 bytes instead of 32
        let result = PublicKey::from_bytes(&invalid_bytes);
        assert!(result.is_err());

        // Test parsing an invalid public key (invalid encoding)
        let invalid_encoding = vec![0u8; 34];
        let result = PublicKey::from_bytes(&invalid_encoding);
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_hkdf_key() {
        // Generate a random secret key
        let sk = generate_random_secret_key();

        // Derive a new key using HKDF
        let salt = b"salt";
        let info = b"info";
        let derived_sk = sk
            .derive_hkdf_key(HashAlgorithm::SHA256, salt, info)
            .unwrap();

        // Ensure the derived key is different from the original key
        assert_ne!(sk.to_bytes(), derived_sk.to_bytes());
    }

    #[test]
    fn test_unsupported_hash_algorithm() {
        // Generate a random secret key
        let sk = generate_random_secret_key();

        // Attempt to derive a key using an unsupported hash algorithm
        let salt = b"salt";
        let info = b"info";
        let result = sk.derive_hkdf_key(HashAlgorithm::SHA512, salt, info);

        // Ensure the operation fails with the correct error
        assert!(matches!(result, Err(Error::UnsupportedHashAlgorithm(_))));
    }
}
