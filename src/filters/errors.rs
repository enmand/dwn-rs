use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("unable to create query: {0}")]
    DbError(String),

    #[error("unable to create filter: {0}")]
    FilterError(#[source] FilterError),
}

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("unable to create filter")]
    UnparseableFilter(String),
}
