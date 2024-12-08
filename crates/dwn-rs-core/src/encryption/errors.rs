use thiserror::Error;

use super::asymmetric::secretkey::Error as AsymmetricSecretKeyError;
use super::hd_keys::Error as HDKeysError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error getting JWK secret: {0}")]
    JWKSecretKeyError(#[from] AsymmetricSecretKeyError),
    #[error("Error deriving key: {0}")]
    DeriveKeyError(#[from] HDKeysError),
}
