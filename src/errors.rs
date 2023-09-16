use thiserror::Error;

#[derive(Error, Debug)]
pub enum SurrealDBError {
    #[error("SurrealDBError: {0}")]
    ConnectionError(#[from] surrealdb::Error),

    #[error("no database initialized")]
    NoInitError,

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
}
