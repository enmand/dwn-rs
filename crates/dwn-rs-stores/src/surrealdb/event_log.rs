use surrealdb::sql::{Id, Table, Thing};

use crate::{SurrealDB, SurrealDBError, SurrealQuery};
use dwn_rs_core::{
    errors::{EventLogError, StoreError},
    filters::{Cursor, Filters, MessageWatermark, Pagination, Query, QueryReturn},
    stores::EventLog,
    value::MapValue,
};

use super::models::{CreateEvent, GetEvent};

const EVENTS_TABLE: &str = "events";

impl EventLog for SurrealDB {
    async fn open(&mut self) -> Result<(), EventLogError> {
        self.open().await.map_err(EventLogError::from)
    }

    async fn close(&mut self) {
        self.close().await
    }

    async fn append(
        &self,
        tenant: &str,
        cid: String,
        indexes: MapValue,
        tags: MapValue,
    ) -> Result<(), EventLogError> {
        // get a mutable reference to self.gen
        let watermark = self.gen.lock().await.generate()?;
        let res = Thing::from((EVENTS_TABLE, cid.to_string().as_str()));

        self.with_database(tenant, |db| async move {
            db.create::<Option<CreateEvent>>(res)
                .content(CreateEvent {
                    watermark,
                    cid,
                    indexes,
                    tags,
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
            .with_database(tenant, |db| async move {
                Ok(SurrealQuery::<GetEvent, MessageWatermark>::new(db))
            })
            .await?;

        let page = Pagination {
            limit: None,
            cursor,
        };

        qb.from(EVENTS_TABLE)
            .filter(&filters)?
            .sort(Some(MessageWatermark::default()))
            .always_cursor()
            .page(Some(page));

        let (events, cursor) = qb.query().await?;

        Ok(QueryReturn {
            items: events.into_iter().map(|e| e.cid.to_string()).collect(),
            cursor,
        })
    }

    async fn delete<'a>(&self, tenant: &str, cids: &'a [&str]) -> Result<(), EventLogError> {
        Ok(self
            .with_database(tenant, |db| async move {
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
