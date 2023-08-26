use std::{collections::TryReserveError, sync::Arc};

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use cid::multihash::{Code, MultihashDigest};
use jose_jws::General as JWS;
use serde::{Deserialize, Serialize, Serializer};
use surrealdb::engine::any::Any;
use thiserror::Error;

const DBNAME: &str = "messages";
const CBOR_TAGS_CID: u64 = 42;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    pub descriptor: Descriptor,
    authroization: Option<JWS>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Descriptor {
    pub interface: String,
    pub method: String,
    pub timestamp: u64,
}

pub enum Filter {
    Property(String),
    Filter(/* filter types */),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Index {
    Key(String),
    Value(IndexValue),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IndexValue {
    Bool(bool),
    String(String),
}

#[derive(Serialize, Deserialize, Debug)]
struct EncodedMessage {
    // serialize as a string
    #[serde(serialize_with = "serialize_base64")]
    encoded_message: Vec<u8>,
    #[serde(serialize_with = "serialize_cid")]
    cid: cid::Cid,
    //indexes: Vec<Index>,
}

fn serialize_base64<T: Serializer>(bytes: &[u8], serializer: T) -> Result<T::Ok, T::Error> {
    serializer.serialize_str(&general_purpose::STANDARD.encode(bytes))
}

fn serialize_cid<T: Serializer>(cid: &cid::Cid, serializer: T) -> Result<T::Ok, T::Error> {
    serializer.serialize_str(&cid.to_string())
}

#[async_trait]
pub trait MessageStore {
    async fn open(&self) -> Result<(), SurrealDBError>;

    async fn close(&self);

    async fn put(
        &self,
        tenant: &str,
        message: &Message,
        indexes: Vec<Index>,
    ) -> Result<(), SurrealDBError>;

    async fn get(&self, tenant: &str, cid: String) -> Message;

    async fn query(&self, tenant: &str, filter: Vec<Filter>) -> Vec<Message>;

    async fn delete(&self, tenant: &str, cid: String);

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

    #[error("failed to encode cid")]
    CidEncodeError(#[from] multihash::Error),
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

    async fn close(&self) {
        let _ = self.db.invalidate().await;
    }

    async fn put(
        &self,
        tenant: &str,
        message: &Message,
        indexes: Vec<Index>,
    ) -> Result<(), SurrealDBError> {
        let tdb = self.db.clone();
        tdb.use_ns(tenant).use_db(DBNAME).await?;

        // todo this should be a multiformat custom code for Block
        let encoded_message = serde_ipld_dagcbor::to_vec(message)?;
        let hash = Code::Sha2_256.digest(&encoded_message);
        let cid = cid::Cid::new_v1(CBOR_TAGS_CID, hash);

        println!("cid: {}", cid.to_string());
        println!(
            "encoded_message: {}",
            general_purpose::STANDARD.encode(&encoded_message)
        );

        self.db
            .create::<Option<EncodedMessage>>(("message", cid.to_string()))
            .content(EncodedMessage {
                encoded_message,
                cid,
                //indexes,
            })
            .await?;

        Ok(())
    }

    async fn get(&self, _tenant: &str, _cid: String) -> Message {
        todo!()
    }

    async fn query(&self, _tenant: &str, _filter: Vec<Filter>) -> Vec<Message> {
        todo!()
    }

    async fn delete(&self, _tenant: &str, _cid: String) {
        todo!()
    }

    async fn clear(&self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::MessageStore;

    #[tokio::test]
    async fn test_surrealdb() {
        let mut db = crate::SurrealDB::new();
        let cwd = std::env::current_dir().unwrap().join("file.db");
        db.connect(format!("file://{file}", file = cwd.to_string_lossy()).as_str())
            .await;
        db.open().await;
        db.put("did", &crate::Message::default(), vec![]).await;
        db.close().await;
    }
}
