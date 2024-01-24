use std::str::FromStr;
use std::sync::Arc;

use crate::filters::{Filters, Query};
use crate::{
    filters::indexes::Indexes,
    message::{CreateEncodedMessage, Message},
};
use crate::{
    GetEncodedMessage, MessageSort, Pagination, QueryReturn, SurrealDBError, SurrealQuery,
};
use async_trait::async_trait;
use libipld::{block::Block, store::DefaultParams};
use libipld_cbor::DagCborCodec;
use libipld_core::ipld::Ipld;
use libipld_core::{
    cid::Cid,
    multihash::Code,
    serde::{from_ipld, to_ipld},
};
use surrealdb::engine::any::Any;
use surrealdb::sql::{Id, Table, Thing};

const NAMESPACE: &str = "dwn";
const DBNAME: &str = "messages";
const CBOR_TAGS_CID: u64 = 42;

#[async_trait]
pub trait MessageStore {
    async fn open(&mut self) -> Result<(), SurrealDBError>;

    async fn close(&mut self);

    async fn put(
        &self,
        tenant: &str,
        message: Message,
        indexes: Indexes,
    ) -> Result<Cid, SurrealDBError>;

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, SurrealDBError>;

    async fn query(
        &self,
        tenant: &str,
        filter: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> Result<QueryReturn, SurrealDBError>;

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError>;

    async fn clear(&self) -> Result<(), SurrealDBError>;
}

pub struct SurrealDB {
    db: Arc<surrealdb::Surreal<Any>>,
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
    async fn open(&mut self) -> Result<(), SurrealDBError> {
        let health = self.db.health().await;
        if health.is_err() {
            if self._constr.is_empty() {
                return Err(SurrealDBError::NoInitError);
            } else {
                let connstr = self._constr.clone();
                self.db.connect(&connstr).await?;
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
    ) -> Result<Cid, SurrealDBError> {
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
            .await?;

        Ok(cid)
    }

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, SurrealDBError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        // fetch and decode the message from the db
        let encoded_message: GetEncodedMessage = self
            .db
            .select(id.clone())
            .await?
            .ok_or(SurrealDBError::NotFound)?;

        if encoded_message.tenant != tenant {
            return Err(SurrealDBError::NotFound);
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
    ) -> Result<QueryReturn, SurrealDBError> {
        let mut qb = SurrealQuery::<GetEncodedMessage>::new(self.db.to_owned());

        qb.from(tenant.to_string())
            .filter(&filters)?
            .sort(sort.clone())
            .page(pagination.clone());

        let (ms, cursor) = match qb.query().await {
            Ok(ms) => ms,
            Err(e) => {
                web_sys::console::log_1(&format!("query error: {:?}", e).into());
                return Err(SurrealDBError::QueryError(e));
            }
        };

        let r = ms
            .into_iter()
            .map(|m: GetEncodedMessage| {
                Cid::from_str(&m.cid.to_string())
                    .map_err(|e| SurrealDBError::CidDecodeError(e))
                    .and_then(|cid| {
                        Block::<DefaultParams>::new(cid, m.encoded_message)
                            .map_err(|e| SurrealDBError::MessageDecodeError(e))
                    })
                    .and_then(|ipld| {
                        from_ipld::<Message>(ipld.decode::<DagCborCodec, Ipld>()?)
                            .map_err(|e| SurrealDBError::SerdeDecodeError(e))
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

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError> {
        let id = Thing::from((
            Table::from(tenant.to_string()).to_string(),
            Id::String(cid.to_string()),
        ));

        // check the tenancy on the messages
        let encoded_message: Option<GetEncodedMessage> = self.db.select(id.clone()).await?;

        if let Some(msg) = encoded_message {
            if msg.tenant != tenant {
                return Err(SurrealDBError::NotFound);
            }

            self.db.delete::<Option<CreateEncodedMessage>>(id).await?;
        }

        Ok(())
    }

    async fn clear(&self) -> Result<(), SurrealDBError> {
        self.db.query(format!("REMOVE DATABASE {}", DBNAME)).await?;

        Ok(())
    }
}
