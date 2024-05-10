use async_stream::stream;
use async_trait::async_trait;
use futures_util::{pin_mut, Stream, StreamExt};
use surrealdb::sql::{Id, Table, Thing};

use crate::{
    DataStore, DataStoreError, GetDataResults, PutDataResults, StoreError, SurrealDB,
    SurrealDBError,
};

use super::models::{CreateData, GetData};

const DATA_TABLE: &str = "data";

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

        let id = Thing::from((DATA_TABLE, Id::String(record_id.to_string())));

        let len = self
            .as_tenant(tenant, |db| async move {
                db.create::<Option<GetData>>(id.clone())
                    .content(CreateData {
                        cid: cid.to_string(),
                        data: Vec::new(),
                        tenant: tenant.to_string(),
                        record_id: record_id.to_string(),
                    })
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)?;

                let mut len = 0;
                while let Some(chunk) = value.next().await {
                    let u = db
                        .update::<Option<GetData>>(id.clone())
                        .merge(CreateData {
                            cid: cid.to_string(),
                            data: chunk,
                            tenant: tenant.to_string(),
                            record_id: record_id.to_string(),
                        })
                        .await
                        .map_err(SurrealDBError::from)
                        .map_err(StoreError::from)?;

                    len += u.unwrap().data.len();
                }

                Ok(len)
            })
            .await?;

        Ok(PutDataResults { size: len })
    }

    async fn get(
        &self,
        tenant: &str,
        record_id: String,
        _: String,
    ) -> Result<GetDataResults, DataStoreError> {
        let id = Thing::from((DATA_TABLE, Id::String(record_id.to_string())));

        let res: GetData = self
            .as_tenant(tenant, |db| async move {
                db.select(id)
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)
                    .expect("failed to fetch from database")
                    .ok_or(StoreError::NotFound)
            })
            .await?;

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

    async fn delete(
        &self,
        tenant: &str,
        record_id: String,
        _: String,
    ) -> Result<(), DataStoreError> {
        let id = Thing::from((DATA_TABLE, Id::String(record_id.to_string())));

        self.as_tenant(tenant, |db| async move {
            db.delete::<Option<GetData>>(id)
                .await
                .map_err(SurrealDBError::from)
                .map_err(StoreError::from)
        })
        .await?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), DataStoreError> {
        self.clear(&Table::from(DATA_TABLE))
            .await
            .map_err(DataStoreError::from)
    }
}
