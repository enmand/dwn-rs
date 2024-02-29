use async_trait::async_trait;
use surrealdb::sql::{Id, Range, Table, Thing};
use ulid::Ulid;

use crate::{
    Cursor, EventLog, EventLogError, Filters, Indexes, MessageCidSort, Pagination, Query,
    QueryReturn, StoreError, SurrealDB, SurrealDBError, SurrealQuery,
};

use super::models::{CreateEvent, GetEvent};

trait IntoRange: Into<Range> + Sized {}

#[async_trait]
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
        indexes: Indexes,
    ) -> Result<(), EventLogError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        let watermark = Ulid::new().to_string();
        self.db
            .create::<Option<CreateEvent>>(id.clone())
            .content(CreateEvent {
                id,
                watermark,
                cid,
                indexes,
            })
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(EventLogError::from)?;

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
        let mut qb = SurrealQuery::<GetEvent, MessageCidSort>::new(self.db.to_owned());

        let page = Pagination {
            limit: None,
            cursor,
        };

        qb.from(tenant.to_string())
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
        for c in cids {
            let id = Thing::from((
                Table::from(tenant.to_string()).to_string(),
                Id::String(c.to_string()),
            ));

            self.db
                .delete::<Option<CreateEvent>>(id)
                .await
                .map_err(SurrealDBError::from)
                .map_err(StoreError::from)
                .map_err(EventLogError::from)?;
        }

        Ok(())
    }

    async fn clear(&self) -> Result<(), EventLogError> {
        self.clear().await.map_err(EventLogError::from)?;

        Ok(())
    }
}
