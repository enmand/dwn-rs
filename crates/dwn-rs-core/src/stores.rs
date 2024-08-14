use std::{fmt::Debug, future::Future, pin::Pin};

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

pub trait MessageStore: Default {
    fn open(&mut self) -> impl Future<Output = Result<(), MessageStoreError>> + Send;

    fn close(&mut self) -> impl Future<Output = ()>;

    fn put(
        &self,
        tenant: &str,
        message: Message,
        indexes: MapValue,
        tags: MapValue,
    ) -> impl Future<Output = Result<Cid, MessageStoreError>> + Send;

    fn get(
        &self,
        tenant: &str,
        cid: String,
    ) -> impl Future<Output = Result<Message, MessageStoreError>> + Send;

    fn query(
        &self,
        tenant: &str,
        filter: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> impl Future<Output = Result<QueryReturn<Message>, MessageStoreError>> + Send;

    fn delete(
        &self,
        tenant: &str,
        cid: String,
    ) -> impl Future<Output = Result<(), MessageStoreError>> + Send;

    fn clear(&self) -> impl Future<Output = Result<(), MessageStoreError>> + Send;
}

pub trait DataStore: Default {
    fn open(&mut self) -> impl Future<Output = Result<(), DataStoreError>> + Send;

    fn close(&mut self) -> impl Future<Output = ()> + Send;

    fn put<T: Stream<Item = Vec<u8>> + Send + Unpin>(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
        value: T,
    ) -> impl Future<Output = Result<PutDataResults, DataStoreError>> + Send;

    fn get(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
    ) -> impl Future<Output = Result<GetDataResults, DataStoreError>> + Send;

    fn delete(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
    ) -> impl Future<Output = Result<(), DataStoreError>> + Send;

    fn clear(&self) -> impl Future<Output = Result<(), DataStoreError>> + Send;
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

pub trait EventLog: Default {
    fn open(&mut self) -> impl Future<Output = Result<(), EventLogError>> + Send;

    fn close(&mut self) -> impl Future<Output = ()>;

    fn append(
        &self,
        tenant: &str,
        cid: String,
        indexes: MapValue,
        tags: MapValue,
    ) -> impl Future<Output = Result<(), EventLogError>>;

    fn get_events(
        &self,
        tenant: &str,
        cursor: Option<Cursor>,
    ) -> impl Future<Output = Result<QueryReturn<String>, EventLogError>> + Send;

    fn query_events(
        &self,
        tenant: &str,
        filter: Filters,
        cursor: Option<Cursor>,
    ) -> impl Future<Output = Result<QueryReturn<String>, EventLogError>> + Send;

    fn delete<'a>(
        &self,
        tenant: &str,
        cid: &'a [&str],
    ) -> impl Future<Output = Result<(), EventLogError>> + Send;

    fn clear(&self) -> impl Future<Output = Result<(), EventLogError>> + Send;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManagedResumableTask<T: Serialize + Sync + Send + Debug> {
    pub id: Ulid,
    pub task: T,
    pub timeout: u64,
    pub retry_count: u64,
}

pub trait ResumableTaskStore: Default {
    fn open(&mut self) -> impl Future<Output = Result<(), ResumableTaskStoreError>> + Send;

    fn close(&mut self) -> impl Future<Output = ()> + Send;

    fn register<T: Serialize + Send + Sync + DeserializeOwned + Debug>(
        &self,
        task: T,
        timeout: u64,
    ) -> impl Future<Output = Result<ManagedResumableTask<T>, ResumableTaskStoreError>> + Send;

    fn grab<T: Serialize + Send + Sync + DeserializeOwned + Debug + Unpin>(
        &self,
        count: u64,
    ) -> impl Future<Output = Result<Vec<ManagedResumableTask<T>>, ResumableTaskStoreError>> + Send;

    fn read<T: Serialize + Send + Sync + DeserializeOwned + Debug>(
        &self,
        task_id: String,
    ) -> impl Future<Output = Result<Option<ManagedResumableTask<T>>, ResumableTaskStoreError>> + Send;

    fn extend(
        &self,
        task_id: String,
        timeout: u64,
    ) -> impl Future<Output = Result<(), ResumableTaskStoreError>> + Send;

    fn delete(
        &self,
        task_id: String,
    ) -> impl Future<Output = Result<(), ResumableTaskStoreError>> + Send;

    fn clear(&self) -> impl Future<Output = Result<(), ResumableTaskStoreError>> + Send;
}

}
