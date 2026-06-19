//! Data source adapters for fetching context data.

use crate::errors::Result;
use async_trait::async_trait;
use serde_json::Value;

/// Trait for data sources that can fetch JSON data.
#[allow(dead_code)]
#[async_trait]
pub trait DataSource: Send + Sync {
    /// Fetch data from the source and return as JSON Value.
    async fn fetch(&self) -> Result<Value>;
}

pub mod csv;
pub mod db;
pub mod http;
pub mod json;

#[allow(unused_imports)]
pub use csv::CsvSource;
#[allow(unused_imports)]
pub use db::DbSource;
#[allow(unused_imports)]
pub use http::HttpSource;
#[allow(unused_imports)]
pub use json::JsonSource;
