use std::fmt::Debug;

use aes::cipher::generic_array::GenericArray;
use ssi_jwk::{Base64urlUInt, OctetParams, Params, JWK};
use typenum::U32;

use super::{DeriveKey, Error, PrivateKeyError, PublicKeyTrait, SecretKeyTrait};

pub struct PublicKey {
    pub pk: x25519_dalek::PublicKey,
}

impl PublicKeyTrait for PublicKey {
    type KeySize = U32;
    type SecretKey = SecretKey;

    fn from_bytes(bytes: GenericArray<u8, Self::KeySize>) -> Result<Self, Error> {
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&bytes);
        Ok(Self {
            pk: x25519_dalek::PublicKey::from(pk),
        })
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::KeySize> {
        let v = self.pk.as_bytes().to_vec();
        GenericArray::from_iter(v.iter().copied())
    }

    fn jwk(&self) -> JWK {
        JWK::from(Params::OKP(OctetParams {
            curve: "X25519".to_string(),
            public_key: Base64urlUInt(self.to_bytes().to_vec()),
            private_key: None,
        }))
    }

    fn decapsulate(self, sk: Self::SecretKey) -> Result<GenericArray<u8, Self::KeySize>, Error> {
        todo!()
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

    fn encapsulate(self, pk: Self::PublicKey) -> Result<GenericArray<u8, Self::KeySize>, Error> {
        let shared = self.sk.diffie_hellman(&pk.pk).to_bytes();

        Ok(GenericArray::from_iter(shared[..32].iter().copied()))
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        todo!();
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
