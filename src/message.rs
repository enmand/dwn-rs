use async_trait::async_trait;
use jose_jws::General as JWS;
use serde::{Deserialize, Serialize};
use surrealdb::engine::{local::Db};
use thiserror::Error;

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

pub enum Index {
    Key(String),
    Value(IndexValue),
}

pub enum IndexValue {
    Bool(bool),
    String(String),
}

#[async_trait]
pub trait MessageStore {
    async fn open(&self);

    async fn close(&self);

    async fn put(&self, message: &Message, indexes: Vec<Index>);

    async fn get(&self, cid: String) -> Message;

    async fn query(&self, filter: Vec<Filter>) -> Vec<Message>;

    async fn delete(&self, cid: String);

    async fn clear(&self);
}

#[derive(Error, Debug)]
pub enum SurrealDBError {
    #[error("SurrealDBError: {0}")]
    ConnectionError(#[from] surrealdb::Error),
}

pub struct SurrealDB {
    db: surrealdb::Surreal<Db>,
}

impl SurrealDB {
    pub fn new() -> Self {
        Self {
            db: surrealdb::Surreal::init(),
        }
    }

    pub fn with_db(&mut self, db: surrealdb::Surreal<Db>) {
        self.db = db;
    }

    pub async fn connect(&mut self, connstr: &str) -> Result<(), SurrealDBError> {
        let conn = surrealdb::Surreal::new::<surrealdb::engine::local::RocksDb>(connstr);

        // await the connect and return ConnectionError if that errors
        // otherwise, unwrap the connection and return Ok(())
        match conn.await {
            Ok(db) => {
                self.db = db;
                Ok(())
            }
            Err(e) => Err(SurrealDBError::ConnectionError(e)),
        }
    }
}

#[async_trait]
impl MessageStore for SurrealDB {
    async fn open(&self) {
        self.db.use_ns("dwn");
        println!("{}", self.db.version().await.unwrap());
    }

    async fn close(&self) {
        todo!()
    }

    async fn put(&self, _message: &Message, _indexes: Vec<Index>) {
        todo!()
    }

    async fn get(&self, _cid: String) -> Message {
        todo!()
    }

    async fn query(&self, _filter: Vec<Filter>) -> Vec<Message> {
        todo!()
    }

    async fn delete(&self, _cid: String) {
        todo!()
    }

    async fn clear(&self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::MessageStore;

    #[test]
    fn test_surrealdb() {
        let fut = async move {
            let mut db = crate::SurrealDB::new();
            let conn = surrealdb::Surreal::new::<surrealdb::engine::local::RocksDb>("temp.db")
                .await
                .unwrap();
            db.with_db(conn);
            db.open().await;
        };

        use tokio::runtime::Handle;

        tokio::task::block_in_place(move || {
            Handle::current().block_on(fut);
        });
    }
}
