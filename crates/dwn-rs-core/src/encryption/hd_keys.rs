use k256::{sha2, SecretKey};
use ssi_jwk::{secp256k1_parse_private, Params, JWK};

use super::DerivationScheme;
use thiserror::Error;

const HKDF_KEY_LENGTH: usize = 32; // * 8; // 32 bytes = 256 bits

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error getting secret key: {0}")]
    SecretKeyError(#[from] ssi_jwk::Error),
    #[error("Error deriving key, bad key length: {0}")]
    DeriveKeyLengthError(hkdf::InvalidLength),
    #[error("Error deriving key: {0}")]
    DeriveKeyError(#[from] k256::elliptic_curve::Error),
    #[error("Error encoding key: {0}")]
    EncodeError(#[from] k256::pkcs8::der::Error),
    #[error("Invalid path segment: {0}")]
    InvalidPathSegment(String),
    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedHashAlgorithm(String),
    #[error("Unsupported key type")]
    UnsupportedKeyType,
}

/// DerivedPrivateJWK represents a derived private JWK, which includes the root key ID, derivation
/// scheme, derivation path, and the key itself. This is used for encrypting records with keys
/// derived from a root key.
#[derive(Debug)]
pub struct DerivedPrivateJWK {
    pub root_key_id: String,
    pub scheme: DerivationScheme,
    pub path: Option<Vec<String>>,
    pub key: JWK,
}

/// HashAlgorithm represents the hash algorithm used for key derivation.
#[derive(PartialEq)]
pub enum HashAlgorithm {
    SHA256,
    SHA384,
    SHA512,
}

impl DerivedPrivateJWK {
    /// derive derives a new private key from the root key using the derivation path.
    pub fn derive(
        ancestor_key: DerivedPrivateJWK,
        derivation_path: Vec<String>,
    ) -> Result<DerivedPrivateJWK, Error> {
        let path: &[&str] = &derivation_path
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        if let Params::EC(ecparam) = ancestor_key.key.params {
            // TODO support x25519
            let sk: k256::SecretKey = (&ecparam).try_into()?;
            let ancestor_path = ancestor_key.path.unwrap_or_default();

            let derived_key = Self::derive_private_key(&sk, path)?;

            let mut pk: JWK = sk.public_key().into();
            let derived_jwk = secp256k1_parse_private(&derived_key.to_sec1_der()?)?;
            pk.params = derived_jwk.params.clone();

            return Ok(DerivedPrivateJWK {
                root_key_id: ancestor_key.root_key_id,
                scheme: ancestor_key.scheme,
                path: Some([ancestor_path, derivation_path].concat()),
                key: pk,
            });
        };

        Err(Error::UnsupportedKeyType)
    }

    pub fn derive_public_key(
        ancestor_key: DerivedPrivateJWK,
        derivation_path: &[&str],
    ) -> Result<JWK, Error> {
        if let Params::EC(ecparam) = ancestor_key.key.params {
            // TODO support x25519
            let sk: k256::SecretKey = (&ecparam).try_into()?;

            let derived_key = Self::derive_private_key(&sk, derivation_path)?;
            let derived_jwk = derived_key.public_key().into();

            return Ok(derived_jwk);
        }

        Err(Error::UnsupportedKeyType)
    }
    pub fn derive_private_key(
        ancestor_key: &SecretKey,
        derivation_path: &[&str],
    ) -> Result<SecretKey, Error> {
        Self::validate_path(derivation_path)?;

        let sk = derivation_path.iter().try_fold(
            ancestor_key.to_owned(),
            |key, segment| -> Result<SecretKey, Error> {
                let seg = segment.as_bytes();
                let key_material = key.to_bytes();
                Self::derive_hkdf_key(HashAlgorithm::SHA256, &key_material, seg)
            },
        )?;

        Ok(sk)
    }

    pub fn derive_hkdf_key(
        hash_algo: HashAlgorithm,
        initial_key_material: &[u8],
        info: &[u8],
    ) -> Result<SecretKey, Error> {
        if hash_algo != HashAlgorithm::SHA256 {
            // TODO support more algorithms
            return Err(Error::UnsupportedHashAlgorithm(
                "Unsupported hash algorithm".to_string(),
            ));
        }

        let mut okm = [0u8; HKDF_KEY_LENGTH];

        hkdf::Hkdf::<sha2::Sha256>::new(None, initial_key_material)
            .expand(info, &mut okm)
            .map_err(Error::DeriveKeyLengthError)?;

        Ok(SecretKey::from_slice(&okm)?)
    }

    fn validate_path(path: &[&str]) -> Result<(), Error> {
        // check if any path segments are empty
        if path.iter().any(|s| s.is_empty()) {
            return Err(Error::InvalidPathSegment("Empty path segment".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssi_jwk::JWK;

    #[test]
    fn test_derive() {
        let root_key = JWK::generate_secp256k1();
        let root_key_id = "root_key_id".to_string();
        let scheme = DerivationScheme::ProtocolPath;
        let path = &["path"];
        let derived_key = DerivedPrivateJWK {
            root_key_id: root_key_id.clone(),
            scheme,
            path: Some(path.iter().map(|s| s.to_string()).collect()),
            key: root_key,
        };
        let derived = DerivedPrivateJWK::derive(derived_key, vec!["path2".to_string()]).unwrap();

        assert_eq!(derived.root_key_id, root_key_id);
        assert_eq!(derived.scheme, DerivationScheme::ProtocolPath);
        assert_eq!(
            derived.path,
            Some(vec!["path".to_string(), "path2".to_string()])
        );
    }

    #[test]
    fn test_derive_public_key() {
        let root_key = JWK::generate_secp256k1();
        let root_key_id = "root_key_id".to_string();
        let scheme = DerivationScheme::ProtocolPath;
        let path = &["path"];
        let derived_key = DerivedPrivateJWK {
            root_key_id: root_key_id.clone(),
            scheme,
            path: Some(path.iter().map(|s| s.to_string()).collect()),
            key: root_key.clone(),
        };

        let derived = DerivedPrivateJWK::derive_public_key(derived_key, path).unwrap();

        assert!(derived.params.is_public());
    }

    #[test]
    fn test_derive_ancestor_chain_path() {
        let root_key = k256::SecretKey::random(&mut rand::thread_rng());

        let path_to_g = ["a", "b", "c", "d", "e", "f", "g"].as_slice();
        let path_to_d = ["a", "b", "c", "d"].as_slice();
        let path_e_to_g = ["e", "f", "g"].as_slice();

        let keyg = DerivedPrivateJWK::derive_private_key(&root_key, path_to_g).unwrap();
        let keyd = DerivedPrivateJWK::derive_private_key(&root_key, path_to_d).unwrap();
        let keydg = DerivedPrivateJWK::derive_private_key(&keyd, path_e_to_g).unwrap();

        assert_eq!(keyg, keydg);
        assert_ne!(keyg, keyd);
    }

    #[test]
    fn test_invalid_path() {
        let root_key = k256::SecretKey::random(&mut rand::thread_rng());
        let path = ["a", "", "c"].as_slice();

        let result = DerivedPrivateJWK::derive_private_key(&root_key, path);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid path segment: Empty path segment"
        );
    }
}
