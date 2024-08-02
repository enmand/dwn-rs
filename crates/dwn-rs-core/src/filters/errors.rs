use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValueError {
    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error("unparseable value: {0}")]
    UnparseableValue(String),
}

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
    #[error("unable to create filter: {0}")]
    UnparseableFilter(String),

    #[error("invalid value in filter: {0}")]
    Value(#[from] ValueError),
}
