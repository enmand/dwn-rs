use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use libipld::{cid::Cid, Block, DefaultParams};
use libipld_cbor::DagCborCodec;
use libipld_core::{
    ipld::Ipld,
    multihash::Code,
    serde::{from_ipld, to_ipld},
};
use surrealdb::{
    engine::any::Any,
    sql::{Id, Table, Thing},
    Surreal,
};

use crate::SurrealQuery;
use dwn_rs_core::Message;
use dwn_rs_stores::{
    Filters, Indexes, MessageSort, MessageStore, MessageStoreError, Pagination, Query, QueryReturn,
};

use super::{
    errors::SurrealDBError,
    models::{CreateEncodedMessage, GetEncodedMessage},
};

const NAMESPACE: &str = "dwn";
const DBNAME: &str = "messages";

pub struct SurrealDB {
    db: Arc<Surreal<Any>>,
    _constr: String,
}

impl SurrealDB {
    pub fn new() -> Self {
        Self {
            db: Arc::new(surrealdb::Surreal::init()),
            _constr: String::default(),
        }
    }

    pub fn with_db(&mut self, db: surrealdb::Surreal<Any>) -> &mut Self {
        self.db = Arc::new(db);
        self
    }

    pub async fn connect(&mut self, connstr: &str) -> Result<(), SurrealDBError> {
        self._constr = connstr.into();
        self.db.connect(connstr).await?;
        self.db
            .health()
            .await
            .map_err(Into::<SurrealDBError>::into)?;
        self.db
            .use_ns(NAMESPACE)
            .use_db(DBNAME)
            .await
            .map_err(Into::into)
    }
}

#[async_trait]
impl MessageStore for SurrealDB {
    async fn open(&mut self) -> Result<(), MessageStoreError> {
        let health = self.db.health().await;
        if health.is_err() {
            if self._constr.is_empty() {
                return Err(MessageStoreError::NoInitError);
            } else {
                let connstr = self._constr.clone();
                self.db
                    .connect(&connstr)
                    .await
                    .map_err(SurrealDBError::from)?;
            }
        }

        Ok(())
    }

    async fn close(&mut self) {
        let _ = self.db.invalidate().await;
    }

    async fn put(
        &self,
        tenant: &str,
        mut message: Message,
        indexes: Indexes,
    ) -> Result<Cid, MessageStoreError> {
        let mut data: Option<Ipld> = None;
        if message.extra.contains_key("encodedData") {
            data = message.extra.remove("encodedData");
        }
        let block =
            Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &to_ipld(&message)?)?;
        let cid = block.cid().to_owned();

        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        self.db
            .create::<Option<GetEncodedMessage>>(id.clone())
            .content(CreateEncodedMessage {
                cid: cid.to_string(),
                encoded_message: block.data().to_vec(),
                encoded_data: data,
                tenant: tenant.to_string(),
                indexes,
            })
            .await
            .map_err(SurrealDBError::from)?;

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
            .map_err(SurrealDBError::from)?
            .ok_or(MessageStoreError::NotFound)?;

        if encoded_message.tenant != tenant {
            return Err(MessageStoreError::NotFound);
        }

        let block =
            Block::<DefaultParams>::new(Cid::try_from(cid)?, encoded_message.encoded_message)?;

        let mut from = from_ipld::<Message>(block.decode::<DagCborCodec, Ipld>()?)?;

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
    ) -> Result<QueryReturn, MessageStoreError> {
        let mut qb = SurrealQuery::<GetEncodedMessage>::new(self.db.to_owned());

        qb.from(tenant.to_string())
            .filter(&filters)?
            .sort(sort.clone())
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
                Cid::from_str(&m.cid.to_string())
                    .map_err(|e| MessageStoreError::CidDecodeError(e))
                    .and_then(|cid| {
                        Block::<DefaultParams>::new(cid, m.encoded_message)
                            .map_err(|e| MessageStoreError::MessageDecodeError(e))
                    })
                    .and_then(|ipld| {
                        from_ipld::<Message>(ipld.decode::<DagCborCodec, Ipld>()?)
                            .map_err(|e| MessageStoreError::SerdeDecodeError(e))
                    })
                    .and_then(|mut msg: Message| {
                        if let Some(data) = m.encoded_data {
                            msg.extra.insert("encodedData".to_string(), data);
                        }
                        Ok(msg)
                    })
                    .unwrap_or_else(|_| Message::default())
            })
            .collect::<Vec<Message>>();

        let qr = QueryReturn {
            messages: r,
            cursor,
        };

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
            .map_err(SurrealDBError::from)?;

        if let Some(msg) = encoded_message {
            if msg.tenant != tenant {
                return Err(MessageStoreError::NotFound);
            }

            self.db
                .delete::<Option<CreateEncodedMessage>>(id)
                .await
                .map_err(SurrealDBError::from)?;
        }

        Ok(())
    }

    async fn clear(&self) -> Result<(), MessageStoreError> {
        self.db
            .query(format!("REMOVE DATABASE {}", DBNAME))
            .await
            .map_err(SurrealDBError::from)?;

        Ok(())
    }
}
