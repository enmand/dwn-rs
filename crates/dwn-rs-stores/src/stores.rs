use async_trait::async_trait;
use libipld_core::cid::Cid;

use crate::{Filters, Indexes, MessageSort, MessageStoreError, Pagination, QueryReturn};
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
