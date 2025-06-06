//! Storage backends for CosmoFlow
//!
//! This module provides various storage backend implementations:
//!
//! - Memory storage (feature: `memory`)
//! - File storage (feature: `file`)

use serde::{Serialize, de::DeserializeOwned};
use std::error::Error;

// ============================================================================
// STORAGE TRAITS
// ============================================================================

/// Trait defining the interface for storage backends used by SharedStore
///
/// The storage backend works with serializable/deserializable types internally,
/// providing flexibility in how data is stored and retrieved.
pub trait StorageBackend: Send + Sync {
    /// Error type returned by storage operations
    type Error: Error + Send + Sync + 'static;

    /// Store a value with the given key
    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error>;

    /// Retrieve a value by key
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;

    /// Remove a value by key, returning it if it existed
    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error>;

    /// Check if a key exists
    fn contains_key(&self, key: &str) -> Result<bool, Self::Error>;

    /// Get all keys
    fn keys(&self) -> Result<Vec<String>, Self::Error>;

    /// Clear all data
    fn clear(&mut self) -> Result<(), Self::Error>;

    /// Get the number of stored items
    fn len(&self) -> Result<usize, Self::Error>;

    /// Check if the storage is empty
    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.len()? == 0)
    }
}

// ============================================================================
// STORAGE IMPLEMENTATIONS (feature-gated)
// ============================================================================

// Memory storage
#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use memory::{MemoryStorage, MemoryStorageError};

// File storage
#[cfg(feature = "file")]
mod file;
#[cfg(feature = "file")]
pub use file::{FileStorage, FileStorageError};
