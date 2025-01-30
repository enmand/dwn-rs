use std::fmt::Debug;

use aes::cipher::generic_array::GenericArray;
use ssi_jwk::{Base64urlUInt, OctetParams, Params, JWK};
use typenum::U32;

use crate::encryption::symmetric;

use super::{DeriveKey, Error, PrivateKeyError, PublicKeyTrait, SecretKeyTrait};

pub struct PublicKey {
    pub pk: x25519_dalek::PublicKey,
}

impl PublicKeyTrait for PublicKey {
    type KeySize = U32;
    type SecretKey = SecretKey;
    type SymmetricEncryption = symmetric::aead::AES256GCM;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut pk = [0u8; 32];

        if bytes.len() != 32 {
            return Err(PrivateKeyError::InvalidKeyLength.into());
        }

        if bytes.iter().all(|&x| x == 0) {
            return Err(PrivateKeyError::InvalidKeyLength.into());
        }

        pk.copy_from_slice(bytes);
        Ok(Self {
            pk: x25519_dalek::PublicKey::from(pk),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.pk.as_bytes().to_vec()
    }

    fn jwk(&self) -> JWK {
        JWK::from(Params::OKP(OctetParams {
            curve: "X25519".to_string(),
            public_key: Base64urlUInt(self.to_bytes().to_vec()),
            private_key: None,
        }))
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

#[derive(Clone)]
pub struct SecretKey {
    sk: x25519_dalek::StaticSecret,
}

impl DeriveKey for SecretKey {}

impl Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("X25519")
            .field("pk", &self.public_key().to_bytes())
            .finish()
    }
}

impl PartialEq for SecretKey {
    fn eq(&self, other: &Self) -> bool {
        self.sk.to_bytes() == other.sk.to_bytes()
    }
}

impl SecretKeyTrait for SecretKey {
    type KeySize = U32;
    type PublicKey = PublicKey;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(PrivateKeyError::InvalidKeyLength.into());
        }

        if bytes.iter().all(|&x| x == 0) {
            return Err(PrivateKeyError::InvalidKeyLength.into());
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);

        Ok(SecretKey {
            sk: x25519_dalek::StaticSecret::from(key),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.sk.as_bytes().to_vec()
    }

    fn public_key(&self) -> Self::PublicKey {
        let pk = x25519_dalek::PublicKey::from(&self.sk);
        PublicKey { pk }
    }

    fn jwk(&self) -> Result<JWK, Error> {
        let pk: x25519_dalek::PublicKey = (&self.sk).into();
        let jwk = JWK::from(Params::OKP(OctetParams {
            curve: "X25519".to_string(),
            public_key: Base64urlUInt(pk.as_bytes().to_vec()),
            private_key: Some(Base64urlUInt(self.sk.as_bytes().to_vec())),
        }));

        Ok(jwk)
    }

    fn encapsulate(
        &self,
        pk: &Self::PublicKey,
    ) -> Result<
        GenericArray<u8, <<Self::PublicKey as PublicKeyTrait>::SymmetricEncryption as symmetric::Encryption>::KeySize>,
        Error,
    >{
        let shared = self.sk.diffie_hellman(&pk.pk).to_bytes();

        Ok(GenericArray::from_iter(shared[..32].iter().copied()))
    }

    fn generate_keypair() -> (Self, Self::PublicKey) {
        let sk = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
        let pk = x25519_dalek::PublicKey::from(&sk);
        (sk.into(), pk.into())
    }
}

impl From<ed25519_dalek::SigningKey> for SecretKey {
    fn from(sk: ed25519_dalek::SigningKey) -> Self {
        SecretKey {
            sk: x25519_dalek::StaticSecret::from(sk.to_scalar_bytes()),
        }
    }
}

impl From<x25519_dalek::StaticSecret> for SecretKey {
    fn from(sk: x25519_dalek::StaticSecret) -> Self {
        SecretKey { sk }
    }
}

impl From<x25519_dalek::PublicKey> for PublicKey {
    fn from(pk: x25519_dalek::PublicKey) -> Self {
        PublicKey { pk }
    }
}

#[cfg(test)]
mod tests {
    use crate::encryption::HashAlgorithm;

    use super::*;
    use rand::thread_rng;
    use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

    // Helper function to generate a random SecretKey
    fn generate_random_secret_key() -> SecretKey {
        let sk = StaticSecret::random_from_rng(thread_rng());
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
        let expected_pk_bytes = X25519PublicKey::from(&sk.sk).to_bytes();
        assert_eq!(pk.to_bytes().as_slice(), expected_pk_bytes.as_slice());
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
        assert_ne!(pk_bytes, vec![0u8; 32]);
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
        let invalid_encoding = vec![0u8; 32]; // All zeros
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
