//! Error types for the codegen crate.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during code generation.
#[derive(Error, Debug)]
pub enum CodegenError {
    /// Template not found at the given path.
    #[error("Template not found: {path}")]
    TemplateNotFound { path: PathBuf },

    /// Failed to parse template.
    #[error("Template parse error at {path}:{line}:{column}: {message}")]
    TemplateParseError {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    /// Failed to render template.
    #[error("Template render error: {message}")]
    TemplateRenderError { message: String },

    /// Data source error.
    #[error("Data source error: {source}")]
    DataSourceError {
        #[from]
        source: DataSourceError,
    },

    /// I/O error.
    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    /// JSON serialization/deserialization error.
    #[error("JSON error: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },

    /// HTTP request error.
    #[error("HTTP error: {source}")]
    HttpError {
        #[from]
        source: reqwest::Error,
    },

    /// Database error.
    #[error("Database error: {source}")]
    DatabaseError {
        #[from]
        source: sqlx::Error,
    },

    /// CSV error.
    #[error("CSV error: {source}")]
    CsvError {
        #[from]
        source: csv::Error,
    },

    /// Variable not found in context.
    #[error("Variable not found in context: {variable}")]
    VariableNotFound { variable: String },

    /// Invalid template syntax.
    #[error("Invalid template syntax: {engine} template syntax: {message}")]
    InvalidSyntax { engine: String, message: String },

    /// Output write error.
    #[error("Failed to write output to {path}: {source}")]
    OutputWriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Errors from data source adapters.
#[derive(Error, Debug)]
pub enum DataSourceError {
    /// CSV data source error.
    #[error("CSV error: {source}")]
    Csv {
        #[from]
        source: csv::Error,
    },

    /// JSON data source error.
    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    /// HTTP data source error.
    #[error("HTTP error: {source}")]
    Http {
        #[from]
        source: reqwest::Error,
    },

    /// Database data source error.
    #[error("Database error: {source}")]
    Database {
        #[from]
        source: sqlx::Error,
    },

    /// I/O error.
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Generic error with message.
    #[error("{message}")]
    Other { message: String },
}

impl From<DataSourceError> for CodegenError {
    fn from(err: DataSourceError) -> Self {
        CodegenError::DataSourceError { source: err }
    }
}

/// Result type for codegen operations.
pub type Result<T> = std::result::Result<T, CodegenError>;