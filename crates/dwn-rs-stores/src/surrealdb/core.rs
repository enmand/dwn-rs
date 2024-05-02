use std::sync::Arc;

use surrealdb::{
    engine::any::Any,
    iam::Level,
    opt::auth::{self},
    Surreal,
};
use ulid::Generator;

use crate::{surrealdb::auth::Auth, StoreError};

use super::errors::SurrealDBError;

pub struct SurrealDB {
    pub(super) db: Arc<Surreal<Any>>,
    constr: String,
    invalid: bool,

    pub(super) ulid_generator: Generator,
}

impl Default for SurrealDB {
    fn default() -> Self {
        Self::new()
    }
}

const META_DB: &str = "_meta";

impl SurrealDB {
    pub(super) async fn open(&mut self) -> Result<(), StoreError> {
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

    pub(super) async fn close(&mut self) {
        let _ = self.db.invalidate().await;
        self.invalid = true;
    }

    pub fn new() -> Self {
        Self {
            db: Arc::new(surrealdb::Surreal::init()),
            constr: String::new(),
            invalid: false,

            ulid_generator: Generator::new(),
        }
    }

    pub fn with_db(&mut self, db: surrealdb::Surreal<Any>) -> &mut Self {
        self.db = Arc::new(db);
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


        }
    }
}


    }
}
