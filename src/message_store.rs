use std::str::FromStr;
use std::sync::Arc;

use crate::filters::Filters;
use crate::{
    indexes::Indexes,
    message::{CreateEncodedMessage, Message},
};
use crate::{GetEncodedMessage, Query, SurrealDBError};
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
use surrealdb::sql::{Table, Thing};

const NAMESPACE: &str = "dwn-store";
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

    async fn query(&self, tenant: &str, filter: Filters) -> Result<Vec<Message>, SurrealDBError>;

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

        let id = Thing::from((tenant.to_string(), block.cid().to_string()));
        self.db
            .create::<Option<GetEncodedMessage>>(id)
            .content(CreateEncodedMessage {
                encoded_message: block.data().to_vec(),
                indexes,
            })
            .await?;

        Ok(block.cid().to_owned())
    }

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, SurrealDBError> {
        let id = Thing::from((tenant.to_string(), cid.clone()));

        // fetch and decode the message from the db
        let encoded_message = self
            .db
            .select::<Option<GetEncodedMessage>>(id)
            .await?
            .ok_or(SurrealDBError::NotFound)?;

        let block =
            Block::<DefaultParams>::new(Cid::try_from(cid)?, encoded_message.encoded_message)?;

        let from = from_ipld::<Message>(block.decode::<DagCborCodec, Ipld>()?);

        Ok(from?)
    }

    async fn query(&self, tenant: &str, filters: Filters) -> Result<Vec<Message>, SurrealDBError> {
        let tenant = Table::from(tenant);

        let (wheres, binds) = filters.query();
        let query = format!("SELECT * FROM {} WHERE {}", tenant, wheres);

        let mut results = self.db.query(query).bind(binds).await?;
        let ms: Vec<GetEncodedMessage> = results.take(0)?;

        let r = ms
            .into_iter()
            .map(|m: GetEncodedMessage| {
                Cid::from_str(&m.id.id.to_string())
                    .map_err(|e| SurrealDBError::CidDecodeError(e))
                    .and_then(|cid| {
                        Block::<DefaultParams>::new(cid, m.encoded_message)
                            .map_err(|e| SurrealDBError::MessageDecodeError(e))
                    })
                    .and_then(|ipld| {
                        from_ipld::<Message>(ipld.decode::<DagCborCodec, Ipld>()?)
                            .map_err(|e| SurrealDBError::SerdeDecodeError(e))
                    })
                    .unwrap_or_else(|_| Message::default())
            })
            .collect::<Vec<Message>>();

        Ok(r)
    }

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError> {
        let tenant = Thing::from((tenant.to_string(), cid.clone()));

        self.db
            .delete::<Option<CreateEncodedMessage>>(tenant)
            .await?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), SurrealDBError> {
        self.db.query(format!("REMOVE DATABASE {}", DBNAME)).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        filters::{EqualFilter, Filter, Filters, OneOfFilter, RangeValue, GT, LT},
        indexes::IndexValue,
        Indexes, MessageStore, SurrealDB,
    };

    #[tokio::test]
    async fn test_surrealdb() {
        let mut db = SurrealDB::new();
        let cwd = std::env::current_dir().unwrap().join("build/file.db");
        let _ = db
            .connect(format!("speedb://{file}", file = cwd.to_string_lossy()).as_str())
            .await;
        let _ = db.open().await;
        let map: Indexes = Indexes::from([
            ("key", IndexValue::from(8)),
            ("key2", IndexValue::from(true)),
            ("key3", IndexValue::from("value")),
            ("key5", IndexValue::from(1.3)),
            ("key6", IndexValue::from(2)),
            ("key7", IndexValue::from("7")),
        ]);
        let cid = db
            .put(
                "did",
                crate::Message {
                    descriptor: crate::Descriptor {
                        interface: "lorempsum doral ip sadsadaslj esflksd sdf".into(),
                        method: "sdfl;kjdsaflksdafj elf;jsdf s".into(),
                        timestamp: chrono::Utc::now(),
                        extra: BTreeMap::from([(
                            "key4".into(),
                            String::from(
                                "silhiofbrvnrews;;ljdlkhglsdkfvbcueiaj;dlksjdsllkhfdksfdajflhdsa",
                            )
                            .into(),
                        )]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                map,
            )
            .await
            .unwrap();

        let m = db.get("did", cid.to_string()).await.unwrap();
        println!("m: {:?}", m);
        let ms = db
            .query(
                "did",
                Filters::from([
                    ("key", GT::from(3).into()),
                    ("key2", Filter::from(true)),
                    ("key3", Filter::from("value")),
                    ("key5", LT::LTE(RangeValue::from(3)).into()),
                    (
                        "key6",
                        OneOfFilter::from(vec![EqualFilter::from(1), EqualFilter::from(2)]).into(),
                    ),
                    ("key7", Filter::from(GT::GTE(RangeValue::from("3")))),
                ]),
            )
            .await
            .unwrap();
        println!("ms: {:?}", ms);

        let _ = db.delete("did", cid.to_string()).await;
        let _ = db.close().await;
    }
}
