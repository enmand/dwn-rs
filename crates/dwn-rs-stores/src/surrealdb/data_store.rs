use async_trait::async_trait;
use libipld::Cid;
use surrealdb::sql::{Id, Table, Thing};
use tokio::io::{AsyncRead, AsyncReadExt};

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

    async fn put<T: AsyncRead + Send + Sync + Unpin>(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
        mut value: T,
    ) -> Result<PutDataResults, DataStoreError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        let mut buf = Vec::new();
        value.read_to_end(&mut buf).await?;

        let res = self
            .db
            .create::<Option<GetData>>(id)
            .content(CreateData {
                cid: cid.to_string(),
                data: buf,
                tenant: tenant.to_string(),
                record_id: record_id.to_string(),
            })
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(DataStoreError::from)?;

        Ok(PutDataResults {
            size: res.unwrap().data.len(),
        })
    }

    async fn get(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
    ) -> Result<Option<GetDataResults>, DataStoreError> {
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

        Ok(Some(GetDataResults {
            size: res.data.len(),
            data: res.data,
        }))
    }

    async fn delete(
        &self,
        tenant: &str,
        record_id: String,
        cid: Cid,
    ) -> Result<(), DataStoreError> {
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
