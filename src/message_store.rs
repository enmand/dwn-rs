use std::sync::Arc;

use crate::filters::Filters;
use crate::{
    indexes::Indexes,
    message::{CreateEncodedMessage, Message},
};
use crate::{GetEncodedMessage, Query};
use async_trait::async_trait;

use cid::multihash::{Code, MultihashDigest};

use surrealdb::engine::any::Any;
use thiserror::Error;

const DBNAME: &str = "messages";
const TABLENAME: &str = "message";
const CBOR_TAGS_CID: u64 = 42;

#[async_trait]
pub trait MessageStore {
    async fn open(&self) -> Result<(), SurrealDBError>;

    async fn close(&mut self);

    async fn put(
        &self,
        tenant: &str,
        message: Message,
        indexes: Indexes,
    ) -> Result<cid::Cid, SurrealDBError>;

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, SurrealDBError>;

    async fn query(&self, tenant: &str, filter: Filters) -> Result<Vec<Message>, SurrealDBError>;

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError>;

    async fn clear(&self) -> Result<(), SurrealDBError>;
}

#[derive(Error, Debug)]
pub enum SurrealDBError {
    #[error("SurrealDBError: {0}")]
    ConnectionError(#[from] surrealdb::Error),

    #[error("no database initialized")]
    NoInitError,

    #[error("failed to encode message: {0}")]
    MessageEncodeError(#[from] serde_cbor::error::Error),

    #[error("failed to encode cid")]
    CidEncodeError(#[from] multihash::Error),

    #[error("unable to find record")]
    NotFound,
}

pub struct SurrealDB {
    db: Arc<surrealdb::Surreal<Any>>,
    tenant: String,
    _constr: String,
}

impl SurrealDB {
    pub fn new() -> Self {
        Self {
            db: Arc::new(surrealdb::Surreal::init()),
            tenant: String::default(),
            _constr: String::default(),
        }
    }

    pub async fn with_tenant(&mut self, tenant: &str) -> Result<(), SurrealDBError> {
        self.tenant = tenant.into();
        self.db.query("DEFINE NAMESPACE did").await?;

        self.db.use_ns(tenant).await.map_err(Into::into)
    }

    pub async fn connect(&mut self, connstr: &str) -> Result<(), SurrealDBError> {
        self.db.connect(connstr).await.map_err(Into::into)
    }
}

#[async_trait]
impl MessageStore for SurrealDB {
    async fn open(&self) -> Result<(), SurrealDBError> {
        self.db.health().await.map_err(Into::into)
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
    ) -> Result<cid::Cid, SurrealDBError> {
        let encoded_message = serde_cbor::to_vec(&message)?;
        let hash = Code::Sha2_256.digest(&encoded_message);
        let cid = cid::Cid::new_v1(CBOR_TAGS_CID, hash);

        let tdb = self.db.clone();
        tdb.use_ns(tenant).use_db(DBNAME).await?;

        tdb.create::<Option<GetEncodedMessage>>((TABLENAME, cid.to_string()))
            .content(CreateEncodedMessage {
                encoded_message,
                indexes,
            })
            .await?;

        Ok(cid)
    }

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, SurrealDBError> {
        let tdb = self.db.clone();
        tdb.use_ns(tenant).use_db(DBNAME).await?;

        // fetch and decode the message from the db
        let encoded_message = tdb
            .select::<Option<GetEncodedMessage>>((TABLENAME, cid))
            .await?
            .ok_or(SurrealDBError::NotFound)?;

        serde_cbor::from_slice::<Message>(&encoded_message.encoded_message).map_err(Into::into)
    }

    async fn query(&self, tenant: &str, filters: Filters) -> Result<Vec<Message>, SurrealDBError> {
        let tdb = self.db.clone();
        tdb.use_ns(tenant).use_db(DBNAME).await.unwrap();

        let (wheres, binds) = filters.query();
        let query = format!("SELECT * FROM {} WHERE {}", TABLENAME, wheres);

        let mut results = tdb.query(query).bind(binds).await?;

        let ms: Vec<Vec<u8>> = results.take((0, "encoded_message"))?;

        let r = ms
            .into_iter()
            .map(|m: Vec<u8>| serde_cbor::from_slice::<Message>(&m))
            .collect::<Result<Vec<Message>, _>>()?;

        Ok(r)
    }

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError> {
        let tdb = self.db.clone();
        tdb.use_ns(tenant).use_db(DBNAME).await?;

        tdb.delete::<Option<CreateEncodedMessage>>((TABLENAME, cid))
            .await?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), SurrealDBError> {
        let _: Vec<CreateEncodedMessage> = self.db.delete(DBNAME).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
        db.with_tenant("did").await.unwrap();
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
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        extra: HashMap::from([(
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
