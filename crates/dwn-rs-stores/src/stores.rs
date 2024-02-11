use async_trait::async_trait;
use libipld_core::cid::Cid;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

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

    async fn put<T: AsyncRead + Send + Sync + Unpin>(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
        value: T,
    ) -> Result<PutDataResults, DataStoreError>;

    async fn get(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
    ) -> Result<Option<GetDataResults>, DataStoreError>;

    async fn delete(&self, tenant: &str, record_id: String, cid: Cid)
        -> Result<(), DataStoreError>;

    async fn clear(&self) -> Result<(), DataStoreError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutDataResults {
    #[serde(rename = "dataSize")]
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDataResults {
    #[serde(rename = "dataSize")]
    pub size: usize,

    #[serde(rename = "dataStream")]
    pub data: Vec<u8>,
}
