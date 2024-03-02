use std::sync::Arc;

use surrealdb::{engine::any::Any, Surreal};
use ulid::Generator;

use crate::StoreError;

use super::errors::SurrealDBError;

const NAMESPACE: &str = "dwn";

// Database is a enum of databases that can be used with SurrealDB stores
#[derive(Debug, Clone, Copy)]
pub enum Database {
    None,
    Messages,
    Data,
    Events,
}

impl From<Database> for String {
    fn from(db: Database) -> Self {
        match db {
            Database::None => "".into(),
            Database::Messages => "messages".into(),
            Database::Data => "data".into(),
            Database::Events => "events".into(),
        }
    }
}

pub struct SurrealDB {
    pub(super) db: Arc<Surreal<Any>>,
    pub(super) db_name: Database,
    pub(super) _constr: String,

    pub(super) ulid_generator: Generator,
}

impl Default for SurrealDB {
    fn default() -> Self {
        Self::new()
    }
}

impl SurrealDB {
    pub(super) async fn open(&mut self) -> Result<(), StoreError> {
        let health = self.db.health().await;
        if health.is_err() {
            if self._constr.is_empty() {
                return Err(StoreError::NoInitError);
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

    pub(super) async fn close(&mut self) {
        let _ = self.db.invalidate().await;
    }

    pub fn new() -> Self {
        Self {
            db: Arc::new(surrealdb::Surreal::init()),
            db_name: Database::None,
            _constr: String::new(),

            ulid_generator: Generator::new(),
        }
    }

    pub fn with_db(&mut self, db: surrealdb::Surreal<Any>) -> &mut Self {
        self.db = Arc::new(db);
        self
    }

    pub async fn connect(
        &mut self,
        connstr: &str,
        db_name: Database,
    ) -> Result<(), SurrealDBError> {
        self.db_name = db_name;
        self._constr = connstr.into();
        self.db.connect(connstr).await?;
        self.db
            .health()
            .await
            .map_err(Into::<SurrealDBError>::into)?;
        self.db
            .use_ns(NAMESPACE)
            .use_db(self.db_name)
            .await
            .map_err(Into::into)
    }

    pub(super) async fn clear(&self) -> Result<(), StoreError> {
        self.db
            .query(format!(
                "REMOVE DATABASE {}",
                Into::<String>::into(self.db_name)
            ))
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?;

        Ok(())
    }
}
