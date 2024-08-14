use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use surrealdb::sql::{
    statements::{BeginStatement, CommitStatement, SelectStatement, SetStatement, UpdateStatement},
    Cond, Data, Duration, Expression, Function, Idiom, Limit, Number, Operator, Output, Param,
    Statement, Subquery, Table, Value as SurrealValue,
};

use crate::{
    surrealdb::models::{CreateTask, ExtendTask, ExtendedTask, Task},
    SurrealDB, SurrealDBError,
};
use dwn_rs_core::{
    errors::{ResumableTaskStoreError, StoreError},
    stores::{ManagedResumableTask, ResumableTaskStore},
};

use ulid::Ulid;

const RESUMABLE_TASKS_DB: &str = "tasks";
const RESUMABLE_TASKS_TABLE: &str = "resumable_tasks";
const TASK_TIMEOUT: u64 = 60;

impl ResumableTaskStore for SurrealDB {
    async fn open(&mut self) -> Result<(), ResumableTaskStoreError> {
        self.db = self.db.clone();

        self.open().await.map_err(ResumableTaskStoreError::from)?;

        self.db
            .use_db(RESUMABLE_TASKS_DB)
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?;

        Ok(())
    }

    async fn close(&mut self) {
        self.close().await
    }

    async fn register<T: Serialize + DeserializeOwned + Sync + Send + Debug>(
        &self,
        task: T,
        timeout: u64,
    ) -> Result<ManagedResumableTask<T>, ResumableTaskStoreError> {
        let id = self.gen.lock().await.generate()?;
        let timeout_expr = timeout_expr(timeout);

        match self
            .db
            .create::<Option<Task<T>>>((RESUMABLE_TASKS_TABLE, id.to_string()))
            .content(CreateTask {
                task,
                timeout: timeout_expr,
            })
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?
        {
            Some(task) => Ok(ManagedResumableTask {
                id,
                task: task.task,
                timeout,
                retry_count: 0,
            }),
            None => Err(ResumableTaskStoreError::StoreError(
                StoreError::InternalException("Failed to register task".to_string()),
            )),
        }
    }

    async fn grab<T: Serialize + Send + Sync + DeserializeOwned + Debug + Unpin>(
        &self,
        count: u64,
    ) -> Result<Vec<ManagedResumableTask<T>>, ResumableTaskStoreError> {
        // select * from resumable_tasks where timeout < time::now() limit count
        // we can't use the update statement with a where, because we have a limit
        // so we need to do a select first, then update in a transaction
        let mut select_stmt = SelectStatement::default();
        select_stmt.expr.0.push(surrealdb::sql::Field::All);
        select_stmt
            .what
            .0
            .push(Table::from(RESUMABLE_TASKS_TABLE).into());

        let mut limit = Limit::default();
        limit.0 = SurrealValue::Number(Number::from(count));
        select_stmt.limit = Some(limit);

        let timeout_field = SurrealValue::Idiom("timeout".into());

        let mut cond = Cond::default();
        cond.0 = SurrealValue::Expression(Box::new(Expression::Binary {
            l: timeout_field.clone(),
            o: Operator::LessThanOrEqual,
            r: SurrealValue::Function(Box::new(Function::Normal("time::now".into(), vec![]))),
        }));
        select_stmt.cond = Some(cond);

        // let statement for updates
        let mut let_stmt = SetStatement::default();
        let_stmt.name = "tasks".into();
        let_stmt.what = SurrealValue::Subquery(Box::new(Subquery::Select(select_stmt)));

        // update statement
        let mut update_stmt = UpdateStatement::default();
        update_stmt
            .what
            .0
            .push(SurrealValue::Param(Param::from(let_stmt.name.clone())));
        update_stmt.output = Some(Output::Before);
        update_stmt.data = Some(Data::SetExpression(vec![(
            Idiom::from(timeout_field.clone().as_string()),
            Operator::Equal,
            SurrealValue::Expression(Box::new(Expression::Binary {
                l: timeout_field.clone(),
                o: Operator::Add,
                r: SurrealValue::Duration(Duration::from_secs(TASK_TIMEOUT)),
            })),
        )]));

        // transaction for atomic updates on tasks
        let grab_stmt: Vec<Statement> = vec![
            Statement::Begin(BeginStatement::default()),
            Statement::Set(let_stmt),
            Statement::Update(update_stmt),
            Statement::Commit(CommitStatement::default()),
        ];

        let tasks: Vec<Task<T>> = self
            .db
            .query(grab_stmt)
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?
            .take(1)
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?;

        let managed_tasks: Result<Vec<ManagedResumableTask<T>>, ResumableTaskStoreError> = tasks
            .into_iter()
            .map(|task| {
                Ok(ManagedResumableTask {
                    id: Ulid::from_string(&task.id.id.to_string())?,
                    task: task.task,
                    timeout: task.timeout.timestamp() as u64,
                    retry_count: 0,
                })
            })
            .collect();

        Ok(managed_tasks?)
    }

    async fn read<T: Serialize + Send + Sync + DeserializeOwned + Debug>(
        &self,
        task_id: String,
    ) -> Result<Option<ManagedResumableTask<T>>, ResumableTaskStoreError> {
        match self
            .db
            .select::<Option<Task<T>>>((RESUMABLE_TASKS_TABLE, task_id))
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?
        {
            Some(task) => Ok(Some(ManagedResumableTask {
                id: Ulid::from_string(&task.id.id.to_string())?,
                task: task.task,
                timeout: task.timeout.timestamp() as u64,
                retry_count: 0,
            })),
            None => Ok(None),
        }
    }

    async fn extend(&self, task_id: String, timeout: u64) -> Result<(), ResumableTaskStoreError> {
        let timeout = timeout_expr(timeout);
        self.db
            .update::<Option<ExtendedTask>>((RESUMABLE_TASKS_TABLE, task_id))
            .merge(ExtendTask { timeout })
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?;

        Ok(())
    }

    async fn delete(&self, task_id: String) -> Result<(), ResumableTaskStoreError> {
        self.db
            .delete::<Option<ExtendedTask>>((RESUMABLE_TASKS_TABLE, task_id))
            .await
            .map_err(SurrealDBError::from)
            .map_err(StoreError::from)?;

        Ok(())
    }

    async fn clear(&self) -> Result<(), ResumableTaskStoreError> {
        self.clear(&RESUMABLE_TASKS_TABLE.into())
            .await
            .map_err(ResumableTaskStoreError::from)?;

        Ok(())
    }
}

pub fn timeout_expr(timeout: u64) -> SurrealValue {
    SurrealValue::Expression(Box::new(Expression::Binary {
        l: SurrealValue::Function(Box::new(Function::Normal("time::now".into(), vec![]))),
        o: Operator::Add,
        r: SurrealValue::Duration(Duration::from_secs(timeout)),
    }))
}
