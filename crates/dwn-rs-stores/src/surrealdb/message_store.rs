use std::str::FromStr;

use cid::Cid;
use serde::{de::DeserializeOwned, Serialize};

use super::core::SurrealDB;
use crate::{generate_cid, SurrealQuery};
use dwn_rs_core::{
    descriptors::MessageDescriptor,
    errors::{MessageStoreError, StoreError},
    fields::MessageFields,
    filters::{Filters, MessageSort, Pagination, Query, QueryReturn},
    interfaces::Message,
    stores::MessageStore,
    value::MapValue,
};

use super::{
    errors::SurrealDBError,
    models::{CreateEncodedMessage, GetEncodedMessage},
};

const MESSAGES_TABLE: &str = "messages";

impl MessageStore for SurrealDB {
    async fn open(&mut self) -> Result<(), MessageStoreError> {
        self.open().await.map_err(MessageStoreError::from)
    }

    async fn close(&mut self) {
        self.close().await
    }

    async fn put<D>(
        &self,
        tenant: &str,
        mut message: Message<D>,
        indexes: MapValue,
        tags: MapValue,
    ) -> Result<Cid, MessageStoreError>
    where
        D: MessageDescriptor + Serialize + Send + 'static,
    {
        let data = message.fields.encoded_data();

        let i = serde_ipld_dagcbor::to_vec(&message)?;
        let cid = generate_cid(&i)?;

        self.with_database(tenant, |db| async move {
            db.create::<Option<GetEncodedMessage>>((MESSAGES_TABLE, cid.to_string()))
                .content(CreateEncodedMessage {
                    cid: cid.to_string(),
                    encoded_message: i,
                    encoded_data: data,
                    tenant: tenant.to_string(),
                    indexes,
                    tags,
                })
                .await
                .map_err(SurrealDBError::from)
                .map_err(StoreError::from)
        })
        .await?;

        Ok(cid)
    }

    async fn get<D>(&self, tenant: &str, cid: &str) -> Result<Message<D>, MessageStoreError>
    where
        Message<D>: DeserializeOwned,
        D: MessageDescriptor + DeserializeOwned + Send + 'static,
    {
        // fetch and decode the message from the db
        let encoded_message: GetEncodedMessage = self
            .with_database(tenant, |db| async move {
                db.select((MESSAGES_TABLE, cid.to_string()))
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)
                    .expect("failed to fetch from database")
                    .ok_or(StoreError::NotFound)
            })
            .await?;

        if encoded_message.tenant != tenant {
            return Err(MessageStoreError::StoreError(StoreError::NotFound));
        }

        let mut from: Message<D> =
            serde_ipld_dagcbor::from_slice(&encoded_message.encoded_message)?;

        if let Some(data) = encoded_message.encoded_data {
            from.fields.encode_data(data);
        };

        Ok(from)
    }

    async fn query<D>(
        &self,
        tenant: &str,
        filters: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> Result<QueryReturn<Message<D>>, MessageStoreError>
    where
        Message<D>: DeserializeOwned,
        D: MessageDescriptor + DeserializeOwned + Send + 'static,
    {
        let mut qb = self
            .with_database(tenant, |db| async move {
                Ok(SurrealQuery::<GetEncodedMessage, MessageSort>::new(db))
            })
            .await?;

        qb.from(MESSAGES_TABLE)
            .filter(&filters)?
            .sort(sort)
            .page(pagination.clone());

        let (ms, cursor) = match qb.query().await {
            Ok(ms) => ms,
            Err(e) => {
                return Err(MessageStoreError::QueryError(e));
            }
        };

        let r = ms
            .into_iter()
            .filter(|m| m.tenant == tenant)
            .map(|m: GetEncodedMessage| {
                let cid = Cid::from_str(m.cid.as_str())?;
                let data_cid = generate_cid(&m.encoded_message)?;

                if cid != data_cid {
                    return Err(MessageStoreError::StoreError(StoreError::NotFound));
                }

                let mut msg: Message<D> = serde_ipld_dagcbor::from_slice(&m.encoded_message)?;

                if let Some(data) = m.encoded_data {
                    msg.fields.encode_data(data);
                }

                Ok(msg)
            })
            .collect::<Result<Vec<Message<D>>, MessageStoreError>>()?;

        Ok(QueryReturn { items: r, cursor })
    }

    async fn delete(&self, tenant: &str, cid: &str) -> Result<(), MessageStoreError> {
        let id = (MESSAGES_TABLE, cid.to_string());

        // check the tenancy on the messages
        let encoded_message: Option<GetEncodedMessage> = self
            .with_database(tenant, |db| async move {
                db.select((MESSAGES_TABLE, cid.to_string()))
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)
            })
            .await?;

        if let Some(msg) = encoded_message {
            if msg.tenant != tenant {
                return Err(MessageStoreError::StoreError(StoreError::NotFound));
            }

            self.with_database(tenant, |db| async move {
                db.delete::<Option<GetEncodedMessage>>(id.clone())
                    .await
                    .map_err(SurrealDBError::from)
                    .map_err(StoreError::from)
            })
            .await?;
        }

        Ok(())
    }

    async fn clear(&self) -> Result<(), MessageStoreError> {
        self.clear(&MESSAGES_TABLE.into())
            .await
            .map_err(MessageStoreError::from)?;

        Ok(())
    }
}
