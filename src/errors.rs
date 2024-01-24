use thiserror::Error;

use crate::{FilterError, QueryError};

#[derive(Error, Debug)]
pub enum ValueError {
    #[error("invalid value: {0}")]
    InvalidValue(#[from] surrealdb::error::Db),

    #[error("invalid filter: {0}")]
    FiltersError(#[from] FilterError),

    #[error("unparseable value: {0}")]
    UnparseableValue(String),
}

impl From<ValueError> for FilterError {
    fn from(e: ValueError) -> Self {
        Self::UnparseableFilter(e.to_string())
    }
}

impl From<surrealdb::Error> for QueryError {
    fn from(e: surrealdb::Error) -> Self {
        Self::DbError(e.to_string())
    }
}

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

    #[error("unable to perform query")]
    QueryError(#[from] QueryError),

    #[error("unable to create filters")]
    FilterError(#[from] FilterError),
}
