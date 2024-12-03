pub mod publickey;
pub mod secretkey;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum ECIESError {
    #[error("Invalid HKDF key length: {0}")]
    InvalidHKDFKeyLength(hkdf::InvalidLength),
}

pub use publickey::PublicKey;
pub use secretkey::{ParseError, PrivateKeyError, SecretKey};
