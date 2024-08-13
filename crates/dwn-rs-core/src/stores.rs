use std::{fmt::Debug, pin::Pin};

use async_trait::async_trait;
use futures_util::Stream;
use ipld_core::cid::Cid;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    errors::{DataStoreError, EventLogError, MessageStoreError, ResumableTaskStoreError},
    filters::filter_key::Filters,
    Cursor, MessageSort, Pagination, QueryReturn,
};
use crate::{MapValue, Message};

#[async_trait]
pub trait MessageStore: Default {
    async fn open(&mut self) -> Result<(), MessageStoreError>;

    async fn close(&mut self);

    async fn put(
        &self,
        tenant: &str,
        message: Message,
        indexes: MapValue,
        tags: MapValue,
    ) -> Result<Cid, MessageStoreError>;

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, MessageStoreError>;

    async fn query(
        &self,
        tenant: &str,
        filter: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> Result<QueryReturn<Message>, MessageStoreError>;

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), MessageStoreError>;

    async fn clear(&self) -> Result<(), MessageStoreError>;
}

#[async_trait]
pub trait DataStore: Default {
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

#[async_trait]
pub trait EventLog: Default {
    async fn open(&mut self) -> Result<(), EventLogError>;

    async fn close(&mut self);

    async fn append(
        &self,
        tenant: &str,
        cid: String,
        indexes: MapValue,
        tags: MapValue,
    ) -> Result<(), EventLogError>;

    async fn get_events(
        &self,
        tenant: &str,
        cursor: Option<Cursor>,
    ) -> Result<QueryReturn<String>, EventLogError>;

    async fn query_events(
        &self,
        tenant: &str,
        filter: Filters,
        cursor: Option<Cursor>,
    ) -> Result<QueryReturn<String>, EventLogError>;

    async fn delete<'a>(&self, tenant: &str, cid: &'a [&str]) -> Result<(), EventLogError>;

    async fn clear(&self) -> Result<(), EventLogError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManagedResumableTask<T: Serialize + Sync + Send + Debug> {
    pub id: Ulid,
    pub task: T,
    pub timeout: u64,
    pub retry_count: u64,
}

#[async_trait]
pub trait ResumableTaskStore: Default {
    async fn open(&mut self) -> Result<(), ResumableTaskStoreError>;

    async fn close(&mut self);

    async fn register<T: Serialize + Send + Sync + DeserializeOwned + Debug>(
        &self,
        task: T,
        timeout: u64,
    ) -> Result<ManagedResumableTask<T>, ResumableTaskStoreError>;

    async fn grab<T: Serialize + Send + Sync + DeserializeOwned + Debug + Unpin>(
        &self,
        count: u64,
    ) -> Result<Vec<ManagedResumableTask<T>>, ResumableTaskStoreError>;

    async fn read<T: Serialize + Send + Sync + DeserializeOwned + Debug>(
        &self,
        task_id: String,
    ) -> Result<Option<ManagedResumableTask<T>>, ResumableTaskStoreError>;

    async fn extend(&self, task_id: String, timeout: u64) -> Result<(), ResumableTaskStoreError>;

    async fn delete(&self, task_id: String) -> Result<(), ResumableTaskStoreError>;

    async fn clear(&self) -> Result<(), ResumableTaskStoreError>;
}
