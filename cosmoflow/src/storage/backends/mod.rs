//! Storage backend implementations for CosmoFlow
//!
//! This module provides the core storage abstraction and various backend implementations
//! for persisting workflow data. Storage backends are responsible for serializing,
//! storing, and retrieving data in different formats and locations.
//!
//! # Available Backends
//!
//! - **Memory Storage** (feature: `memory`) - Fast in-memory storage using HashMap
//! - **File Storage** (feature: `file`) - JSON-based persistent file storage
//!
//! # Backend Selection
//!
//! Choose the appropriate backend based on your use case:
//!
//! | Backend | Use Case | Persistence | Performance | Memory Usage |
//! |---------|----------|-------------|-------------|--------------|
//! | Memory  | Development, Testing | None | Fastest | High |
//! | File    | Small workflows | Full | Fast | Low |
//!
//! # Custom Backends
//!
//! You can implement the [`StorageBackend`] trait to create custom storage solutions:
//!
//! ```rust
//! use cosmoflow::storage::backends::StorageBackend;
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
//! impl StorageBackend for DatabaseBackend {
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

use serde::{Serialize, de::DeserializeOwned};
use std::error::Error;

// ============================================================================
// STORAGE TRAITS
// ============================================================================

/// Trait defining the interface for storage backends used by SharedStore
///
/// The `StorageBackend` trait provides a consistent interface for all storage
/// implementations in CosmoFlow. It handles serialization/deserialization
/// automatically and provides a set of standard operations for data persistence.
///
/// # Design Principles
///
/// - **Type Safety**: All operations work with strongly-typed data through Serde
/// - **Error Handling**: Each backend defines its own error type for specific failure modes
/// - **Performance**: Operations are designed to be efficient for the target storage medium
/// - **Flexibility**: Supports both synchronous and asynchronous implementations
///
/// # Required Methods
///
/// Implementers must provide all methods except `is_empty()`, which has a default implementation.
///
/// # Error Handling
///
/// Each backend defines its own error type that must implement:
/// - `std::error::Error` for error trait support
/// - `Send + Sync + 'static` for use in async contexts
///
/// Common error scenarios include:
/// - Serialization/deserialization failures
/// - Storage medium errors (disk full, network failures, etc.)
/// - Key not found (for required operations)
/// - Permission or access errors
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use cosmoflow::storage::backends::StorageBackend;
///
/// // Simple mock storage for demonstration
/// #[derive(Debug)]
/// struct MockStorage;
///
/// #[derive(Debug)]
/// struct MockError;
///
/// impl std::fmt::Display for MockError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "Mock storage error")
///     }
/// }
///
/// impl std::error::Error for MockError {}
///
/// impl StorageBackend for MockStorage {
///     type Error = MockError;
///     
///     fn get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, Self::Error> {
///         Ok(None)
///     }
///     
///     fn set<T: serde::Serialize>(&mut self, _key: String, _value: T) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     
///     fn remove<T: serde::de::DeserializeOwned>(&mut self, _key: &str) -> Result<Option<T>, Self::Error> {
///         Ok(None)
///     }
///     
///     fn contains_key(&self, _key: &str) -> Result<bool, Self::Error> {
///         Ok(false)
///     }
///     
///     fn keys(&self) -> Result<Vec<String>, Self::Error> {
///         Ok(vec![])
///     }
///     
///     fn clear(&mut self) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     
///     fn len(&self) -> Result<usize, Self::Error> {
///         Ok(0)
///     }
/// }
///
/// let mut storage = MockStorage;
///
/// // Store different types of data
/// storage.set("count".to_string(), 42u32).unwrap();
/// storage.set("message".to_string(), "Hello".to_string()).unwrap();
/// ```
///
/// ## Error Handling
///
/// ```rust
/// use cosmoflow::storage::backends::StorageBackend;
///
/// // Simple mock storage for demonstration
/// #[derive(Debug)]
/// struct MockStorage;
///
/// #[derive(Debug)]
/// struct MockError;
///
/// impl std::fmt::Display for MockError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "Mock storage error")
///     }
/// }
///
/// impl std::error::Error for MockError {}
///
/// impl StorageBackend for MockStorage {
///     type Error = MockError;
///     
///     fn get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, Self::Error> {
///         Ok(None)
///     }
///     
///     fn set<T: serde::Serialize>(&mut self, _key: String, _value: T) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     
///     fn remove<T: serde::de::DeserializeOwned>(&mut self, _key: &str) -> Result<Option<T>, Self::Error> {
///         Ok(None)
///     }
///     
///     fn contains_key(&self, _key: &str) -> Result<bool, Self::Error> {
///         Ok(false)
///     }
///     
///     fn keys(&self) -> Result<Vec<String>, Self::Error> {
///         Ok(vec![])
///     }
///     
///     fn clear(&mut self) -> Result<(), Self::Error> {
///         Ok(())
///     }
///     
///     fn len(&self) -> Result<usize, Self::Error> {
///         Ok(0)
///     }
/// }
///
/// let storage = MockStorage;
///
/// // Handle missing keys gracefully
/// match storage.get::<String>("nonexistent") {
///     Ok(Some(value)) => println!("Found: {}", value),
///     Ok(None) => println!("Key not found"),
///     Err(e) => eprintln!("Storage error: {}", e),
/// }
/// ```
///
/// # Performance Considerations
///
/// - **Serialization**: All data goes through Serde serialization/deserialization
/// - **Key Lookups**: Efficiency depends on the underlying storage implementation
/// - **Memory Usage**: Varies significantly between backends (memory vs. file vs. network)
/// - **Concurrency**: Most implementations are not thread-safe; use external synchronization
///
/// # Thread Safety
///
/// The trait requires `Send + Sync`, but individual implementations may not be thread-safe
/// for mutations. The CosmoFlow execution model uses single-threaded workflows, avoiding
/// concurrency issues while allowing multiple workflows to run in parallel.
pub trait StorageBackend: Send + Sync {
    /// Error type returned by storage operations
    ///
    /// Each backend implementation should define a specific error type that
    /// captures the various failure modes of that storage medium.
    type Error: Error + Send + Sync + 'static;

