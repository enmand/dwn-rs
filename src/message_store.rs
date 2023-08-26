use std::{collections::TryReserveError, convert::Infallible, sync::Arc};

use crate::message::{Descriptor, EncodedMessage, Filter, Index, IndexValue, Message};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use cid::multihash::{Code, MultihashDigest};
use jose_jws::General as JWS;
use serde::{Deserialize, Serialize, Serializer};
use surrealdb::engine::any::Any;
use thiserror::Error;

const DBNAME: &str = "messages";
const CBOR_TAGS_CID: u64 = 42;

#[async_trait]
pub trait MessageStore {
    async fn open(&self) -> Result<(), SurrealDBError>;

    async fn close(&mut self);

    async fn put(
        &self,
        tenant: &str,
        message: Message,
        indexes: Vec<Index>,
    ) -> Result<cid::Cid, SurrealDBError>;

    async fn get(&self, tenant: &str, cid: String) -> Result<Message, SurrealDBError>;

    async fn query(&self, tenant: &str, filter: Vec<Filter>) -> Vec<Message>;

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError>;

    async fn clear(&self);
}

#[derive(Error, Debug)]
pub enum SurrealDBError {
    #[error("SurrealDBError: {0}")]
    ConnectionError(#[from] surrealdb::Error),

    #[error("no database initialized")]
    NoInitError,

    #[error("failed to encode message")]
    MessageEncodeError(#[from] serde_ipld_dagcbor::error::EncodeError<TryReserveError>),

    #[error("failed to decode message")]
    MessageDecodeError(#[from] serde_ipld_dagcbor::error::DecodeError<Infallible>),

    #[error("failed to encode cid")]
    CidEncodeError(#[from] multihash::Error),

    #[error("unable to find record")]
    NotFound,
}

pub struct SurrealDB {
    db: Arc<surrealdb::Surreal<Any>>,
    tentant: String,
    _constr: String,
}

impl SurrealDB {
    pub fn new() -> Self {
        Self {
            db: Arc::new(surrealdb::Surreal::init()),
            tentant: String::default(),
            _constr: String::default(),
        }
    }

    pub async fn with_tenant(&mut self, tenant: &str) -> Result<(), SurrealDBError> {
        self.tentant = tenant.into();
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
        indexes: Vec<Index>,
    ) -> Result<cid::Cid, SurrealDBError> {
        let encoded_message = serde_ipld_dagcbor::to_vec(&message)?;
        let hash = Code::Sha2_256.digest(&encoded_message);
        let cid = cid::Cid::new_v1(CBOR_TAGS_CID, hash);

        let tdb = self.db.clone();
        tdb.use_ns(tenant).use_db(DBNAME).await?;

        tdb.create::<Option<EncodedMessage>>(("message", cid.to_string()))
            .content(EncodedMessage {
                encoded_message,
                cid,
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
            .select::<Option<EncodedMessage>>(("message", cid))
            .await?
            .ok_or(SurrealDBError::NotFound)?;

        serde_ipld_dagcbor::from_slice::<Message>(&encoded_message.encoded_message)
            .map_err(Into::into)
    }

    async fn query(&self, _tenant: &str, _filter: Vec<Filter>) -> Vec<Message> {
        todo!()
    }

    async fn delete(&self, tenant: &str, cid: String) -> Result<(), SurrealDBError> {
        let tdb = self.db.clone();
        tdb.use_ns(tenant).use_db(DBNAME).await?;

        tdb.delete::<Option<EncodedMessage>>(("message", cid))
            .await?;

        Ok(())
    }

    async fn clear(&self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Index, IndexValue, MessageStore, SurrealDB};

    #[tokio::test]
    async fn test_surrealdb() {
        let mut db = SurrealDB::new();
        let cwd = std::env::current_dir().unwrap().join("file.db");
        let _ = db
            .connect(format!("file://{file}", file = cwd.to_string_lossy()).as_str())
            .await;
        let _ = db.open().await;
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
                    },
                    authroization: None,
                },
                vec![Index {
                    key: "something".into(),
                    value: IndexValue::Bool(false),
                }],
            )
            .await
            .unwrap();

        let m = db.get("did", cid.to_string()).await.unwrap();
        println!("cid: {}, m: {:?}", cid.to_string(), m);
        //let _ = db.delete("did", cid.to_string()).await;
        let _ = db.close().await;
    }
}
