use std::fmt::Debug;

use aes::cipher::generic_array::GenericArray;
use k256::{elliptic_curve::sec1::ToEncodedPoint, sha2};
use ssi_jwk::{secp256k1_parse_private, JWK};
use tracing::error;
use typenum::U33;

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

    fn from_bytes(bytes: GenericArray<u8, Self::KeySize>) -> Result<Self, Error> {
        let pk = k256::PublicKey::from_sec1_bytes(&bytes).map_err(PublicKeyError::CurveError)?;
        Ok(Self { pk })
    }

    fn to_bytes(&self) -> GenericArray<u8, Self::KeySize> {
        let v = self.pk.to_encoded_point(false).to_bytes().to_vec();
        GenericArray::from_iter(v[..32].iter().copied())
    }

    fn jwk(&self) -> JWK {
        self.pk.into()
    }

    fn decapsulate(self, sk: Self::SecretKey) -> Result<GenericArray<u8, Self::KeySize>, Error> {
        sk.encapsulate(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SecretKey {
    sk: k256::SecretKey,
}

impl DeriveKey for SecretKey {}

impl SecretKeyTrait for SecretKey {
    type KeySize = U33;
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

    fn encapsulate(self, pk: Self::PublicKey) -> Result<GenericArray<u8, Self::KeySize>, Error> {
        let mut okm: GenericArray<u8, Self::KeySize> = GenericArray::default();

        k256::ecdh::diffie_hellman(self.sk.to_nonzero_scalar(), pk.pk.as_affine())
            .extract::<sha2::Sha256>(None)
            .expand(&[], &mut okm)
            .map_err(ECIESError::InvalidHKDFKeyLength)?;

        Ok(okm)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Error> {
        todo!();
    }
}

impl From<k256::SecretKey> for SecretKey {
    fn from(sk: k256::SecretKey) -> Self {
        SecretKey { sk }
    }
}
