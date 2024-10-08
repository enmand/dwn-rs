use std::{collections::BTreeMap, fmt::Debug};

use surrealdb::{
    engine::any::Any,
    iam::Level,
    opt::auth::{self},
    sql::{
        statements::{RemoveStatement, RemoveTableStatement},
        Table,
    },
    Surreal,
};
use tokio::sync::Mutex;
use ulid::Generator;

use crate::surrealdb::auth::Auth;
use dwn_rs_core::errors::StoreError;

use super::errors::SurrealDBError;

pub struct SurrealDB {
    pub(super) db: Surreal<Any>,
    constr: String,
    invalid: bool,

    pub(super) gen: Mutex<Generator>,
}

impl Debug for SurrealDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealDB")
            .field("constr", &self.constr)
            .field("invalid", &self.invalid)
            .finish()
    }
}

impl Default for SurrealDB {
    fn default() -> Self {
        Self::new()
    }
}

const META_DB: &str = "_meta";

impl SurrealDB {
    pub async fn open(&mut self) -> Result<(), StoreError> {
        let health = self.db.health().await;
        if health.is_err() || self.invalid {
            if self.constr.is_empty() {
                return Err(StoreError::NoInitError);
            } else {
                let connstr = self.constr.clone();
                self.connect(&connstr).await.map_err(SurrealDBError::from)?;
            }
        }

        Ok(())
    }

    pub async fn close(&mut self) {
        let _ = self.db.invalidate().await;
        self.invalid = true;
    }

    pub fn new() -> Self {
        Self {
            db: surrealdb::Surreal::init(),
            constr: String::new(),
            invalid: false,

            gen: Mutex::new(Generator::new()),
        }
    }

    pub fn with_db(&mut self, db: surrealdb::Surreal<Any>) -> &mut Self {
        self.db = db;
        self
    }

    pub async fn connect(&mut self, connstr: &str) -> Result<(), SurrealDBError> {
        self.constr = connstr.into();
        let (connstr, auth) = Auth::parse_connstr(connstr)?;

        if !self.invalid {
            self.db.connect(connstr.clone()).await?;
        }

        if let Some(auth) = auth {
            if auth.has_auth() {
                match auth.level {
                    Level::Root => {
                        self.db
                            .signin(auth::Root {
                                username: auth.username.as_ref().unwrap().as_str(),
                                password: auth.password.as_ref().unwrap().as_str(),
                            })
                            .await
                            .map_err(Into::<SurrealDBError>::into)?;
                    }
                    Level::Namespace(_) => {
                        self.db
                            .signin(auth::Namespace {
                                username: auth.username.as_ref().unwrap().as_str(),
                                password: auth.password.as_ref().unwrap().as_str(),
                                namespace: auth.namespace.as_str(),
                            })
                            .await
                            .map_err(Into::<SurrealDBError>::into)?;
                    }
                    _ => unreachable!(),
                }
            }

            self.db.use_ns(auth.ns()).await?;
        }

        self.db.use_db(META_DB).await?;
        self.db
            .health()
            .await
            .map_err(Into::<SurrealDBError>::into)?;

        Ok(())
    }

    pub(super) async fn clear(&self, table: &Table) -> Result<(), StoreError> {
        let mut res = self
            .db
            .query("INFO FOR NS")
            .await
            .map_err(SurrealDBError::from)?;

        let databases = res
            .take::<Option<BTreeMap<String, String>>>((0, "databases"))
            .map_err(SurrealDBError::from)?;

        for (db_name, _) in databases.unwrap() {
            self.with_database(&db_name, |db| async move {
                let mut rts = RemoveTableStatement::default();
                rts.name = table.to_string().into();
                rts.if_exists = false;
                db.query(RemoveStatement::Table(rts))
                    .bind(("table", table.clone()))
                    .await
                    .map_err(SurrealDBError::from)?;

                Ok(())
            })
            .await?;
        }

        Ok(())
    }

    pub async fn with_database<F, O, Fut>(&self, database: &str, f: F) -> Result<O, StoreError>
    where
        F: FnOnce(Surreal<Any>) -> Fut,
        Fut: std::future::Future<Output = Result<O, StoreError>>,
    {
        let db = self.db.to_owned();
        db.use_db(database)
            .into_owned()
            .await
            .map_err(SurrealDBError::from)?;

        f(db).await
    }
}
