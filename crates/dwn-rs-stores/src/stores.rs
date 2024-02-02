use std::io;

use async_trait::async_trait;
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

    async fn put(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
        value: impl io::Write + Send + Sync,
    ) -> Result<PutDataResults, DataStoreError>;

    async fn get<T: io::Read + Send + Sync>(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
    ) -> Result<Option<GetDataResults<T>>, DataStoreError>;

    async fn delete(&self, tenant: &str, record_id: String, cid: Cid)
        -> Result<(), DataStoreError>;

    async fn clear(&self) -> Result<(), DataStoreError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutDataResults {
    #[serde(rename = "dataSize")]
    size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDataResults<T>
where
    T: io::Read + Send + Sync,
{
    #[serde(rename = "dataSize")]
    size: usize,

    #[serde(rename = "dataStream")]
    data: T,
}
