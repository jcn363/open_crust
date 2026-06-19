//! Data source adapters for fetching context data.

use async_trait::async_trait;
use serde_json::Value;
use crate::errors::{DataSourceError, Result};

/// Trait for data sources that can fetch JSON data.
#[async_trait]
pub trait DataSource: Send + Sync {
    /// Fetch data from the source and return as JSON Value.
    async fn fetch(&self) -> Result<Value>;
}

pub mod csv;
pub mod json;
pub mod http;
pub mod db;

pub use csv::CsvSource;
pub use json::JsonSource;
pub use http::HttpSource;
pub use db::DbSource;