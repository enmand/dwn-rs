use thiserror::Error;

use super::asymmetric;
use super::hd_keys::Error as HDKeysError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error getting JWK secret: {0}")]
    JWKSecretKeyError(#[from] asymmetric::Error),
    #[error("Error deriving key: {0}")]
    DeriveKeyError(#[from] HDKeysError),
}
