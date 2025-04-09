use ssi_jwk::JWK;

use super::{
    asymmetric::{self, secretkey::SecretKey},
    DerivationScheme,
};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Error getting JWK secret key: {0}")]
    JWKSecretKeyError(#[from] ssi_jwk::Error),
    #[error("Error getting SecretKey from bytes: {0}")]
    SecretKeyError(#[from] asymmetric::Error),
    #[error("Error deriving key: {0}")]
    DeriveKeyError(#[from] k256::elliptic_curve::Error),
    #[error("Error encoding key: {0}")]
    EncodeError(#[from] k256::pkcs8::der::Error),
    #[error("Invalid path segment: {0}")]
    InvalidPathSegment(String),
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

        let sk: SecretKey = ancestor_key.key.try_into()?;
        let ancestor_path = ancestor_key.path.unwrap_or_default();
        let derived_key = Self::derive_secret(&sk, path)?;
        let pjwk: JWK = derived_key.jwk()?;

        Ok(DerivedPrivateJWK {
            root_key_id: ancestor_key.root_key_id,
            scheme: ancestor_key.scheme,
            path: Some([ancestor_path, derivation_path].concat()),
            key: pjwk,
        })
    }

    /// derive_public_key derives a new public key from the root key using the derivation path.
    pub fn derive_public_key(
        ancestor_key: DerivedPrivateJWK,
        derivation_path: Vec<String>,
    ) -> Result<JWK, Error> {
        let derived_key = Self::derive(ancestor_key, derivation_path)?;
        let sk: SecretKey = derived_key.key.try_into()?;

        Ok(sk.public_key().jwk())
    }

    pub fn derive_secret(
        ancestor_key: &SecretKey,
        derivation_path: &[&str],
    ) -> Result<SecretKey, Error> {
        Self::validate_path(derivation_path)?;

        let sk = derivation_path.iter().try_fold(
            ancestor_key.to_owned(),
            |key, segment| -> Result<SecretKey, Error> {
                let seg = segment.as_bytes();
                key.derive_hkdf(HashAlgorithm::SHA256, &[], seg)
                    .map_err(Error::SecretKeyError)
            },
        )?;

        Ok(sk)
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
    use tracing_test::traced_test;

    struct JWKTestTable {
        private_jwk: JWK,
    }

    struct SecretKeyTestTable {
        secret_key: SecretKey,
    }

    #[traced_test]
    #[test]
    fn test_derive() {
        let tcs = vec![
            JWKTestTable {
                private_jwk: JWK::generate_secp256k1(),
            },
            JWKTestTable {
                private_jwk: {
                    let sk = SecretKey::X25519(
                        x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng()).into(),
                    );
                    sk.try_into().unwrap()
                },
            },
            JWKTestTable {
                private_jwk: JWK::generate_ed25519().expect("unable to gnenerate ed25519 key"),
            },
        ];

        for tc in tcs {
            let root_key = tc.private_jwk.clone();
            let root_key_id = "root_key_id".to_string();
            let scheme = DerivationScheme::ProtocolPath;
            let path = vec!["path".to_string()];
            let derived_key = DerivedPrivateJWK {
                root_key_id: root_key_id.clone(),
                scheme,
                path: Some(path),
                key: root_key,
            };
            let derived =
                DerivedPrivateJWK::derive(derived_key, vec!["path2".to_string()]).unwrap();

            assert_eq!(derived.root_key_id, root_key_id);
            assert_eq!(derived.scheme, DerivationScheme::ProtocolPath);
            assert_eq!(
                derived.path,
                Some(vec!["path".to_string(), "path2".to_string()])
            );
        }
    }

    #[test]
    fn test_derive_public_key() {
        let tcs = vec![
            JWKTestTable {
                private_jwk: JWK::generate_secp256k1(),
            },
            JWKTestTable {
                private_jwk: {
                    let sk = SecretKey::X25519(
                        x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng()).into(),
                    );
                    sk.try_into().unwrap()
                },
            },
            JWKTestTable {
                private_jwk: JWK::generate_ed25519().expect("unable to gnenerate ed25519 key"),
            },
        ];

        for tc in tcs {
            let root_key = tc.private_jwk.clone();
            let root_key_id = "root_key_id".to_string();
            let scheme = DerivationScheme::ProtocolPath;
            let path = vec!["path".to_string()];
            let derived_key = DerivedPrivateJWK {
                root_key_id: root_key_id.clone(),
                scheme,
                path: Some(path.iter().map(|s| s.to_string()).collect()),
                key: root_key.clone(),
            };

            let derived = DerivedPrivateJWK::derive_public_key(derived_key, path).unwrap();

            assert!(derived.params.is_public());
        }
    }

    #[test]
    fn test_derive_ancestor_chain_path() {
        let tcs = vec![
            SecretKeyTestTable {
                secret_key: SecretKey::Secp256k1(
                    k256::SecretKey::random(&mut rand::thread_rng()).into(),
                ),
            },
            SecretKeyTestTable {
                secret_key: SecretKey::X25519(
                    x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng()).into(),
                ),
            },
            SecretKeyTestTable {
                secret_key: SecretKey::X25519(
                    ed25519_dalek::SigningKey::generate(&mut rand::thread_rng()).into(),
                ),
            },
        ];

        for tc in tcs {
            let root_key = tc.secret_key;

            let path_to_g = ["a", "b", "c", "d", "e", "f", "g"].as_slice();
            let path_to_d = ["a", "b", "c", "d"].as_slice();
            let path_e_to_g = ["e", "f", "g"].as_slice();

            let keyg = DerivedPrivateJWK::derive_secret(&root_key, path_to_g).unwrap();
            let keyd = DerivedPrivateJWK::derive_secret(&root_key, path_to_d).unwrap();
            let keydg = DerivedPrivateJWK::derive_secret(&keyd, path_e_to_g).unwrap();

            assert_eq!(keyg, keydg);
            assert_ne!(keyg, keyd);
        }
    }

    #[test]
    fn test_invalid_path() {
        let root_key: SecretKey =
            SecretKey::Secp256k1(k256::SecretKey::random(&mut rand::thread_rng()).into());
        let path = ["a", "", "c"].as_slice();

        let result = DerivedPrivateJWK::derive_secret(&root_key, path);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid path segment: Empty path segment"
        );
    }
}
