use crate::encryption::symmetric::{Encryption, IVEncryption};
pub mod publickey;
pub(crate) mod secp256k1;
pub mod secretkey;
pub(crate) mod x25519;

use aes::cipher::{generic_array::GenericArray, ArrayLength};
use bytes::BytesMut;
use k256::sha2;
use ssi_jwk::JWK;
use thiserror::Error;
use typenum::Unsigned;

use super::{symmetric, HashAlgorithm};
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
    #[error("Error encrypting symmetric key: {0}")]
    EncryptionError(#[from] symmetric::Error),
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
    type PublicKey: PublicKeyTrait<SecretKey = Self>;

    fn generate_keypair() -> (Self, Self::PublicKey);

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;
    fn to_bytes(&self) -> Vec<u8>;
    fn public_key(&self) -> Self::PublicKey;
    fn jwk(&self) -> Result<JWK, Error>;

    #[allow(clippy::type_complexity)]
    fn encapsulate(
        &self,
        pk: &Self::PublicKey,
    ) -> Result<
        GenericArray<
            u8,
            <<Self::PublicKey as PublicKeyTrait>::SymmetricEncryption as Encryption>::KeySize,
        >,
        Error,
    >;

    // (EC)IES decryption for SecretKey
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let ephemeral_pk_len = <<Self::PublicKey as PublicKeyTrait>::KeySize as Unsigned>::USIZE;
        let ephemeral_pk = Self::PublicKey::from_bytes(&data[0..ephemeral_pk_len])?;

        let nonce_size = <<<Self::PublicKey as PublicKeyTrait>::SymmetricEncryption as IVEncryption>::NonceSize as Unsigned>::USIZE;
        let nonce_start = ephemeral_pk_len;
        let nonce_end = nonce_start + nonce_size;

        let nonce = GenericArray::from_slice(&data[nonce_start..nonce_end]).to_owned();
        let ciphertext = &data[nonce_end..];

        let key = self.encapsulate(&ephemeral_pk)?;

        let mut ciper = <Self::PublicKey as PublicKeyTrait>::SymmetricEncryption::new(key)?;
        let mut buf = BytesMut::from(ciphertext);
        let plaintext = ciper.with_iv(nonce)?.decrypt(&mut buf)?;

        Ok(plaintext.to_vec())
    }
}

trait PublicKeyTrait: Sized {
    type KeySize: ArrayLength<u8>;
    type SecretKey: SecretKeyTrait<PublicKey = Self>;
    type SymmetricEncryption: Encryption + IVEncryption;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;
    fn to_bytes(&self) -> Vec<u8>;
    fn jwk(&self) -> JWK;
    fn decapsulate(
        &self,
        sk: Self::SecretKey,
    ) -> Result<
        GenericArray<u8, <<Self as PublicKeyTrait>::SymmetricEncryption as Encryption>::KeySize>,
        Error,
    >;

    // (EC)IES encryption for PublicKey
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        let (empheral_sk, ephemeral_pk) = Self::SecretKey::generate_keypair();
        let key = empheral_sk.encapsulate(self)?;

        let mut cipher = Self::SymmetricEncryption::new(key)?;
        let nonce = cipher.nonce();

        let mut buf = BytesMut::from(data);
        let ciphertext = cipher.with_iv(nonce.clone())?.encrypt(&mut buf)?;

        let mut res = Vec::new();
        res.extend_from_slice(&ephemeral_pk.to_bytes());
        res.extend_from_slice(&nonce);
        res.extend_from_slice(&ciphertext); // ciphertext includes tag

        Ok(res)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        // Generate a key pair
        let (sk, pk) = secp256k1::SecretKey::generate_keypair();

        // Plaintext to encrypt
        let plaintext = b"Hello, world!";

        // Encrypt using the public key
        let ciphertext = pk.encrypt(plaintext).unwrap();

        // Decrypt using the secret key
        let decrypted = sk.decrypt(&ciphertext).unwrap();

