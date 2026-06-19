//! CSV data source adapter.

use std::path::Path;
use async_trait::async_trait;
use csv::ReaderBuilder;
use serde_json::{Value, json};
use crate::adapters::DataSource;
use crate::errors::Result;

/// CSV data source that reads a CSV file and returns an array of objects.
pub struct CsvSource {
    path: String,
    has_headers: bool,
    delimiter: u8,
}

impl CsvSource {
    /// Create a new CSV source from a file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().to_string(),
            has_headers: true,
            delimiter: b',',
        }
    }

    /// Set whether the CSV has headers (default: true).
    pub fn with_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    /// Set the delimiter character (default: ',').
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }
}

#[async_trait]
impl DataSource for CsvSource {
    async fn fetch(&self) -> Result<Value> {
        let path = self.path.clone();
        let has_headers = self.has_headers;
        let delimiter = self.delimiter;

        tokio::task::spawn_blocking(move || {
            let mut reader = ReaderBuilder::new()
                .has_headers(has_headers)
                .delimiter(delimiter)
                .from_path(&path)?;

            let headers = if has_headers {
                reader.headers()?.clone()
            } else {
                // Generate default headers if none
                let first_record = reader.records().next().transpose()?;
                let count = first_record.as_ref().map(|r| r.len()).unwrap_or(0);
                (0..count).map(|i| format!("col{}", i)).collect()
            };

            let mut records = Vec::new();
            for result in reader.records() {
                let record = result?;
                let mut obj = serde_json::Map::new();
                for (i, field) in record.iter().enumerate() {
                    if let Some(header) = headers.get(i) {
                        // Try to parse as number, fallback to string
                        let value: Value = if let Ok(n) = field.parse::<i64>() {
                            json!(n)
                        } else if let Ok(f) = field.parse::<f64>() {
                            json!(f)
                        } else if let Ok(b) = field.parse::<bool>() {
                            json!(b)
                        } else {
                            json!(field)
                        };
                        obj.insert(header.to_string(), value);
                    }
                }
                records.push(Value::Object(obj));
            }

            Ok(json!(records))
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
    async fn test_csv_source_with_headers() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,active").unwrap();
        writeln!(file, "Alice,30,true").unwrap();
        writeln!(file, "Bob,25,false").unwrap();
        file.flush().unwrap();

        let source = CsvSource::new(file.path());
        let result = source.fetch().await.unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[0]["age"], 30);
        assert_eq!(arr[0]["active"], true);
    }

    #[tokio::test]
    async fn test_csv_source_without_headers() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Alice,30,true").unwrap();
        writeln!(file, "Bob,25,false").unwrap();
        file.flush().unwrap();

        let source = CsvSource::new(file.path()).with_headers(false);
        let result = source.fetch().await.unwrap();

        assert!(result.is_array());
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["col0"], "Alice");
        assert_eq!(arr[0]["col1"], 30);
        assert_eq!(arr[0]["col2"], true);
    }
}