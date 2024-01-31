use thiserror::Error;

use crate::{FilterError, QueryError};

#[derive(Error, Debug)]
pub enum MessageStoreError {
    #[error("error opening database: {0}")]
    OpenError(String),

    #[error("no database initialized")]
    NoInitError,

    #[error("Put error: {0}")]
    StoreException(String),

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

    #[error("unable to find record")]
    NotFound,

    #[error("unable to perform query")]
    QueryError(#[from] QueryError),

    #[error("unable to create filters")]
    FilterError(#[from] FilterError),
}
