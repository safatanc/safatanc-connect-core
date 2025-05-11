use async_trait::async_trait;
use sqlx::{postgres::PgRow, Executor, PgPool, Postgres, Transaction};
use std::future::Future;

/// DbExecutor trait provides an abstraction for executing queries,
/// both on pool and within a transaction
#[async_trait]
pub trait DbExecutor: Send + Sync {
    async fn fetch_one<'q, E>(&self, query: E) -> Result<PgRow, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>;

    async fn fetch_optional<'q, E>(&self, query: E) -> Result<Option<PgRow>, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>;

    async fn fetch_all<'q, E>(&self, query: E) -> Result<Vec<PgRow>, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>;

    async fn execute<'q, E>(&self, query: E) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>;
}

// Implementation for PgPool
#[async_trait]
impl DbExecutor for PgPool {
    async fn fetch_one<'q, E>(&self, query: E) -> Result<PgRow, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .fetch_one(self)
            .await
    }

    async fn fetch_optional<'q, E>(&self, query: E) -> Result<Option<PgRow>, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .fetch_optional(self)
            .await
    }

    async fn fetch_all<'q, E>(&self, query: E) -> Result<Vec<PgRow>, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .fetch_all(self)
            .await
    }

    async fn execute<'q, E>(&self, query: E) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .execute(self)
            .await
    }
}

// Implementation for Transaction<'_, Postgres>
#[async_trait]
impl DbExecutor for Transaction<'_, Postgres> {
    async fn fetch_one<'q, E>(&self, query: E) -> Result<PgRow, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .fetch_one(self)
            .await
    }

    async fn fetch_optional<'q, E>(&self, query: E) -> Result<Option<PgRow>, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .fetch_optional(self)
            .await
    }

    async fn fetch_all<'q, E>(&self, query: E) -> Result<Vec<PgRow>, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .fetch_all(self)
            .await
    }

    async fn execute<'q, E>(&self, query: E) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error>
    where
        E: 'q + Executor<'q, Database = Postgres>,
    {
        sqlx::query::Query::<Postgres, _>::from(query)
            .execute(self)
            .await
    }
}

// Helper function to run a transaction
pub async fn run_transaction<F, T, E>(pool: &PgPool, f: F) -> Result<T, E>
where
    F: for<'c> FnOnce(
        &'c mut Transaction<'_, Postgres>,
    ) -> Box<dyn Future<Output = Result<T, E>> + Send + 'c>,
    E: From<sqlx::Error>,
{
    let mut tx = pool.begin().await.map_err(E::from)?;

    let result = f(&mut tx).await;

    match result {
        Ok(value) => {
            tx.commit().await.map_err(E::from)?;
            Ok(value)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}
