use async_std::stream::from_iter;
use bytes::Bytes;
use futures_util::{pin_mut, Stream, StreamExt};
use surrealdb::{
    sql::{
        statements::{RelateStatement, SelectStatement},
        Cond, Data, Dir, Expression, Field, Fields, Graph, Ident, Idiom, Operator, Param, Part,
        Query, Statement, Table, Value, Values,
    },
    RecordId,
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
const CHUNK_CAPACITY: usize = 512 * 1024;

impl DataStore for SurrealDB {
    async fn open(&mut self) -> Result<(), DataStoreError> {
        let _ = chunks_graph_query(); // compile the chunks graph query on open
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
        T: Stream<Item = Bytes> + Unpin + Send,
    {
        let id: RecordId = (DATA_TABLE, record_id.to_string()).into();

        pin_mut!(value);
        let mut chunks = value
            .flat_map(|b| from_iter(b.into_iter()))
            .chunks(CHUNK_CAPACITY);

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
                        .create::<Option<DataChunk>>(CHUNK_TABLE)
                        .content(DataChunk {
                            id: None,
                            data: chunk.clone(),
                        })
                        .await
                        .map_err(SurrealDBError::from)
                        .map_err(StoreError::from)?
                        .ok_or(StoreError::NotFound)?;

                    let mut relate = RelateStatement::default();
                    relate.from = Value::Param(Param::from(Ident::from("chunk")));
                    relate.kind = Value::Table(Table::from("chunk_of"));
                    relate.with = Value::Param(Param::from(Ident::from("data")));
                    relate.data = Some(Data::SetExpression(vec![(
                        Idiom::from("offset"),
                        Operator::Equal,
                        Value::Param(Param::from("offset")),
                    )]));
                    relate.uniq = true;
                    relate.only = true;

                    tracing::trace!(relate = relate.to_string(), chunk = chunk.len());

                    db.query(relate)
                        .bind(("chunk", u.id.clone().unwrap()))
                        .bind(("data", id.clone()))
                        .bind(("offset", offset))
                        .await
                        .map_err(SurrealDBError::from)
                        .map_err(StoreError::from)?;

                    offset += 1;
                    len += chunk.len();
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
        let id: RecordId = (DATA_TABLE, record_id.to_string()).into();
        let query = chunks_graph_query();

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
                        let query = query.clone();
                        let db = db.clone();
                        let id = id.clone();

                        futures_util::stream::once(
                            async move {
                                tracing::trace!(?id, query = query.to_string(), "chunk");

                                let r = db
                                    .query(query.clone())
                                    .bind(("from", id.clone()))
                                    .bind(("offset", offset))
                                    .await
                                    .map_err(SurrealDBError::from)
                                    .map_err(StoreError::from)?
                                    .take::<Vec<Vec<u8>>>(0)
                                    .map_err(SurrealDBError::from)
                                    .map_err(StoreError::from)?;

                                Ok(r)
                            }
                            .instrument(tracing::trace_span!("fetching data", offset)),
                        )
                    })
                    .flat_map(|r| {
                        from_iter(
                            r.unwrap_or_else(|e: StoreError| {
                                tracing::error!(err=?e, "unable to fetch data");
                                Vec::new()
                            })
                            .into_iter()
                            .flatten(),
                        )
                    });

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
        let id = (DATA_TABLE, record_id.to_string());

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

/// Creates a memoizable query for fetching data chunks for a given record.
/// The query includes parameters for $offset (chunk offset) and $from
/// (record).
///
/// The query is:
/// ```sql
///     SELECT
///         ->(chunk_of WHERE offset = $offset)->data_chunks.data AS chunks
///     FROM ONLY $from
/// ```
//
// The query is memoized to avoid recompilation on each call. The
// `surrealdb::sql::parse` is not used, as it can result in a runtime
// error if the query is not valid. The query is guaranteed to be valid
// at compile time.
#[memoize::memoize]
fn chunks_graph_query() -> Query {
    let mut offset_graph = Graph::default();
    offset_graph.dir = Dir::In; // <-
    offset_graph.expr = Fields::all();
    offset_graph.what = Table::from("chunk_of").into();
    let mut offset_cond = Cond::default();
    offset_cond.0 = Expression::Binary {
        l: Idiom::from("offset").into(),
        o: Operator::Equal,
        r: Param::from("offset").into(),
    }
    .into();
    offset_graph.cond = Some(offset_cond);

    // <->data_chunks.data
    let mut chunk_graph = Graph::default();
    chunk_graph.dir = Dir::In;
    chunk_graph.expr = Fields::all();
    chunk_graph.what = Table::from(CHUNK_TABLE).into();

    // FROM ONLY $from
    let mut graph_edge = Values::default();
    graph_edge.0.push(Value::Param(Param::from("from")));

    // SELECT for graph elements
    let mut query = SelectStatement::default();
    query.expr.1 = true; // VALUE
    query.expr.0.push(Field::Single {
        expr: Idiom::from(vec![
            Part::Graph(offset_graph),
            Part::Graph(chunk_graph),
            Part::Field(Ident::from("data")),
        ])
        .into(),
        alias: None,
    });
    query.only = true; // single chunk per poll
    query.what = graph_edge;

    Statement::Select(query).into()
}

#[cfg(test)]
mod test {
    use async_std::stream;
    use futures_util::StreamExt;
    use std::iter::repeat_with;

    use super::*;
    use dwn_rs_core::stores::DataStore;

    #[tokio::test]
    async fn test_open_close() {
        let mut db = SurrealDB::new();
        db.connect("mem://").await.unwrap();
        db.open().await.unwrap();
        db.close().await;
    }

    #[tokio::test]
    async fn test_put_get() {
        let mut db = SurrealDB::new();
        db.connect("mem://").await.unwrap();
        db.open().await.unwrap();

        let tenant = "test";
        let record_id = "test_put_get";
        let cid = "test_put_get_cid";

        let data = Bytes::from_iter(
            repeat_with(rand::random::<u8>)
                .take(1024 * 1024)
                .collect::<Vec<u8>>(),
        );

        let put = db
            .put(tenant, record_id, cid, stream::once(data.clone()))
            .await
            .unwrap();
        assert_eq!(put.size, data.len());

        let get = db.get(tenant, record_id, cid).await.unwrap();
        assert_eq!(get.size, data.len());

        let get_data = get.data.collect::<Vec<u8>>().await;
        assert_eq!(data, get_data);

        db.close().await;
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let mut db = SurrealDB::new();
        db.connect("mem://").await.unwrap();
        db.open().await.unwrap();

        let tenant = "test";
        let record_id = "test_get_not_found";
        let cid = "test_get_not_found_cid";

        let get = db.get(tenant, record_id, cid).await;
        assert!(get.is_err());

        db.close().await;
    }

    #[tokio::test]
    async fn test_delete() {
        let mut db = SurrealDB::new();
        db.connect("mem://").await.unwrap();
        db.open().await.unwrap();

        let tenant = "test";
        let record_id = "test_delete";
        let cid = "test_delete_cid";

        let data = Bytes::from_iter(
            repeat_with(rand::random::<u8>)
                .take(1024 * 1024)
                .collect::<Vec<u8>>(),
        );

        let put = db
            .put(tenant, record_id, cid, stream::once(data.clone()))
            .await
            .unwrap();
        assert_eq!(put.size, data.len());

        let get = db.get(tenant, record_id, cid).await.unwrap();
        assert_eq!(get.size, data.len());

        let get_data = get.data.collect::<Vec<u8>>().await;
        assert_eq!(data, get_data);

        db.delete(tenant, record_id, cid).await.unwrap();

        let get = db.get(tenant, record_id, cid).await;
        assert!(get.is_err());

        db.close().await;
    }
}
