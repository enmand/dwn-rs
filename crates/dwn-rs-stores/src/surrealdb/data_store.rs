use async_stream::stream;
use async_trait::async_trait;
use futures_util::{pin_mut, Stream, StreamExt};
use surrealdb::sql::{Id, Table, Thing};

use crate::{
    DataStore, DataStoreError, GetDataResults, PutDataResults, StoreError, SurrealDB,
    SurrealDBError,
};

use super::models::{CreateData, GetData};

#[async_trait]
impl DataStore for SurrealDB {
    async fn open(&mut self) -> Result<(), DataStoreError> {
        self.open().await.map_err(DataStoreError::from)
    }

    async fn close(&mut self) {
        self.close().await
    }

    async fn put<T>(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
        value: T,
    ) -> Result<PutDataResults, DataStoreError>
    where
        T: Stream<Item = Vec<u8>> + Unpin + Send,
    {
        pin_mut!(value);

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        self.db
            .create::<Option<GetData>>(id.clone())
            .content(CreateData {
                cid: cid.to_string(),
                data: Vec::new(),
                tenant: tenant.to_string(),
                record_id: record_id.to_string(),
            })
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(DataStoreError::from)?;

        let mut len = 0;
        while let Some(chunk) = value.next().await {
            let u = self
                .db
                .update::<Option<GetData>>(id.clone())
                .merge(CreateData {
                    cid: cid.to_string(),
                    data: chunk,
                    tenant: tenant.to_string(),
                    record_id: record_id.to_string(),
                })
                .await
                .map_err(SurrealDBError::from)
                .map_err(StoreError::from)
                .map_err(DataStoreError::from)?;

            len += u.unwrap().data.len();
        }

        Ok(PutDataResults { size: len })
    }

    async fn get(
        &self,
        tenant: &str,
        record_id: String,
        cid: String,
    ) -> Result<GetDataResults, DataStoreError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        let res: GetData = self
            .db
            .select(id)
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(DataStoreError::from)?
            .ok_or(DataStoreError::StoreError(StoreError::NotFound))?;

        if res.record_id != record_id {
            return Err(DataStoreError::StoreError(StoreError::NotFound));
        }

        let size = res.data.len();
        let s = stream! {
            yield res.data;
        };

        Ok(GetDataResults {
            size,
            data: Box::pin(s),
        })
    }

    async fn delete(&self, tenant: &str, _: String, cid: String) -> Result<(), DataStoreError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        self.db
            .delete::<Option<GetData>>(id)
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(DataStoreError::from)?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), DataStoreError> {
        self.clear().await.map_err(DataStoreError::from)
    }
}
