//! Storage backend implementations for CosmoFlow
//!
//! This module provides the core storage abstraction and various backend implementations
//! for persisting workflow data. Storage backends are responsible for serializing,
//! storing, and retrieving data in different formats and locations.
//!
//! # Available Backends
//!
//! - **Memory Storage** (feature: `storage-memory`) - Fast in-memory storage using HashMap
//! - **File Storage** (feature: `storage-file`) - JSON-based persistent file storage  
//! - **Redis Storage** (feature: `storage-redis`) - High-performance distributed storage using Redis
//!
//! # Backend Selection
//!
//! Choose the appropriate backend based on your use case:
//!
//! | Backend | Use Case | Persistence | Performance | Memory Usage | Scalability |
//! |---------|----------|-------------|-------------|--------------|-------------|
//! | Memory  | Development, Testing | None | Fastest | High | Single Process |
//! | File    | Small workflows | Full | Fast | Low | Single Process |
//! | Redis   | Production, Distributed | Full | Very Fast | Low | Multi-Process |
//!
//! # Custom Backends
//!
//! You can implement the [`crate::shared_store::SharedStore`] trait to create custom storage solutions:
//!
//! ```rust
//! use cosmoflow::shared_store::SharedStore;
//! use serde::{Serialize, de::DeserializeOwned};
//! use thiserror::Error;
//!
//! #[derive(Debug, Error)]
//! enum CustomError {
//!     #[error("Custom storage error: {0}")]
//!     Custom(String),
//! }
//!
//! struct DatabaseBackend {
//!     // Database connection, configuration, etc.
//! }
//!
//! impl SharedStore for DatabaseBackend {
//!     type Error = CustomError;
//!
//!     fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
//!         // Implement database storage
//!         # Ok(())
//!     }
//!
//!     fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
//!         // Implement database retrieval
//!         # Ok(None)
//!     }
//!
//!     // ... implement other required methods
//!     # fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> { Ok(None) }
//!     # fn contains_key(&self, key: &str) -> Result<bool, Self::Error> { Ok(false) }
//!     # fn keys(&self) -> Result<Vec<String>, Self::Error> { Ok(vec![]) }
//!     # fn clear(&mut self) -> Result<(), Self::Error> { Ok(()) }
//!     # fn len(&self) -> Result<usize, Self::Error> { Ok(0) }
//! }
//! ```

// ============================================================================
// STORAGE IMPLEMENTATIONS (feature-gated)
// ============================================================================

// Memory storage
#[cfg(feature = "storage-memory")]
mod memory;
#[cfg(feature = "storage-memory")]
pub use memory::{MemoryStorage, MemoryStorageError};

// File storage
#[cfg(feature = "storage-file")]
mod file;
#[cfg(feature = "storage-file")]
pub use file::{FileStorage, FileStorageError};

// Redis storage
#[cfg(feature = "storage-redis")]
mod redis;
#[cfg(feature = "storage-redis")]
pub use redis::{RedisStorage, RedisStorageError};
