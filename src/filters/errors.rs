use thiserror::Error;

use crate::ValueError;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("unable to create query: {0}")]
    DbError(String),

    #[error("unable to create filter: {0}")]
    FilterError(#[from] FilterError),

    #[error("unable to fetch from cursor: {0}")]
    CursorError(String),

    #[error("unable to fetch from cursor: {0}")]
    ValueError(#[from] ValueError),
}

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("unable to create filter")]
    UnparseableFilter(String),
}
