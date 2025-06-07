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
//! The [`SharedStore`] is the primary interface for workflow data operations. It provides
//! only two essential methods: `set()` for storing data and `get()` for retrieving data.
//! All data is automatically serialized/deserialized using Serde, providing type safety
//! and supporting any type that implements the required traits.
//!
//! ## Storage Backends
//!
//! The shared store is backend-agnostic, working with any type that implements
//! [`storage::StorageBackend`]. This allows you to choose the appropriate storage
//! solution for your use case:
//!
//! - **Memory Storage**: Fast, temporary storage for development and testing
//! - **File Storage**: Persistent JSON-based storage for small to medium datasets
//! - **Custom Backends**: Implement your own for databases, cloud storage, etc.
//!
//! # Type Safety
//!
//! The shared store provides compile-time type safety through Rust's type system:
//!
//! ```rust
//! use cosmoflow::shared_store::SharedStore;
//! use serde::{Serialize, Deserialize};
//!
//! // Simple mock storage for demonstration
//! #[derive(Debug)]
//! struct MockStorage;
//!
//! #[derive(Debug)]
//! struct MockError;
//!
//! impl std::fmt::Display for MockError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "Mock storage error")
//!     }
//! }
//!
//! impl std::error::Error for MockError {}
//!
//! impl cosmoflow::storage::StorageBackend for MockStorage {
//!     type Error = MockError;
//!     
//!     fn get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, Self::Error> {
//!         Ok(None)
//!     }
//!     
//!     fn set<T: Serialize>(&mut self, _key: String, _value: T) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     
//!     fn remove<T: serde::de::DeserializeOwned>(&mut self, _key: &str) -> Result<Option<T>, Self::Error> {
//!         Ok(None)
//!     }
//!     
//!     fn contains_key(&self, _key: &str) -> Result<bool, Self::Error> {
//!         Ok(false)
//!     }
//!     
//!     fn keys(&self) -> Result<Vec<String>, Self::Error> {
//!         Ok(vec![])
//!     }
//!     
//!     fn clear(&mut self) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     
//!     fn len(&self) -> Result<usize, Self::Error> {
//!         Ok(0)
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct UserData {
//!     id: u64,
//!     name: String,
//!     active: bool,
//! }
//!
//! let storage = MockStorage;
//! let mut store = SharedStore::with_storage(storage);
//!
//! // Type-safe storage
//! let user = UserData {
//!     id: 42,
//!     name: "Alice".to_string(),
//!     active: true,
//! };
//! store.set("user".to_string(), user).unwrap();
//! ```
//!
//! # Data Types
//!
//! The shared store supports any type that implements:
//! - [`serde::Serialize`] for storage operations
//! - [`serde::de::DeserializeOwned`] for retrieval operations
//!
//! ## Supported Types
//!
//! - **Primitives**: `bool`, `i32`, `u64`, `f64`, `String`, etc.
//! - **Collections**: `Vec<T>`, `HashMap<K, V>`, `BTreeMap<K, V>`, etc.
//! - **Optional Values**: `Option<T>`
//! - **Custom Structures**: Any struct/enum with `#[derive(Serialize, Deserialize)]`
//! - **JSON Values**: `serde_json::Value` for dynamic data
//!
//! ```rust
//! use cosmoflow::shared_store::SharedStore;
//! use std::collections::HashMap;
//!
//! // Simple mock storage for demonstration
//! #[derive(Debug)]
//! struct MockStorage;
//!
//! #[derive(Debug)]
//! struct MockError;
//!
//! impl std::fmt::Display for MockError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "Mock storage error")
//!     }
//! }
//!
//! impl std::error::Error for MockError {}
//!
//! impl cosmoflow::storage::StorageBackend for MockStorage {
//!     type Error = MockError;
//!     
//!     fn get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, Self::Error> {
//!         Ok(None)
//!     }
//!     
//!     fn set<T: serde::Serialize>(&mut self, _key: String, _value: T) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     
//!     fn remove<T: serde::de::DeserializeOwned>(&mut self, _key: &str) -> Result<Option<T>, Self::Error> {
//!         Ok(None)
//!     }
//!     
//!     fn contains_key(&self, _key: &str) -> Result<bool, Self::Error> {
//!         Ok(false)
//!     }
//!     
//!     fn keys(&self) -> Result<Vec<String>, Self::Error> {
//!         Ok(vec![])
//!     }
//!     
//!     fn clear(&mut self) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     
//!     fn len(&self) -> Result<usize, Self::Error> {
//!         Ok(0)
//!     }
//! }
//!
//! let storage = MockStorage;
//! let mut store = SharedStore::with_storage(storage);
//!
//! // Store various types
//! store.set("count".to_string(), 42u32).unwrap();
//! store.set("message".to_string(), "Hello, World!".to_string()).unwrap();
//! store.set("tags".to_string(), vec!["rust".to_string(), "workflow".to_string()]).unwrap();
//! ```
//!
//! # Error Handling
//!
//! The shared store propagates errors from the underlying storage backend. Common
//! error scenarios include:
//!
//! - **Serialization Errors**: Invalid data that can't be serialized
//! - **Deserialization Errors**: Type mismatches or corrupted data
//! - **Storage Errors**: Backend-specific issues (disk full, network errors, etc.)
//! - **Key Not Found**: Attempting to retrieve non-existent keys
//!
//! ```rust
//! use cosmoflow::shared_store::SharedStore;
//!
//! // Simple mock storage for demonstration
//! #[derive(Debug)]
//! struct MockStorage;
//!
//! #[derive(Debug)]
//! struct MockError;
//!
//! impl std::fmt::Display for MockError {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         write!(f, "Mock storage error")
//!     }
//! }
//!
//! impl std::error::Error for MockError {}
//!
//! impl cosmoflow::storage::StorageBackend for MockStorage {
//!     type Error = MockError;
//!     
//!     fn get<T: serde::de::DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, Self::Error> {
//!         Ok(None)
//!     }
//!     
//!     fn set<T: serde::Serialize>(&mut self, _key: String, _value: T) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     
//!     fn remove<T: serde::de::DeserializeOwned>(&mut self, _key: &str) -> Result<Option<T>, Self::Error> {
//!         Ok(None)
//!     }
//!     
//!     fn contains_key(&self, _key: &str) -> Result<bool, Self::Error> {
//!         Ok(false)
//!     }
//!     
//!     fn keys(&self) -> Result<Vec<String>, Self::Error> {
//!         Ok(vec![])
//!     }
//!     
//!     fn clear(&mut self) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     
//!     fn len(&self) -> Result<usize, Self::Error> {
//!         Ok(0)
//!     }
//! }
//!
//! let storage = MockStorage;
//! let mut store = SharedStore::with_storage(storage);
//!
//! // Handle errors gracefully
//! match store.get::<String>("nonexistent_key") {
//!     Ok(Some(value)) => println!("Value: {}", value),
//!     Ok(None) => println!("Key not found"),
//!     Err(e) => eprintln!("Failed to retrieve value: {}", e),
//! }
//! ```
//!
//! # Integration with CosmoFlow
//!
//! In CosmoFlow workflows, the shared store is automatically managed by the flow
//! execution engine and provides a type-safe interface for data communication
//! between nodes.
//!
//! # Performance Considerations
//!
//! ## Serialization Overhead
//!
//! The shared store serializes all data through Serde. For maximum performance:
//! - Use efficient serialization formats (consider `bincode` for custom backends)
//! - Minimize large data copies between nodes
//! - Consider streaming for large datasets
//!
//! ## Memory Usage
//!
//! The memory usage depends on the chosen storage backend:
//! - **Memory Storage**: Keeps all data in RAM
//! - **File Storage**: Loads/saves data on demand
//! - **Custom Backends**: Depends on implementation
//!
//! ## Concurrency
//!
//! The shared store is designed for CosmoFlow's single-threaded execution model.
//! Each workflow instance has its own store, avoiding concurrency issues while
//! allowing multiple workflows to run in parallel.
//!
//! # Best Practices
//!
//! 1. **Use Descriptive Keys**: Choose clear, consistent naming conventions
//! 2. **Define Data Contracts**: Use well-defined types for node communication
//! 3. **Handle Errors**: Always handle potential serialization/storage errors
//! 4. **Minimize Data**: Only store necessary data to reduce overhead
//! 5. **Version Your Types**: Consider schema evolution for long-term storage
//!
//! ```rust
//! // Good: Clear data contracts
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct ProcessingConfig {
//!     threshold: f64,
//!     algorithm: String,
//!     max_iterations: usize,
//! }
//!
//! #[derive(serde::Serialize, serde::Deserialize)]
//! struct ProcessingResult {
//!     success: bool,
//!     iterations_used: usize,
//!     final_value: f64,
//!     metadata: std::collections::HashMap<String, String>, // Simplified for doc test
//! }
//! ```

use crate::storage::StorageBackend;
use serde::{Serialize, de::DeserializeOwned};

/// SharedStore provides a clean, type-safe interface for data communication between nodes
/// in CosmoFlow workflows. It can use different storage backends for flexibility.
///
/// The store provides only two methods: set() and get(), which work with any
/// serializable/deserializable types automatically.
#[derive(Debug)]
pub struct SharedStore<S: StorageBackend> {
    storage: S,
}

impl<S: StorageBackend> SharedStore<S> {
    /// Creates a new SharedStore with the provided storage backend
    pub fn with_storage(storage: S) -> Self {
        Self { storage }
    }

    /// Sets a value in the SharedStore.
    ///
    /// Accepts any type that implements `serde::Serialize`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to associate with the value.
    /// * `value` - The value to store (any serializable type).
    pub fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), S::Error> {
        self.storage.set(key, value)
    }

    /// Gets a value from the SharedStore.
    ///
    /// Returns any type that implements `serde::de::DeserializeOwned`.
    /// The caller must specify the expected return type.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result<Option<T>, S::Error>` which is `Ok(Some(T))` if the key exists
    /// and can be deserialized to type T, `Ok(None)` if the key doesn't exist,
    /// or `Err` if there was a storage or deserialization error.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, S::Error> {
        self.storage.get(key)
    }

    /// Removes a value from the SharedStore, returning it if it existed.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to remove.
    ///
    /// # Returns
    ///
    /// A `Result<Option<T>, S::Error>` which is `Ok(Some(T))` if the key existed,
    /// `Ok(None)` if it didn't, or `Err` if there was a storage error.
    pub fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, S::Error> {
        self.storage.remove(key)
    }

    /// Checks if a key exists in the SharedStore.
    pub fn contains_key(&self, key: &str) -> Result<bool, S::Error> {
        self.storage.contains_key(key)
    }

    /// Gets all keys from the SharedStore.
    pub fn keys(&self) -> Result<Vec<String>, S::Error> {
        self.storage.keys()
    }

    /// Clears all data from the SharedStore.
    pub fn clear(&mut self) -> Result<(), S::Error> {
        self.storage.clear()
    }

    /// Gets the number of items in the SharedStore.
    pub fn len(&self) -> Result<usize, S::Error> {
        self.storage.len()
    }

    /// Checks if the SharedStore is empty.
    pub fn is_empty(&self) -> Result<bool, S::Error> {
        self.storage.is_empty()
    }
}
