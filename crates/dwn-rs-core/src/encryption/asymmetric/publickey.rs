use aes::cipher::generic_array::GenericArray;
use ssi_jwk::{Params, JWK};

use super::{secp256k1, secretkey, x25519, Error, PublicKeyError, PublicKeyTrait};

impl From<secp256k1::PublicKey> for PublicKey {
    fn from(pk: secp256k1::PublicKey) -> Self {
        PublicKey::Secp256k1(pk)
    }
}

impl From<x25519::PublicKey> for PublicKey {
    fn from(pk: x25519::PublicKey) -> Self {
        PublicKey::X25519(pk)
    }
}

pub enum PublicKey {
    Secp256k1(secp256k1::PublicKey),
    X25519(x25519::PublicKey),
}

// Maximum potential size utilized here based on known key sizes.
static MAX_PUBLIC_KEY_SIZE: usize = 33;

impl PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let ga = GenericArray::from_slice(bytes);
        match bytes.len() {
            33 => Ok(PublicKey::Secp256k1(secp256k1::PublicKey::from_bytes(*ga)?)),
            32 => {
                let mut x = [0u8; 32];
                x.copy_from_slice(bytes);
                let ga = GenericArray::from_slice(&x);

                Ok(PublicKey::X25519(x25519::PublicKey::from_bytes(*ga)?))
            }
            _ => Err(PublicKeyError::InvalidKey.into()),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Direct handling, interpolation balancing satisfies binary exact
        match self {
            PublicKey::Secp256k1(pk) => pk.to_bytes().to_vec(),
            PublicKey::X25519(pk) => pk.to_bytes().to_vec(),
        }
    }

    pub fn jwk(&self) -> JWK {
        match self {
            PublicKey::Secp256k1(pk) => pk.jwk(),
            PublicKey::X25519(pk) => pk.jwk(),
        }
    }

    pub fn decapsulate(self, sk: secretkey::SecretKey) -> Result<Vec<u8>, Error> {
        match self {
            PublicKey::Secp256k1(pk) => pk.decapsulate(sk.into()).map(|ga| ga.to_vec()),
            PublicKey::X25519(pk) => pk.decapsulate(sk.into()).map(|ga| ga.to_vec()),
        }
    }
}

impl TryFrom<JWK> for PublicKey {
    type Error = PublicKeyError;

    fn try_from(jwk: JWK) -> Result<PublicKey, Self::Error> {
        match jwk.params {
            Params::EC(ref ec) => Ok(PublicKey::Secp256k1(secp256k1::PublicKey {
                pk: ec.try_into().map_err(PublicKeyError::PublicKeyError)?,
            })),
            Params::OKP(ref op) => match op.curve.to_lowercase().as_str() {
                "x25519" => {
                    let mut sk = [0u8; 32];
                    sk.copy_from_slice(&op.public_key.0);
                    Ok(PublicKey::X25519(x25519::PublicKey {
                        pk: x25519_dalek::PublicKey::from(sk),
                    }))
                }
                "ed25519" => {
                    let pk: ed25519_dalek::VerifyingKey =
                        op.try_into().map_err(PublicKeyError::PublicKeyError)?;
                    Ok(PublicKey::X25519(x25519::PublicKey {
                        pk: x25519_dalek::PublicKey::from(pk.to_montgomery().to_bytes()),
                    }))
                }
                _ => Err(
                    PublicKeyError::InvalidCurve(format!("Unsupported curve: {}", op.curve)).into(),
                ),
            },
            _ => Err(PublicKeyError::InvalidCurve("Unsupported key type".to_string()).into()),
        }
    }
}
