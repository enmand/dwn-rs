use ssi_jwk::{Params, JWK};

use crate::encryption::HashAlgorithm;

use super::{publickey::PublicKey, secp256k1, x25519, Error, PrivateKeyError, SecretKeyTrait, DeriveKey};

#[derive(Debug, PartialEq, Clone)]
pub enum SecretKey {
    Secp256k1(secp256k1::SecretKey),
    X25519(x25519::SecretKey),
}

impl SecretKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            SecretKey::Secp256k1(sk) => sk.to_bytes().to_vec(),
            SecretKey::X25519(sk) => sk.to_bytes().to_vec(),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        match self {
            SecretKey::Secp256k1(sk) => PublicKey::Secp256k1(sk.public_key()),
            SecretKey::X25519(sk) => PublicKey::X25519(sk.public_key()),
        }
    }

    pub fn jwk(&self) -> Result<JWK, Error> {
        match self {
            SecretKey::Secp256k1(sk) => sk.jwk(),
            SecretKey::X25519(sk) => sk.jwk(),
        }
    }

    pub fn encapsulate(self, pk: PublicKey) -> Result<Vec<u8>, Error> {
        match (self, pk) {
            (SecretKey::Secp256k1(sk), PublicKey::Secp256k1(pk)) => {
                sk.encapsulate(&pk).map(|ga| ga.to_vec())
            }
            (SecretKey::X25519(sk), PublicKey::X25519(pk)) => {
                sk.encapsulate(&pk).map(|ga| ga.to_vec())
            }
            _ => Err(Error::SecretKeyError("Invalid key pair".to_string())),
        }
    }

    pub fn derive_hkdf(
        &self,
        hash_algo: HashAlgorithm,
        salt: &[u8],
        info: &[u8],
    ) -> Result<Self, Error> {
        match self {
            SecretKey::Secp256k1(sk) => {
                Ok(Self::Secp256k1(sk.derive_hkdf_key(hash_algo, salt, info)?))
            }
            SecretKey::X25519(sk) => Ok(Self::X25519(sk.derive_hkdf_key(hash_algo, salt, info)?)),
        }
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            SecretKey::Secp256k1(sk) => sk.decrypt(data),
            SecretKey::X25519(sk) => sk.decrypt(data),
        }
    }
}

impl From<SecretKey> for secp256k1::SecretKey {
    fn from(sk: SecretKey) -> Self {
        match sk {
            SecretKey::Secp256k1(sk) => sk,
            _ => panic!("Invalid conversion"),
        }
    }
}

impl From<SecretKey> for x25519::SecretKey {
    fn from(sk: SecretKey) -> Self {
        match sk {
            SecretKey::X25519(sk) => sk,
            _ => panic!("Invalid conversion"),
        }
    }
}

impl TryFrom<SecretKey> for JWK {
    type Error = Error;
    fn try_from(sk: SecretKey) -> Result<JWK, Self::Error> {
        match sk {
            SecretKey::Secp256k1(sk) => sk.jwk(),
            SecretKey::X25519(sk) => sk.jwk(),
        }
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

                Ok(SecretKey::Secp256k1(sk.into()))
            }
            Params::OKP(okpparams) => {
                if okpparams.curve.to_lowercase() == "x25519" {
                    let sk: x25519_dalek::StaticSecret = match okpparams.private_key.clone() {
                        Some(sk) => {
                            let mut sk_bytes = [0u8; 32];
                            sk_bytes.copy_from_slice(sk.0.as_slice());
                            sk_bytes.into()
                        }
                        None => {
                            return Err(Error::SecretKeyError("Missing private key".to_string()))
                        }
                    };

                    Ok(SecretKey::X25519(sk.into()))
                } else if okpparams.curve.to_lowercase() == "ed25519" {
                    let edsk: ed25519_dalek::SigningKey = (&okpparams)
                        .try_into()
                        .map_err(PrivateKeyError::PrivateKeyError)?;

                    Ok(SecretKey::X25519(edsk.into()))
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

#[cfg(test)]
mod test {
    use crate::encryption::asymmetric::SecretKeyTrait;

    use super::*;
    use ssi_jwk::JWK;
    use std::convert::TryInto;

    #[test]
    fn test_secret_key() {
        let sk: secp256k1::SecretKey = k256::SecretKey::random(&mut rand::thread_rng()).into();
        let jwk: JWK = sk.jwk().unwrap();
        let sk2: SecretKey = jwk.try_into().unwrap();
        assert_eq!(sk, sk2.into());

        let sk: x25519::SecretKey =
            x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng()).into();
        let jwk: JWK = sk.jwk().unwrap();
        let sk2: SecretKey = jwk.try_into().unwrap();
        assert_eq!(sk, sk2.into());

        let sk: x25519::SecretKey =
            ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()).into();
        let jwk: JWK = sk.jwk().unwrap();
        let sk2: SecretKey = jwk.try_into().unwrap();
        assert_eq!(sk, sk2.into());
    }
}
