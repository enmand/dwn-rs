use std::fmt::Debug;

use k256::sha2;
use ssi_jwk::{secp256k1_parse_private, Base64urlUInt, OctetParams, Params, JWK};
use thiserror::Error;

use super::{ECIESError, PublicKey};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error getting SecretKey from bytes: {0}")]
    SecretKeyError(String),
    #[error("Error parsing private key: {0}")]
    PrivateKeyError(#[from] PrivateKeyError),
    #[error("ECIES encryption error: {0}")]
    ECIESError(#[from] ECIESError),
}

#[derive(Error, Debug)]
pub enum PrivateKeyError {
    #[error("Error encoding key: {0}")]
    EncodeError(#[from] k256::pkcs8::der::Error),
    #[error("Error parsing private key: {0}")]
    PrivateKeyError(#[from] ssi_jwk::Error),
    #[error("Error parsing private key: {0}")]
    ParseError(#[from] ParseError),
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

/// SecretKey represents a private asymmetric key. Supported key types are:
/// - secp256k1
/// - x25519
/// - ed25519 (converted to x25519)
///
/// secp256k1 keys are preferred, since the x26619 keys are converted from ed25519 keys. See
/// also: https://eprint.iacr.org/2021/509
#[derive(Clone)]
#[non_exhaustive]
pub enum SecretKey {
    Secp256k1(k256::SecretKey),
    X25519(x25519_dalek::StaticSecret),
}

impl PartialEq for SecretKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SecretKey::Secp256k1(sk1), SecretKey::Secp256k1(sk2)) => sk1 == sk2,
            (SecretKey::X25519(sk1), SecretKey::X25519(sk2)) => sk1.as_bytes() == sk2.as_bytes(),
            _ => false,
        }
    }
}

impl Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn x25519_bytes(sk: &x25519_dalek::StaticSecret) -> [u8; 32] {
            let pk: x25519_dalek::PublicKey = sk.into();
            pk.to_bytes()
        }

        match self {
            SecretKey::Secp256k1(sk) => write!(f, "Secp256k1(pub: {:?})", sk.public_key()),
            SecretKey::X25519(sk) => write!(f, "X25519(pub: {:?})", x25519_bytes(sk)),
        }
    }
}

impl SecretKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            SecretKey::Secp256k1(sk) => sk.to_bytes().to_vec(),
            SecretKey::X25519(sk) => sk.as_bytes().to_vec(),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.into()
    }

    pub fn jwk(&self) -> Result<JWK, Error> {
        match self {
            SecretKey::Secp256k1(sk) => {
                let mut jwk: JWK = sk.public_key().into();
                let pjwk = secp256k1_parse_private(
                    &sk.to_sec1_der().map_err(PrivateKeyError::EncodeError)?,
                )
                .map_err(PrivateKeyError::PrivateKeyError)?;
                jwk.params = pjwk.params.clone();

                Ok(jwk)
            }
            SecretKey::X25519(sk) => {
                let pk: x25519_dalek::PublicKey = sk.into();
                let jwk = JWK::from(Params::OKP(OctetParams {
                    curve: "X25519".to_string(),
                    public_key: Base64urlUInt(pk.as_bytes().to_vec()),
                    private_key: Some(Base64urlUInt(sk.as_bytes().to_vec())),
                }));

                Ok(jwk)
            }
        }
    }

    pub fn encapsulate(self, pk: PublicKey) -> Result<[u8; 32], Error> {
        // TODO support key compression for secp256k1 hkdf key and ephemeral key
        match (self, pk) {
            (SecretKey::Secp256k1(sk), PublicKey::Secp256k1(pk)) => {
                let mut okm = [0u8; 32];
                k256::ecdh::diffie_hellman(sk.to_nonzero_scalar(), pk.as_affine())
                    .extract::<sha2::Sha256>(None)
                    .expand(&[], &mut okm)
                    .map_err(ECIESError::InvalidHKDFKeyLength)?;

                Ok(okm)
            }
            (SecretKey::X25519(sk), PublicKey::X25519(pk)) => Ok(sk.diffie_hellman(&pk).to_bytes()),
            _ => Err(Error::SecretKeyError("Unsupported key type".to_string())),
        }
    }
}

