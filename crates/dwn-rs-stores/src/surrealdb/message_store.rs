use std::str::FromStr;

use async_trait::async_trait;
use cid::Cid;
use multihash_codetable::{Code, MultihashDigest};
use surrealdb::sql::{Id, Table, Thing};

use super::core::SurrealDB;
use crate::{
    Filters, MessageSort, MessageStore, MessageStoreError, Pagination, Query, QueryReturn,
};
use crate::{StoreError, SurrealQuery};
use dwn_rs_core::{MapValue, Message, Value};

use super::{
    errors::SurrealDBError,
    models::{CreateEncodedMessage, GetEncodedMessage},
};

#[async_trait]
impl MessageStore for SurrealDB {
    async fn open(&mut self) -> Result<(), MessageStoreError> {
        self.open().await.map_err(MessageStoreError::from)
    }

    async fn close(&mut self) {
        self.close().await
    }

    async fn put(
        &self,
        tenant: &str,
        mut message: Message,
        indexes: MapValue,
    ) -> Result<Cid, MessageStoreError> {
        let mut data: Option<Value> = None;
        if message.extra.contains_key("encodedData") {
            data = message.extra.remove("encodedData");
        }

        let i = serde_ipld_dagcbor::to_vec(&message)?;
        let mh = Code::Sha2_256.digest(i.as_slice());
        let cid = Cid::new_v1(multicodec::Codec::Dag_Cbor.code().into(), mh);

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        self.db
            .create::<Option<GetEncodedMessage>>(id.clone())
            .content(CreateEncodedMessage {
                cid: cid.to_string(),
                encoded_message: i,
                encoded_data: data,
                tenant: tenant.to_string(),
                indexes,
            })
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(MessageStoreError::from)?;

        Ok(cid)
    }

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, MessageStoreError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        // fetch and decode the message from the db
        let encoded_message: GetEncodedMessage = self
            .db
            .select(id.clone())
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(MessageStoreError::from)?
            .ok_or(MessageStoreError::StoreError(StoreError::NotFound))?;

        if encoded_message.tenant != tenant {
            return Err(MessageStoreError::StoreError(StoreError::NotFound));
        }

        let mut from: Message = serde_ipld_dagcbor::from_slice(&encoded_message.encoded_message)?;

        if let Some(data) = encoded_message.encoded_data {
            from.extra.insert("encodedData".to_string(), data);
        }

        Ok(from)
    }

    async fn query(
        &self,
        tenant: &str,
        filters: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> Result<QueryReturn<Message>, MessageStoreError> {
        let mut qb = SurrealQuery::<GetEncodedMessage, MessageSort>::new(self.db.to_owned());

        qb.from(tenant.to_string())
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
            .map(|m: GetEncodedMessage| {
                let cid = Cid::from_str(m.cid.as_str())?;
                let mh = Code::Sha2_256.digest(&m.encoded_message);
                let data_cid = Cid::new_v1(multicodec::Codec::Dag_Cbor.code().into(), mh);

                if cid != data_cid {
                    return Err(MessageStoreError::StoreError(StoreError::NotFound));
                }

                let mut msg: Message = serde_ipld_dagcbor::from_slice(&m.encoded_message)?;

                if let Some(data) = m.encoded_data {
                    msg.extra.insert("encodedData".to_string(), data);
                }

                Ok(msg)
            })
            .collect::<Result<Vec<Message>, MessageStoreError>>();

        let qr = QueryReturn { items: r?, cursor };

        Ok(qr)
    }

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), MessageStoreError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        // check the tenancy on the messages
        let encoded_message: Option<GetEncodedMessage> = self
            .db
            .select(id.clone())
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)
            .map_err(MessageStoreError::from)?;

        if let Some(msg) = encoded_message {
            if msg.tenant != tenant {
                return Err(MessageStoreError::StoreError(StoreError::NotFound));
            }

            self.db
                .delete::<Option<CreateEncodedMessage>>(id)
                .await
                .map_err(SurrealDBError::from)
                .map_err(StoreError::from)
                .map_err(MessageStoreError::from)?;
        }

        Ok(())
    }

    async fn clear(&self) -> Result<(), MessageStoreError> {
        self.clear().await.map_err(MessageStoreError::from)?;

        Ok(())
    }
}
