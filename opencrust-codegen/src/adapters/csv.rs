//! CSV data source adapter.

use crate::adapters::DataSource;
use crate::errors::Result;
use async_trait::async_trait;
use csv::ReaderBuilder;
use serde_json::{Value, json};
use std::path::Path;

/// CSV data source that reads a CSV file and returns an array of objects.
#[allow(dead_code)]
pub struct CsvSource {
    path: String,
    has_headers: bool,
    delimiter: u8,
}

impl CsvSource {
    /// Create a new CSV source from a file path.
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().to_string(),
            has_headers: true,
            delimiter: b',',
        }
    }

    /// Set whether the CSV has headers (default: true).
    #[allow(dead_code)]
    pub fn with_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    /// Set the delimiter character (default: ',').
    #[allow(dead_code)]
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
            // Always read with has_headers=false to get all rows as data
            let mut reader = ReaderBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .from_path(&path)?;

            // Collect all records first
            let all_records: std::result::Result<Vec<_>, csv::Error> = reader.records().collect();
            let all_records = all_records?;

            let headers = if has_headers {
                // Use first record as headers, process remaining records
                if let Some(first_record) = all_records.first() {
                    first_record
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            } else {
                // Generate default headers from first record's column count
                if let Some(first_record) = all_records.first() {
                    (0..first_record.len())
                        .map(|i| format!("col{}", i))
                        .collect()
                } else {
                    Vec::new()
                }
            };

            let mut records = Vec::new();
            let start_idx = if has_headers { 1 } else { 0 };

            for record in all_records.iter().skip(start_idx) {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

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
