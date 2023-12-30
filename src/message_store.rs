use std::str::FromStr;
use std::sync::Arc;

use crate::filters::{Filters, Query};
use crate::{
    indexes::Indexes,
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
use surrealdb::sql::Thing;

const NAMESPACE: &str = "dwn";
const DBNAME: &str = "store";
const TABLENAME: &str = "messages";
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
                self.connect(&connstr).await?;
            }
        }

        Ok(())
    }

    async fn close(&mut self) {
        let _ = self.db.invalidate().await;
        self.db = Arc::new(surrealdb::Surreal::init());
    }

    async fn put(
        &self,
        tenant: &str,
        message: Message,
        indexes: Indexes,
    ) -> Result<Cid, SurrealDBError> {
        let ipld = to_ipld(&message)?;
        let block = Block::<DefaultParams>::encode(DagCborCodec, Code::Sha2_256, &ipld)?;

        // last_id is made from block.cid() and the tenant to ensure uniqueness
        let id = Thing::from((
            TABLENAME.to_string(),
            tenant.to_string() + &block.cid().to_string(),
        ));

        self.db
            .create::<Option<GetEncodedMessage>>(id)
            .content(CreateEncodedMessage {
                cid: block.cid().to_string(),
                encoded_message: block.data().to_vec(),
                tenant: tenant.to_string(),
                indexes,
            })
            .await?;

        Ok(block.cid().to_owned())
    }

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, SurrealDBError> {
        let id = Thing::from((TABLENAME.to_string(), tenant.to_string() + &cid.clone()));

        // fetch and decode the message from the db
        let encoded_message: GetEncodedMessage =
            self.db.select(id).await?.ok_or(SurrealDBError::NotFound)?;

        if encoded_message.tenant != tenant {
            return Err(SurrealDBError::NotFound);
        }

        let block =
            Block::<DefaultParams>::new(Cid::try_from(cid)?, encoded_message.encoded_message)?;

        let from = from_ipld::<Message>(block.decode::<DagCborCodec, Ipld>()?);

        Ok(from?)
    }

    async fn query(
        &self,
        tenant: &str,
        filters: Filters,
        sort: Option<MessageSort>,
        pagination: Option<Pagination>,
    ) -> Result<QueryReturn, SurrealDBError> {
        let mut qb = SurrealQuery::<GetEncodedMessage>::new(self.db.to_owned());

        qb.from(TABLENAME.to_string())
            .filter(&filters)?
            .sort(sort.clone())
            .page(pagination.clone());

        let ms = qb.query().await?;

        let r = ms
            .into_iter()
            .filter(|m| m.tenant == tenant)
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
                    .and_then(|pm| Ok((pm, m.tenant.clone())))
                    .unwrap_or_else(|e| {
                        web_sys::console::log_1(&format!("Error: {:?}", e).into());
                        (Message::default(), m.tenant)
                    })
            })
            // .filter(|(m, msg_tenant)| {
            //     tenant == msg_tenant
            //         || ((m.descriptor.published.is_some() && m.descriptor.published.unwrap())
            //             || m.descriptor.published.is_none())
            // })
            .map(|(m, _)| m)
            .collect::<Vec<Message>>();

        web_sys::console::log_1(&format!("Query results: {:?}", r).into());

        let last_cid = match r.last() {
            Some(m) => m.record_id.clone(),
            None => None,
        };

        Ok(QueryReturn {
            messages: r,
            pagination_message_cid: last_cid,
        })
    }

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError> {
        let id = Thing::from((TABLENAME.to_string(), tenant.to_string() + &cid.clone()));

        // check the tenancy on the messages
        let encoded_message: GetEncodedMessage = self
            .db
            .select(id.clone())
            .await?
            .ok_or(SurrealDBError::NotFound)?;

        if encoded_message.tenant != tenant {
            return Err(SurrealDBError::NotFound);
        }

        self.db.delete::<Option<CreateEncodedMessage>>(id).await?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), SurrealDBError> {
        self.db.query(format!("REMOVE DATABASE {}", DBNAME)).await?;

        Ok(())
    }
}
