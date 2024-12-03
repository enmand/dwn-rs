use k256::{elliptic_curve::sec1::ToEncodedPoint, sha2};
use ssi_jwk::{Base64urlUInt, OctetParams, Params, JWK};

use super::{ECIESError, SecretKey};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing JWK: {0}")]
    PublicKeyError(#[from] ssi_jwk::Error),
    #[error("Unsupported Curve: {0}")]
    InvalidCurve(String),
    #[error("ECIES encryption error: {0}")]
    ECIESError(#[from] ECIESError),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum PublicKey {
    Secp256k1(k256::PublicKey),
    X25519(x25519_dalek::PublicKey),
}

impl PublicKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PublicKey::Secp256k1(pk) => pk.to_encoded_point(true).as_bytes().to_vec(),
            PublicKey::X25519(pk) => pk.as_bytes().to_vec(),
        }
    }

    pub fn jwk(&self) -> JWK {
        match self {
            PublicKey::Secp256k1(pk) => (*pk).into(),
            PublicKey::X25519(pk) => JWK::from(Params::OKP(OctetParams {
                curve: "X25519".to_string(),
                public_key: Base64urlUInt(pk.to_bytes().to_vec()),
                private_key: None,
            })),
        }
    }

    pub fn decapsulate(self, sk: &SecretKey) -> Result<[u8; 32], Error> {
        match (self, sk) {
            (PublicKey::Secp256k1(pk), SecretKey::Secp256k1(sk)) => {
                let mut okm = [0u8; 32];
                k256::ecdh::diffie_hellman(sk.to_nonzero_scalar(), pk.as_affine())
                    .extract::<sha2::Sha256>(None)
                    .expand(&[], &mut okm)
                    .map_err(ECIESError::InvalidHKDFKeyLength)?;
                Ok(okm)
            }
            (PublicKey::X25519(pk), SecretKey::X25519(sk)) => Ok(sk.diffie_hellman(&pk).to_bytes()),
            _ => Err(Error::InvalidCurve("Unsupported key type".to_string())),
        }
    }
}

impl From<&SecretKey> for PublicKey {
    fn from(sk: &SecretKey) -> Self {
        match sk {
            SecretKey::Secp256k1(sk) => PublicKey::Secp256k1(sk.public_key()),
            SecretKey::X25519(sk) => PublicKey::X25519(sk.into()),
        }
    }
}

impl TryFrom<JWK> for PublicKey {
    type Error = Error;

    fn try_from(jwk: JWK) -> Result<PublicKey, Self::Error> {
        match jwk.params {
            Params::EC(ref ec) => Ok(PublicKey::Secp256k1(ec.try_into()?)),
            Params::OKP(ref op) => match op.curve.to_lowercase().as_str() {
                "x25519" => {
                    let mut sk = [0u8; 32];
                    sk.copy_from_slice(&op.public_key.0);
                    Ok(PublicKey::X25519(x25519_dalek::PublicKey::from(sk)))
                }
                "ed25519" => {
                    let pk: ed25519_dalek::VerifyingKey = op.try_into()?;
                    Ok(PublicKey::X25519(x25519_dalek::PublicKey::from(
                        pk.to_montgomery().to_bytes(),
                    )))
                }
                _ => Err(Error::InvalidCurve(format!(
                    "Unsupported curve: {}",
                    op.curve
                ))),
            },
            _ => Err(Error::InvalidCurve("Unsupported key type".to_string())),
        }
    }
}