        // Ensure the decrypted data matches the original plaintext
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_large_data() {
        // Generate a key pair
        let (sk, pk) = x25519::SecretKey::generate_keypair();

        // Large plaintext to encrypt
        let plaintext = vec![0u8; 1024 * 1024]; // 1 MB of data

        // Encrypt using the public key
        let ciphertext = pk.encrypt(&plaintext).unwrap();

        // Decrypt using the secret key
        let decrypted = sk.decrypt(&ciphertext).unwrap();

        // Ensure the decrypted data matches the original plaintext
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty_data() {
        // Generate a key pair
        let (sk, pk) = secp256k1::SecretKey::generate_keypair();

        // Empty plaintext to encrypt
        let plaintext = b"";

        // Encrypt using the public key
        let ciphertext = pk.encrypt(plaintext).unwrap();

        // Decrypt using the secret key
        let decrypted = sk.decrypt(&ciphertext).unwrap();

        // Ensure the decrypted data matches the original plaintext
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_invalid_ciphertext() {
        // Generate a key pair
        let (sk, _) = secp256k1::SecretKey::generate_keypair();

        // Invalid ciphertext (too short)
        let invalid_ciphertext = vec![0u8; 47]; // Less than 48 bytes (ephemeral_pk + nonce)

        // Attempt to decrypt
        let result = sk.decrypt(&invalid_ciphertext);

        // Ensure the operation fails with the correct error
        assert!(matches!(result, Err(Error::PublicKeyError(_))));
    }

    #[test]
    fn test_encrypt_decrypt_invalid_ephemeral_public_key() {
        // Generate a key pair
        let (sk, _) = secp256k1::SecretKey::generate_keypair();

        // Invalid ciphertext with invalid ephemeral public key
        let mut invalid_ciphertext = vec![0u8; 48]; // 32 bytes (ephemeral_pk) + 16 bytes (nonce)
        invalid_ciphertext.extend_from_slice(b"invalid ciphertext");

        // Attempt to decrypt
        let result = sk.decrypt(&invalid_ciphertext);

        // Ensure the operation fails with the correct error
        assert!(matches!(result, Err(Error::PublicKeyError(_))));
    }

    #[test]
    fn test_encrypt_decrypt_invalid_nonce() {
        // Generate a key pair
        let (sk, pk) = secp256k1::SecretKey::generate_keypair();

        // Encrypt valid data
        let plaintext = b"Hello, world!";
        let mut ciphertext = pk.encrypt(plaintext).unwrap();

        // Corrupt the nonce in the ciphertext
        ciphertext[32..48].copy_from_slice(&[0u8; 16]); // Overwrite nonce with zeros

        // Attempt to decrypt
        let result = sk.decrypt(&ciphertext);

        // Ensure the operation fails with the correct error
        assert!(matches!(result, Err(Error::EncryptionError(_))));
    }

    #[test]
    fn test_encrypt_decrypt_invalid_ciphertext_tag() {
        // Generate a key pair
        let (sk, pk) = secp256k1::SecretKey::generate_keypair();

        // Encrypt valid data
        let plaintext = b"Hello, world!";
        let mut ciphertext = pk.encrypt(plaintext).unwrap();

        // Corrupt the tag in the ciphertext
        let len = ciphertext.len();
        ciphertext[len - 1] ^= 0xFF; // Flip the last byte of the tag

        // Attempt to decrypt
        let result = sk.decrypt(&ciphertext);

        // Ensure the operation fails with the correct error
        assert!(matches!(result, Err(Error::EncryptionError(_))));
    }

    #[test]
    fn test_encrypt_decrypt_key_mismatch() {
        // Generate two key pairs
        let (_, pk1) = secp256k1::SecretKey::generate_keypair();
        let (sk2, _) = secp256k1::SecretKey::generate_keypair();

        // Encrypt using the first public key
        let plaintext = b"Hello, world!";
        let ciphertext = pk1.encrypt(plaintext).unwrap();

        // Attempt to decrypt using the second secret key
        let result = sk2.decrypt(&ciphertext);

        // Ensure the operation fails with the correct error
        assert!(matches!(result, Err(Error::EncryptionError(_))));
    }

    #[test]
    fn test_encapsulate_decapsulate() {
        // Generate two key pairs
        let (sk1, pk1) = secp256k1::SecretKey::generate_keypair();
        let (sk2, pk2) = secp256k1::SecretKey::generate_keypair();

        // Encapsulate using sk1 and pk2
        let shared_secret1 = sk1.encapsulate(&pk2).unwrap();

        // Encapsulate using sk2 and pk1
        let shared_secret2 = sk2.encapsulate(&pk1).unwrap();

        // Ensure the shared secrets match
        assert_eq!(shared_secret1, shared_secret2);
    }
}