/// TryFrom (&SecretKey, &[u8; 32]) for SecretKey implements the conversion of a HKDF derived key
/// into a SecretKey of the same type
impl TryFrom<(&SecretKey, &[u8; 32])> for SecretKey {
    type Error = Error;

    fn try_from(value: (&SecretKey, &[u8; 32])) -> Result<Self, Self::Error> {
        let sk = match value.0 {
            SecretKey::Secp256k1(_) => {
                let sk: k256::SecretKey = k256::SecretKey::from_slice(value.1)
                    .map_err(|e| PrivateKeyError::ParseError(ParseError::Secp256k1(e)))?;
                SecretKey::Secp256k1(sk)
            }
            SecretKey::X25519(_) => {
                let mut sk_bytes = [0u8; 32];
                sk_bytes.copy_from_slice(value.1);
                let sk: x25519_dalek::StaticSecret = x25519_dalek::StaticSecret::from(sk_bytes);

                SecretKey::X25519(sk)
            }
        };

        Ok(sk)
    }
}

/// TryFrom<JWK> for a SecretKey implements the converstion of a (private) JWK into a SecretKey
impl TryFrom<JWK> for SecretKey {
    type Error = Error;
    fn try_from(jwk: JWK) -> Result<SecretKey, Self::Error> {
        match jwk.params {
            Params::EC(ecparams) => {
                let sk: k256::SecretKey = (&ecparams)
                    .try_into()
                    .map_err(PrivateKeyError::PrivateKeyError)?;

                Ok(SecretKey::Secp256k1(sk))
            }
            Params::OKP(okpparams) => {
                if okpparams.curve.to_lowercase() == "x25519" {
                    let sk: [u8; 32] = match okpparams.private_key.clone() {
                        Some(sk) => {
                            let mut sk_bytes = [0u8; 32];
                            sk_bytes.copy_from_slice(sk.0.as_slice());
                            sk_bytes
                        }
                        None => {
                            return Err(Error::SecretKeyError("Missing private key".to_string()))
                        }
                    };

                    Ok(SecretKey::X25519(x25519_dalek::StaticSecret::from(sk)))
                } else if okpparams.curve.to_lowercase() == "ed25519" {
                    let edsk: ed25519_dalek::SigningKey = (&okpparams)
                        .try_into()
                        .map_err(PrivateKeyError::PrivateKeyError)?;

                    Ok(SecretKey::X25519(x25519_dalek::StaticSecret::from(
                        edsk.to_scalar_bytes(),
                    )))
                } else {
                    Err(Error::SecretKeyError(format!(
                        "Unsupported curve type: {}",
                        okpparams.curve
                    )))
                }
            }
            _ => Err(Error::SecretKeyError("Unsupported key type".to_string())),
        }
    }
}

impl From<ed25519_dalek::SigningKey> for SecretKey {
    fn from(sk: ed25519_dalek::SigningKey) -> Self {
        SecretKey::X25519(x25519_dalek::StaticSecret::from(sk.to_scalar_bytes()))
    }
}

impl TryFrom<SecretKey> for JWK {
    type Error = Error;
    fn try_from(sk: SecretKey) -> Result<JWK, Self::Error> {
        sk.jwk()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ssi_jwk::JWK;
    use std::convert::TryInto;

    #[test]
    fn test_secret_key() {
        let sk = SecretKey::Secp256k1(k256::SecretKey::random(&mut rand::thread_rng()));
        let jwk: JWK = sk.jwk().unwrap();
        let sk2: SecretKey = jwk.try_into().unwrap();
        assert_eq!(sk, sk2);

        let sk = SecretKey::X25519(x25519_dalek::StaticSecret::random_from_rng(
            rand::thread_rng(),
        ));
        let jwk: JWK = sk.jwk().unwrap();
        let sk2: SecretKey = jwk.try_into().unwrap();
        assert_eq!(sk, sk2);

        let sk: SecretKey = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()).into();
        let jwk: JWK = sk.jwk().unwrap();
        let sk2: SecretKey = jwk.try_into().unwrap();
        assert_eq!(sk, sk2);
    }
}
