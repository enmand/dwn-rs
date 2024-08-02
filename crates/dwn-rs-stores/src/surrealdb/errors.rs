use crate::surrealdb::auth::AuthError;
use dwn_rs_core::{
    errors::StoreError,
    filters::errors::{QueryError, ValueError},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SurrealDBError {
    #[error("Database error: {0}")]
    DBError(#[from] surrealdb::error::Db),

    #[error("Surreal error: {0}")]
    SurrealError(#[from] surrealdb::Error),

    #[error("no namespace provided")]
    NoNamespace,

    #[error("authentication error: {0}")]
    AuthError(#[from] AuthError),
}

impl From<SurrealDBError> for QueryError {
    fn from(e: SurrealDBError) -> Self {
        Self::DbError(e.to_string())
    }
}

impl From<SurrealDBError> for ValueError {
    fn from(e: SurrealDBError) -> Self {
        Self::InvalidValue(e.to_string())
    }
}

impl From<SurrealDBError> for StoreError {
    fn from(e: SurrealDBError) -> Self {
        match e {
            SurrealDBError::DBError(e) => Self::InternalException(e.to_string()),
            SurrealDBError::SurrealError(e) => Self::InternalException(e.to_string()),
            SurrealDBError::NoNamespace => Self::InternalException(e.to_string()),
            SurrealDBError::AuthError(e) => Self::InternalException(e.to_string()),
        }
    }
}