    /// Store a value with the given key
    ///
    /// Serializes the provided value and stores it under the specified key.
    /// If the key already exists, the value is replaced.
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the stored value
    /// * `value` - Any serializable value to store
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Value was successfully stored
    /// * `Err(Self::Error)` - Storage or serialization error occurred
    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error>;

    /// Retrieve a value by key
    ///
    /// Looks up the value associated with the key and deserializes it to the
    /// requested type. Returns `None` if the key doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to look up
    ///
    /// # Returns
    ///
    /// * `Ok(Some(T))` - Value found and successfully deserialized
    /// * `Ok(None)` - Key not found
    /// * `Err(Self::Error)` - Storage or deserialization error occurred
    ///
    /// # Type Safety
    ///
    /// The method will fail with a deserialization error if the stored data
    /// cannot be converted to the requested type.
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;

    /// Remove a value by key, returning it if it existed
    ///
    /// Removes the value associated with the key and returns it if it was present.
    /// This is useful when you need to both remove and process the stored value.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to remove
    ///
    /// # Returns
    ///
    /// * `Ok(Some(T))` - Value was found, deserialized, and removed
    /// * `Ok(None)` - Key was not found
    /// * `Err(Self::Error)` - Storage or deserialization error occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::storage::backends::StorageBackend;
    ///
    /// // Simple mock storage for demonstration
    /// #[derive(Debug)]
    /// struct MockStorage;
    ///
    /// #[derive(Debug)]
    /// struct MockError;
    ///
    /// impl std::fmt::Display for MockError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "Mock storage error")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MockError {}
    ///
    /// impl StorageBackend for MockStorage {
    ///     type Error = MockError;
    ///     
    ///     fn get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, Self::Error> {
    ///         Ok(None)
    ///     }
    ///     
    ///     fn set<T: serde::Serialize>(&mut self, _key: String, _value: T) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    ///     
    ///     fn remove<T: serde::de::DeserializeOwned>(&mut self, _key: &str) -> Result<Option<T>, Self::Error> {
    ///         Ok(None)
    ///     }
    ///     
    ///     fn contains_key(&self, _key: &str) -> Result<bool, Self::Error> {
    ///         Ok(false)
    ///     }
    ///     
    ///     fn keys(&self) -> Result<Vec<String>, Self::Error> {
    ///         Ok(vec![])
    ///     }
    ///     
    ///     fn clear(&mut self) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    ///     
    ///     fn len(&self) -> Result<usize, Self::Error> {
    ///         Ok(0)
    ///     }
    /// }
    ///
    /// let mut storage = MockStorage;
    /// storage.set("temp_data".to_string(), vec![1, 2, 3]).unwrap();
    /// let removed: Option<Vec<i32>> = storage.remove("temp_data").unwrap();
    /// ```
    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error>;

    /// Check if a key exists in storage
    ///
    /// Efficiently checks for key existence without deserializing the value.
    /// This is useful for conditional logic and validation.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Key exists
    /// * `Ok(false)` - Key does not exist
    /// * `Err(Self::Error)` - Storage error occurred
    fn contains_key(&self, key: &str) -> Result<bool, Self::Error>;

    /// Get all keys currently stored
    ///
    /// Returns a list of all keys in the storage. Useful for debugging,
    /// data migration, or implementing complex queries.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of all keys
    /// * `Err(Self::Error)` - Storage error occurred
    ///
    /// # Performance
    ///
    /// Performance varies by backend. Memory storage is O(n) where n is the
    /// number of keys. File storage may need to parse the entire file.
    fn keys(&self) -> Result<Vec<String>, Self::Error>;

    /// Clear all data from storage
    ///
    /// Removes all stored key-value pairs. This operation is typically
    /// irreversible, so use with caution.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All data successfully cleared
    /// * `Err(Self::Error)` - Storage error occurred
    fn clear(&mut self) -> Result<(), Self::Error>;

    /// Get the number of stored items
    ///
    /// Returns the total count of key-value pairs in storage.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of stored items
    /// * `Err(Self::Error)` - Storage error occurred
    fn len(&self) -> Result<usize, Self::Error>;

    /// Check if the storage is empty
    ///
    /// Convenience method that checks if the storage contains no items.
    /// Has a default implementation based on `len()`.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Storage is empty
    /// * `Ok(false)` - Storage contains items
    /// * `Err(Self::Error)` - Storage error occurred
    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.len()? == 0)
    }
}

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
