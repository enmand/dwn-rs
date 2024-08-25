use async_std::stream::{self, from_iter};
use futures_util::{pin_mut, Stream, StreamExt};
use surrealdb::sql::{
    statements::RelateStatement, Data, Id, Ident, Idiom, Operator, Param, Table, Thing, Value,
};
use tracing::Instrument;

use crate::{
    surrealdb::models::{DataChunk, DataChunkSize},
    SurrealDB, SurrealDBError,
};
use dwn_rs_core::{
    errors::{DataStoreError, StoreError},
    stores::{DataStore, GetDataResults, PutDataResults},
};

use super::models::{CreateData, GetData};

const DATA_TABLE: &str = "data";
const CHUNK_TABLE: &str = "data_chunks";
const CHUNK_CAPACITY: usize = 1024 * 1024;

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
        record_id: &str,
        cid: &str,
        value: T,
    ) -> Result<PutDataResults, DataStoreError>
    where
        T: Stream<Item = u8> + Unpin + Send,
    {
        let chunks = value.chunks(CHUNK_CAPACITY);
        pin_mut!(chunks);

        let id = Thing::from((DATA_TABLE, Id::String(record_id.to_string())));

        let len = self
            .with_database(tenant, |db| async move {
                db.delete::<Option<GetData>>(id.clone())
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)?;

                db.create::<Option<GetData>>(id.clone())
                    .content(CreateData {
                        cid: cid.to_string(),
                        tenant: tenant.to_string(),
                        record_id: record_id.to_string(),
                    })
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)?;

                let mut offset = 0;
                let mut len = 0;
                while let Some(chunk) = chunks.next().await {
                    let u = db
                        .create::<Vec<DataChunk>>(CHUNK_TABLE)
                        .content(DataChunk {
                            id: None,
                            data: chunk.clone(),
                        })
                        .await
                        .map_err(SurrealDBError::from)
                        .map_err(StoreError::from)?;

                    let mut relate = RelateStatement::default();
                    relate.from = Value::Param(Param::from(Ident::from("chunk")));
                    relate.kind = Value::Table(Table::from("data_for"));
                    relate.with = Value::Param(Param::from(Ident::from("data")));
                    relate.data = Some(Data::SetExpression(vec![(
                        Idiom::from("offset"),
                        Operator::Equal,
                        Value::Param(Param::from("offset")),
                    )]));
                    relate.only = true;

                    tracing::trace!(relate = relate.to_string(), chunk = chunk.len());

                    db.query(relate)
                        .bind(("chunk", u[0].id.clone().unwrap()))
                        .bind(("data", id.clone()))
                        .bind(("offset", offset))
                        .await
                        .map_err(SurrealDBError::from)
                        .map_err(StoreError::from)?;

                    offset += 1;
                    len += u[0].data.len();
                }

                db.update::<Option<DataChunkSize>>(id.clone())
                    .merge(DataChunkSize {
                        length: Some(len),
                        chunks: Some(offset),
                    })
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)?;

                Ok(len)
            })
            .await?;

        Ok(PutDataResults { size: len })
    }

    async fn get(
        &self,
        tenant: &str,
        record_id: &str,
        _: &str,
    ) -> Result<GetDataResults, DataStoreError> {
        let id = Thing::from((DATA_TABLE, Id::String(record_id.to_string())));

        let (res, s) = self
            .with_database(tenant, |db| async move {
                let d = db
                    .select::<Option<GetData>>(id.clone())
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)?
                    .ok_or(StoreError::NotFound)?;

                let chunks = d.chunks.ok_or(StoreError::NotFound)?;
                tracing::trace!(chunks, d = ?d, "fetching chunks for data");

                let i = from_iter(0..chunks)
                    .flat_map(move |offset| {
                        let db = db.clone();
                        let id = id.clone();

                        futures_util::stream::once(
                            async move {
                                tracing::trace!(?id, "fetching data");

                                db
                                    .query(
                                        "
                                SELECT
                                    <->(data_for WHERE offset = $offset)<->data_chunks.data AS chunks
                                FROM ONLY $from
                            ",
                                    )
                                    .bind(("from", id.clone()))
                                    .bind(("offset", offset))
                                    .await
                                    .expect("failed to bind")
                                    // .map_err(SurrealDBError::from)
                                    // .map_err(StoreError::from)?;
                                    .take::<Vec<Vec<u8>>>((0, "chunks"))
                                    .expect("failed to take 0")
                            }
                            .instrument(tracing::trace_span!("fetching data", offset)),
                        )
                    })
                    .flat_map(|r| stream::from_iter(r.into_iter().flatten()));


                Ok((d, i))
            })
            .await?;

        if res.record_id != record_id {
            return Err(DataStoreError::StoreError(StoreError::NotFound));
        }

        let size = res.length.ok_or(StoreError::NotFound)?;

        Ok(GetDataResults {
            size,
            data: Box::pin(s),
        })
    }

    async fn delete(&self, tenant: &str, record_id: &str, _: &str) -> Result<(), DataStoreError> {
        let id = Thing::from((DATA_TABLE, Id::String(record_id.to_string())));

        self.with_database(tenant, |db| async move {
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
            .map_err(DataStoreError::from)?;

        self.clear(&Table::from(CHUNK_TABLE))
            .await
            .map_err(DataStoreError::from)
    }
}
