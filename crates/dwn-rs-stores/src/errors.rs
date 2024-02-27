use thiserror::Error;

use crate::{FilterError, QueryError};

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("error opening database: {0}")]
    OpenError(String),

    #[error("no database initialized")]
    NoInitError,

    #[error("internal store error: {0}")]
    InternalException(String),

    #[error("unable to find record")]
    NotFound,
}

#[derive(Error, Debug)]
pub enum MessageStoreError {
    #[error("error operating the store: {0}")]
    StoreError(#[from] StoreError),

    #[error("failed to encode message: {0}")]
    MessageEncodeError(#[from] libipld_core::error::Error),

    #[error("failed to decode message: {0}")]
    MessageDecodeError(#[source] libipld_core::error::Error),

    #[error("failed to serde encode message: {0}")]
    SerdeEncodeError(#[from] libipld_core::error::SerdeError),

    #[error("failed to serde decode message: {0}")]
    SerdeDecodeError(#[source] libipld_core::error::SerdeError),

    #[error("failed to encode cid")]
    CidEncodeError(#[from] libipld_core::cid::Error),

    #[error("failed to decode cid")]
    CidDecodeError(#[source] libipld_core::cid::Error),

    #[error("unable to perform query")]
    QueryError(#[from] QueryError),

    #[error("unable to create filters")]
    FilterError(#[from] FilterError),
}

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("error opening database: {0}")]
    OpenError(String),

    #[error("no database initialized")]
    NoInitError,

    #[error("error operating the store: {0}")]
    StoreError(#[from] StoreError),

    #[error("unable to read data from buffer")]
    ReadError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum EventLogError {
    #[error("error operating the store: {0}")]
    StoreError(#[from] StoreError),

    #[error("unable to create filters")]
    FilterError(#[from] FilterError),

    #[error("unable to perform query")]
    QueryError(#[from] QueryError),
}
