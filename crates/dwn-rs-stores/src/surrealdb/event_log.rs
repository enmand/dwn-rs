use async_trait::async_trait;
use dwn_rs_core::MapValue;
use surrealdb::sql::{Id, Table, Thing};

use crate::{
    Cursor, EventLog, EventLogError, Filters, MessageCidSort, Pagination, Query, QueryReturn,
    StoreError, SurrealDB, SurrealDBError, SurrealQuery,
};

use super::models::{CreateEvent, GetEvent};

const EVENTS_TABLE: &str = "events";

#[async_trait]
impl EventLog for SurrealDB {
    async fn open(&mut self) -> Result<(), EventLogError> {
        self.open().await.map_err(EventLogError::from)
    }

    async fn close(&mut self) {
        self.close().await
    }

    async fn append(
        &mut self,
        tenant: &str,
        cid: String,
        indexes: MapValue,
    ) -> Result<(), EventLogError> {
        let id = Thing::from((EVENTS_TABLE, Id::String(cid.to_string())));
        let watermark = self.ulid_generator.generate()?.to_string();

        self.as_tenant(tenant, |db| async move {
            db.create::<Option<CreateEvent>>(id.clone())
                .content(CreateEvent {
                    id,
                    watermark,
                    cid,
                    indexes,
                })
                .await
                .map_err(SurrealDBError::from)
                .map_err(StoreError::from)
        })
        .await?;

        Ok(())
    }

    async fn get_events(
        &self,
        tenant: &str,
        cursor: Option<Cursor>,
    ) -> Result<QueryReturn<String>, EventLogError> {
        self.query_events(tenant, Filters::default(), cursor).await
    }

    async fn query_events(
        &self,
        tenant: &str,
        filters: Filters,
        cursor: Option<Cursor>,
    ) -> Result<QueryReturn<String>, EventLogError> {
        let mut qb = self
            .as_tenant(tenant, |db| async move {
                Ok(SurrealQuery::<GetEvent, MessageCidSort>::new(db))
            })
            .await?;

        let page = Pagination {
            limit: None,
            cursor,
        };

        qb.from(EVENTS_TABLE)
            .filter(&filters)?
            .sort(Some(MessageCidSort::default()))
            .always_cursor()
            .page(Some(page));

        let (mut events, cursor) = qb.query().await?;
        events.sort_by_key(|e| e.watermark.to_owned());

        Ok(QueryReturn {
            items: events.into_iter().map(|e| e.id.id.to_string()).collect(),
            cursor,
        })
    }

    async fn delete<'a>(&self, tenant: &str, cids: &'a [&str]) -> Result<(), EventLogError> {
        Ok(self
            .as_tenant(tenant, |db| async move {
                for c in cids {
                    let id = Thing::from((EVENTS_TABLE, Id::String(c.to_string())));

                    db.delete::<Option<CreateEvent>>(id)
                        .await
                        .map_err(SurrealDBError::from)
                        .map_err(StoreError::from)?;
                }

                Ok(())
            })
            .await?)
    }

    async fn clear(&self) -> Result<(), EventLogError> {
        self.clear(&Table::from(EVENTS_TABLE))
            .await
            .map_err(EventLogError::from)?;

        Ok(())
    }
}
