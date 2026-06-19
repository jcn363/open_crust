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

    /// Anyhow error.
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    /// Handlebars template error.
    #[error("Handlebars error: {0}")]
    HandlebarsError(#[from] handlebars::TemplateError),

    /// Tera template error.
    #[error("Tera error: {0}")]
    TeraError(#[from] tera::Error),

    /// Tokio JoinError.
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    /// Handlebars render error.
    #[error("Render error: {0}")]
    RenderError(#[from] handlebars::RenderError),
}

/// Errors from data source adapters.
#[allow(dead_code)]
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

/// Result type for codegen operations.
pub type Result<T> = std::result::Result<T, CodegenError>;
