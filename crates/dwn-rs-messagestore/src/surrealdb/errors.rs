use dwn_rs_stores::{MessageStoreError, QueryError, ValueError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SurrealDBError {
    #[error("Database error: {0}")]
    DBError(#[from] surrealdb::error::Db),

    #[error("Surreal error: {0}")]
    SurrealError(#[from] surrealdb::Error),
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

impl From<SurrealDBError> for MessageStoreError {
    fn from(e: SurrealDBError) -> Self {
        match e {
            SurrealDBError::DBError(e) => Self::StoreException(e.to_string()),
            SurrealDBError::SurrealError(e) => Self::StoreException(e.to_string()),
        }
    }
}
