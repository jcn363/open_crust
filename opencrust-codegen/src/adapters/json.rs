//! JSON data source adapter.

use std::path::Path;
use async_trait::async_trait;
use serde_json::Value;
use crate::adapters::DataSource;
use crate::errors::Result;

/// JSON data source that loads a JSON file.
pub struct JsonSource {
    path: String,
}

impl JsonSource {
    /// Create a new JSON source from a file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().to_string(),
        }
    }
}

#[async_trait]
impl DataSource for JsonSource {
    async fn fetch(&self) -> Result<Value> {
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let content = std::fs::read_to_string(&path)?;
            let value: Value = serde_json::from_str(&content)?;
            Ok(value)
        })
        .await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_json_source_object() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"name": "Alice", "age": 30}}"#).unwrap();
        file.flush().unwrap();

        let source = JsonSource::new(file.path());
        let result = source.fetch().await.unwrap();

        assert!(result.is_object());
        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
    }

    #[tokio::test]
    async fn test_json_source_array() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"[1, 2, 3]"#).unwrap();
        file.flush().unwrap();

        let source = JsonSource::new(file.path());
        let result = source.fetch().await.unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }
}