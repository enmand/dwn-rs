use std::{collections::TryReserveError, convert::Infallible};

use thiserror::Error;
use ulid::MonotonicError;

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
    MessageEncodeError(#[from] ipld_core::serde::SerdeError),

    #[error("failed to decode message: {0}")]
    MessageDecodeError(#[source] ipld_core::serde::SerdeError),

    #[error("failed to serde encode message: {0}")]
    SerdeEncodeError(#[from] serde_ipld_dagcbor::error::EncodeError<TryReserveError>),

    #[error("failed to serde decode message: {0}")]
    SerdeDecodeError(#[from] serde_ipld_dagcbor::error::DecodeError<Infallible>),

    #[error("failed to encode cid")]
    CidEncodeError(#[from] ipld_core::cid::Error),

    #[error("failed to decode cid")]
    CidDecodeError(#[source] ipld_core::cid::Error),

    #[error("unable to perform query: {0}")]
    QueryError(#[from] QueryError),

    #[error("unable to create filters: {0}")]
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

    #[error("unable to create filters: {0}")]
    FilterError(#[from] FilterError),

    #[error("unable to perform query: {0}")]
    QueryError(#[from] QueryError),

    #[error("unable to generate watermark: {0}")]
    WatermarkError(#[from] MonotonicError),
}

#[derive(Error, Debug)]
pub enum ResumableTaskStoreError {
    #[error("error operating the store: {0}")]
    StoreError(#[from] StoreError),

    #[error("unable to perform query: {0}")]
    QueryError(#[from] QueryError),

    #[error("unable to generate task id: {0}")]
    IdGenerationError(#[from] MonotonicError),

    #[error("unable to create filters: {0}")]
    FilterError(#[from] FilterError),

    #[error("unable to decode task id: {0}")]
    TaskIdDecodeError(#[from] ulid::DecodeError),
}
