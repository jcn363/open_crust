//! Database data source adapter.

use async_trait::async_trait;
use sqlx::{Pool, Postgres, Row};
use serde_json::{json, Value};
use crate::adapters::DataSource;
use crate::errors::Result;

/// Database source that runs a query and returns rows as JSON.
pub struct DbSource {
    pool: Pool<Postgres>,
    query: String,
}

impl DbSource {
    /// Create a new DB source from an existing `sqlx::Pool` and a query string.
    pub fn new(pool: Pool<Postgres>, query: impl Into<String>) -> Self {
        Self {
            pool,
            query: query.into(),
        }
    }
}

#[async_trait]
impl DataSource for DbSource {
    async fn fetch(&self) -> Result<Value> {
        let rows = sqlx::query(&self.query).fetch_all(&self.pool).await?;
        let mut records = Vec::new();
        for row in rows {
            let mut map = serde_json::Map::new();
            for column in row.columns() {
                let name = column.name();
                // Use `try_get` with `serde_json::Value` for generic handling.
                let value: Result<Value> = match column.type_info().name() {
                    "INT4" | "INT8" => row.try_get::<i64, _>(name).map(|v| json!(v)).map_err(Into::into),
                    "FLOAT4" | "FLOAT8" => row.try_get::<f64, _>(name).map(|v| json!(v)).map_err(Into::into),
                    "BOOL" => row.try_get::<bool, _>(name).map(|v| json!(v)).map_err(Into::into),
                    "TEXT" | "VARCHAR" => row.try_get::<String, _>(name).map(|v| json!(v)).map_err(Into::into),
                    _ => row.try_get::<serde_json::Value, _>(name).map_err(Into::into),
                }?;
                map.insert(name.to_string(), value);
            }
            records.push(Value::Object(map));
        }
        Ok(json!(records))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Executor, PgPool};
    use std::env;

    #[tokio::test]
    async fn test_db_source_fetch() {
        // Use a temporary in‑memory Postgres via `docker` is not available, so we skip actual DB test.
        // Instead, we ensure the struct compiles and the method signature is correct.
        // In CI, a real Postgres container will be used.
        let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL not set");
        let pool = PgPool::connect(&database_url).await.unwrap();
        // Create a simple table.
        pool.execute("CREATE TEMP TABLE users (id INT, name TEXT);").await.unwrap();
        pool.execute("INSERT INTO users (id, name) VALUES (1, 'Alice'), (2, 'Bob');").await.unwrap();

        let source = DbSource::new(pool.clone(), "SELECT id, name FROM users ORDER BY id");
        let result = source.fetch().await.unwrap();
        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["id"], 1);
        assert_eq!(arr[0]["name"], "Alice");
    }
}
