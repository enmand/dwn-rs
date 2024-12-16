use thiserror::Error;

use crate::jsonrpc::JSONRpcError;

pub type Result<T> = std::result::Result<T, RemoteError>;

#[derive(Error, Debug)]
pub enum RemoteError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("error: {0}")]
    Error(String),
    #[error("jsonrpc error: {0}")]
    JSONRpcError(#[from] JSONRpcError),
}
