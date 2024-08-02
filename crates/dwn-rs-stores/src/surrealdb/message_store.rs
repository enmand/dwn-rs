use std::str::FromStr;

use async_trait::async_trait;
use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::sql::{Id, Thing};

use super::core::SurrealDB;
use crate::SurrealQuery;
use dwn_rs_core::{
    errors::{MessageStoreError, StoreError},
    filters::{Filters, MessageSort, Pagination, Query, QueryReturn},
    interfaces::{Descriptor, Fields, Message},
    stores::MessageStore,
    value::{MapValue, Value},
};

use super::{
    errors::SurrealDBError,
    models::{CreateEncodedMessage, GetEncodedMessage},
};

const MESSAGES_TABLE: &str = "messages";

#[async_trait]
impl MessageStore for SurrealDB {
    async fn open(&mut self) -> Result<(), MessageStoreError> {
        self.open().await.map_err(MessageStoreError::from)
    }

    async fn close(&mut self) {
        self.close().await
    }

    async fn put<D: Descriptor + Serialize + Send, F: Fields + Serialize + Send>(
        &self,
        tenant: &str,
        mut message: Message<D, F>,
        indexes: MapValue,
        tags: MapValue,
    ) -> Result<Cid, MessageStoreError> {
        let mut data: Option<Value> = None;
        if message.fields.contains_key("encodedData") {
            data = message.fields.remove("encodedData");
        }

        let i = serde_ipld_dagcbor::to_vec(&message)?;
        let mh = Code::Sha2_256.digest(i.as_slice());
        let cid = Cid::new_v1(multicodec::Codec::DagCbor.code(), mh);

        self.with_database(tenant, |db| async move {
            db.create::<Option<GetEncodedMessage>>((MESSAGES_TABLE, Id::String(cid.to_string())))
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

    async fn get<D: Descriptor + DeserializeOwned, F: Fields + DeserializeOwned>(
        &self,
        tenant: &str,
        cid: String,
    ) -> Result<Message<D, F>, MessageStoreError> {
        // fetch and decode the message from the db
        let encoded_message: GetEncodedMessage = self
            .with_database(tenant, |db| async move {
                db.select(Thing::from((MESSAGES_TABLE, Id::String(cid.to_string()))))
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

        let mut from: Message<D, F> =
            serde_ipld_dagcbor::from_slice(&encoded_message.encoded_message)?;

        if let Some(data) = encoded_message.encoded_data {
            from.fields.insert("encodedData".to_string(), data);
        }

        Ok(from)
    }

    async fn query<D: Descriptor + DeserializeOwned, F: Fields + DeserializeOwned>(
        &self,
        tenant: &str,
        filters: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> Result<QueryReturn<Message<D, F>>, MessageStoreError> {
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
                let mh = Code::Sha2_256.digest(&m.encoded_message);
                let data_cid = Cid::new_v1(multicodec::Codec::DagCbor.code(), mh);

                if cid != data_cid {
                    return Err(MessageStoreError::StoreError(StoreError::NotFound));
                }

                let mut msg: Message<D, F> = serde_ipld_dagcbor::from_slice(&m.encoded_message)?;

                if let Some(data) = m.encoded_data {
                    msg.fields.insert("encodedData".to_string(), data);
                }

                Ok(msg)
            })
            .collect::<Result<Vec<Message<_, _>>, MessageStoreError>>()?;

        Ok(QueryReturn { items: r, cursor })
    }

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), MessageStoreError> {
        let id = Thing::from((MESSAGES_TABLE, Id::String(cid.to_string())));

        // check the tenancy on the messages
        let encoded_message: Option<GetEncodedMessage> = self
            .with_database(tenant, |db| async move {
                db.select(Thing::from((MESSAGES_TABLE, Id::String(cid.to_string()))))
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
