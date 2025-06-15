#![deny(missing_docs)]
//! Type-safe data communication layer for CosmoFlow workflows
//!
//! The shared store provides a unified interface for data communication between nodes
//! in CosmoFlow workflows. It acts as a bridge between the workflow execution engine
//! and pluggable storage backends, ensuring type safety and consistent data handling.
//!
//! # Core Concepts
//!
//! ## Shared Store
//!
//! The [`SharedStore`] trait is the primary interface for workflow data operations. It provides
//! essential methods for storing and retrieving data. All data is automatically
//! serialized/deserialized using Serde, providing type safety and supporting any type
//! that implements the required traits.
//!
//! # Integration with CosmoFlow
//!
//! In CosmoFlow workflows, the shared store is automatically managed by the flow
//! execution engine and provides a type-safe interface for data communication
//! between nodes.
//!

#[cfg(any(
    feature = "storage-memory",
    feature = "storage-file",
    feature = "storage-redis"
))]
pub mod backends;

use serde::{Serialize, de::DeserializeOwned};
use std::error::Error;

/// Unified SharedStore trait that combines storage backend functionality with shared store interface
///
/// This trait replaces both the old `StorageBackend` trait and `SharedStore` struct, providing
/// a unified interface for type-safe data communication between nodes in CosmoFlow workflows.
///
/// # Design Principles
///
/// - **Type Safety**: All operations work with strongly-typed data through Serde
/// - **Error Handling**: Each implementation defines its own error type for specific failure modes
/// - **Performance**: Operations are designed to be efficient for the target storage medium
/// - **Flexibility**: Supports both synchronous and asynchronous implementations
/// - **Simplicity**: Single trait interface reduces complexity
///
/// # Required Methods
///
/// Implementers must provide all methods except `is_empty()`, which has a default implementation.
///
/// # Error Handling
///
/// Each implementation defines its own error type that must implement:
/// - `std::error::Error` for error trait support
/// - `Send + Sync + 'static` for use in async contexts
///
/// # Examples
///
/// ## Basic Usage Pattern
///
/// ```rust
/// use serde::{Serialize, de::DeserializeOwned};
/// use std::error::Error;
///
/// // Define your storage implementation
/// trait SharedStore: Send + Sync {
///     type Error: Error + Send + Sync + 'static;
///     fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error>;
///     fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;
///     fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error>;
///     fn contains_key(&self, key: &str) -> Result<bool, Self::Error>;
///     fn keys(&self) -> Result<Vec<String>, Self::Error>;
///     fn clear(&mut self) -> Result<(), Self::Error>;
///     fn len(&self) -> Result<usize, Self::Error>;
///     fn is_empty(&self) -> Result<bool, Self::Error> { Ok(self.len()? == 0) }
/// }
/// ```
pub trait SharedStore: Send + Sync {
    /// Error type returned by storage operations
    ///
    /// Each implementation should define a specific error type that
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
    /// Performance varies by implementation. Memory storage is O(n) where n is the
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
