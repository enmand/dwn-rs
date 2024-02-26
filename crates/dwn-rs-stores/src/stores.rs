use std::pin::Pin;

use async_trait::async_trait;
use futures_util::Stream;
use libipld_core::cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{
    DataStoreError, Filters, Indexes, MessageSort, MessageStoreError, Pagination, QueryReturn,
};
use dwn_rs_core::Message;

#[async_trait]
pub trait MessageStore {
    async fn open(&mut self) -> Result<(), MessageStoreError>;

    async fn close(&mut self);

    async fn put(
        &self,
        tenant: &str,
        message: Message,
        indexes: Indexes,
    ) -> Result<Cid, MessageStoreError>;

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, MessageStoreError>;

    async fn query(
        &self,
        tenant: &str,
        filter: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> Result<QueryReturn, MessageStoreError>;

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), MessageStoreError>;

    async fn clear(&self) -> Result<(), MessageStoreError>;
}

#[async_trait]
pub trait DataStore {
    async fn open(&mut self) -> Result<(), DataStoreError>;

    async fn close(&mut self);

    async fn put<T: Stream<Item = Vec<u8>> + Send + Unpin>(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
        value: T,
    ) -> Result<PutDataResults, DataStoreError>;

    async fn get(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
    ) -> Result<GetDataResults, DataStoreError>;
    async fn delete(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
    ) -> Result<(), DataStoreError>;

    async fn clear(&self) -> Result<(), DataStoreError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutDataResults {
    #[serde(rename = "dataSize")]
    pub size: usize,
}

pub struct GetDataResults {
    pub size: usize,
    pub data: Pin<Box<dyn Stream<Item = Vec<u8>>>>,
}
