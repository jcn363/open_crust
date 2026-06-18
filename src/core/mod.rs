//! Core services and utilities for OpenCrust
//!
//! This module provides the central service registry for dependency injection
//! following the vLLM Phase 1 pattern.
//!
//! # Service Registry
//!
//! The service registry provides a type-safe way to register and retrieve services.
//! Macros are exported at crate root level:
//! - `register_service!()` - Register a service
//! - `get_service!()` - Get a registered service
//! - `get_or_init_service!()` - Get or initialize a service with a factory
//!
//! # Example
//!
//! ```rust,ignore
//! use std::sync::Arc;
//!
//! // Register a service
//! let service = Arc::new(MyService::new());
//! register_service!(service);
//!
//! // Get the service
//! let retrieved: Option<Arc<MyService>> = get_service!(MyService);
//! ```

pub mod services;
