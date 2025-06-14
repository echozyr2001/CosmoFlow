use crate::shared_store::new_design::SharedStore;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// High-performance in-memory storage backend using HashMap
///
/// `MemoryStorage` provides extremely fast data access by storing all data in memory
/// using a HashMap. This makes it ideal for development, testing, and scenarios where
/// persistence is not required but performance is critical.
///
/// # Features
///
/// - **Zero I/O overhead**: All operations happen in memory
/// - **Type safety**: Generic interface with automatic JSON serialization
/// - **Thread-safe for reads**: Multiple readers can access data safely
/// - **Configurable capacity**: Can pre-allocate HashMap capacity for performance
/// - **Clone support**: Efficient cloning for workflow distribution
///
/// # Performance Characteristics
///
/// - **Read operations**: O(1) average case
/// - **Write operations**: O(1) average case  
/// - **Memory usage**: Proportional to stored data size
/// - **Startup time**: Instant (no file loading)
///
/// # Use Cases
///
/// ## Development and Testing
/// ```rust
/// use cosmoflow::storage::MemoryStorage;
/// use cosmoflow::SharedStore;
///
/// let mut storage = MemoryStorage::new();
/// storage.set("test_key".to_string(), "test_value").unwrap();
///
/// let value: Option<String> = storage.get("test_key").unwrap();
/// assert_eq!(value, Some("test_value".to_string()));
/// ```
///
/// ## High-Performance Workflows
/// ```rust
/// use cosmoflow::storage::MemoryStorage;
/// use cosmoflow::SharedStore;
/// use serde_json::json;
///
/// // Pre-allocate for better performance
/// let mut storage = MemoryStorage::with_capacity(1000);
///
/// // Store complex data structures efficiently
/// storage.set("metrics".to_string(), json!({
///     "cpu_usage": 45.2,
///     "memory_usage": 67.8,
///     "active_connections": 1250
/// })).unwrap();
/// ```
///
/// ## Temporary Data Processing
/// ```rust
/// use cosmoflow::storage::{MemoryStorage, SharedStore};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct ProcessingState {
///     stage: String,
///     progress: f64,
///     intermediate_results: Vec<i32>,
/// }
///
/// let mut storage = MemoryStorage::new();
/// let state = ProcessingState {
///     stage: "data_analysis".to_string(),
///     progress: 0.75,
///     intermediate_results: vec![1, 2, 3, 4, 5],
/// };
///
/// storage.set("current_state".to_string(), state).unwrap();
/// ```
///
/// # Limitations
///
/// - **No persistence**: Data is lost when the process ends
/// - **Memory constraints**: Limited by available system memory
/// - **Single process**: Cannot share data between different processes
/// - **No crash recovery**: No automatic data recovery mechanisms
///
/// # Thread Safety
///
/// While the struct implements `Send + Sync`, the actual HashMap operations
/// are not thread-safe for mutations. Use external synchronization if you
/// need concurrent write access from multiple threads.
#[derive(Debug, Clone, Default)]
pub struct MemoryStorage {
    data: HashMap<String, Value>,
}

/// Comprehensive error type for in-memory storage operations
///
/// `MemoryStorageError` captures the various failure modes specific to in-memory
/// storage operations. Since memory storage doesn't involve I/O, errors are
/// primarily related to data serialization and deserialization.
///
/// # Error Variants
///
/// ## SerializationError
/// Occurs when converting Rust types to JSON fails. This can happen with:
/// - Non-serializable types (those not implementing `Serialize`)
/// - Circular references in data structures
/// - Custom serialization logic that fails
///
/// ## DeserializationError
/// Occurs when converting stored JSON back to Rust types fails. Common causes:
/// - Type mismatches (storing as one type, retrieving as another)
/// - Invalid JSON structure for the target type
/// - Missing required fields in structs
/// - Enum variant mismatches
///
/// # Examples
///
/// ## Handling Serialization Errors
/// ```rust
/// use cosmoflow::storage::{MemoryStorage, MemoryStorageError, SharedStore};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct MyType {
///     value: i32,
/// }
///
/// let mut storage = MemoryStorage::new();
/// let my_value = MyType { value: 42 };
///
/// // This would fail if MyType doesn't implement Serialize
/// match storage.set("key".to_string(), my_value) {
///     Ok(()) => println!("Stored successfully"),
///     Err(MemoryStorageError::SerializationError(msg)) => {
///         eprintln!("Failed to serialize: {}", msg);
///     },
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// ```
///
/// ## Handling Deserialization Errors
/// ```rust
/// use cosmoflow::storage::{MemoryStorage, MemoryStorageError, SharedStore};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug)]
/// struct MyType {
///     value: i32,
/// }
///
/// let storage = MemoryStorage::new();
///
/// // This would fail if stored data doesn't match expected type
/// match storage.get::<MyType>("key") {
///     Ok(Some(value)) => println!("Retrieved: {:?}", value),
///     Ok(None) => println!("Key not found"),
///     Err(MemoryStorageError::DeserializationError(msg)) => {
///         eprintln!("Failed to deserialize: {}", msg);
///     },
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// ```
#[derive(Debug, Clone, Error)]
pub enum MemoryStorageError {
    /// JSON serialization error
    ///
    /// Indicates that a Rust value could not be converted to JSON.
    /// This typically happens when the type doesn't properly implement
    /// `serde::Serialize` or contains unsupported data structures.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// JSON deserialization error
    ///
    /// Indicates that stored JSON data could not be converted to the
    /// requested Rust type. This commonly occurs with type mismatches
    /// or when the JSON structure doesn't match the target type.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl MemoryStorage {
    /// Create a new empty in-memory storage
    ///
    /// Creates a new `MemoryStorage` instance with default HashMap capacity.
    /// This is the most common way to create memory storage for general use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::storage::MemoryStorage;
    ///
    /// let storage = MemoryStorage::new();
    /// // Ready to store and retrieve data
    /// ```
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create a new in-memory storage with pre-allocated capacity
    ///
    /// Pre-allocates the internal HashMap with the specified capacity to avoid
    /// reallocations during initial data insertion. Use this when you have a
    /// good estimate of how many keys you'll be storing.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Initial capacity for the internal HashMap
    ///
    /// # Performance Benefits
    ///
    /// - Reduces memory allocations during growth
    /// - Improves performance for large datasets
    /// - Prevents HashMap rehashing for known sizes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::storage::MemoryStorage;
    ///
    /// // Pre-allocate for storing 1000 workflow variables
    /// let storage = MemoryStorage::with_capacity(1000);
    ///
    /// // More efficient than using new() for large datasets
    /// let large_storage = MemoryStorage::with_capacity(10_000);
    /// ```
    ///
    /// # When to Use
    ///
    /// - When you know approximate data size in advance
    /// - For performance-critical applications
    /// - When loading large datasets into memory
    /// - For bulk operations with many keys
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }
}

impl SharedStore for MemoryStorage {
    type Error = MemoryStorageError;

    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| MemoryStorageError::SerializationError(e.to_string()))?;
        self.data.insert(key, json_value);
        Ok(())
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.get(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value.clone())
                    .map_err(|e| MemoryStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.remove(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value)
                    .map_err(|e| MemoryStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(key))
    }

    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.data.clear();
        Ok(())
    }

    fn len(&self) -> Result<usize, Self::Error> {
        Ok(self.data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestData {
        id: i32,
        name: String,
    }

    #[test]
    fn test_memory_storage_basic_operations() {
        let mut storage = MemoryStorage::new();

        // Test set and get with JSON Value
        storage.set("key1".to_string(), json!("value1")).unwrap();
        let retrieved: Option<Value> = storage.get("key1").unwrap();
        assert_eq!(retrieved, Some(json!("value1")));

        // Test non-existent key
        let nonexistent: Option<Value> = storage.get("nonexistent").unwrap();
        assert_eq!(nonexistent, None);

        // Test contains_key
        assert!(storage.contains_key("key1").unwrap());
        assert!(!storage.contains_key("nonexistent").unwrap());

        // Test len
        assert_eq!(storage.len().unwrap(), 1);
        assert!(!storage.is_empty().unwrap());

        // Test remove
        let removed: Option<Value> = storage.remove("key1").unwrap();
        assert_eq!(removed, Some(json!("value1")));
        let removed_again: Option<Value> = storage.remove("key1").unwrap();
        assert_eq!(removed_again, None);
        assert_eq!(storage.len().unwrap(), 0);
        assert!(storage.is_empty().unwrap());
    }

    #[test]
    fn test_memory_storage_with_structs() {
        let mut storage = MemoryStorage::new();

        let test_data = TestData {
            id: 123,
            name: "Test Item".to_string(),
        };

        // Test set and get with custom struct
        storage.set("struct_key".to_string(), &test_data).unwrap();
        let retrieved: Option<TestData> = storage.get("struct_key").unwrap();
        assert_eq!(retrieved, Some(test_data.clone()));

        // Test remove with struct
        let removed: Option<TestData> = storage.remove("struct_key").unwrap();
        assert_eq!(removed, Some(test_data));
    }

    #[test]
    fn test_memory_storage_keys_and_clear() {
        let mut storage = MemoryStorage::new();

        storage.set("key1".to_string(), json!("value1")).unwrap();
        storage.set("key2".to_string(), json!("value2")).unwrap();
        storage.set("key3".to_string(), json!("value3")).unwrap();

        let mut keys = storage.keys().unwrap();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2", "key3"]);

        storage.clear().unwrap();
        assert_eq!(storage.len().unwrap(), 0);
        assert!(storage.keys().unwrap().is_empty());
    }

    #[test]
    fn test_memory_storage_type_flexibility() {
        let mut storage = MemoryStorage::new();

        // Store different types under different keys
        storage
            .set("string_key".to_string(), "Hello World".to_string())
            .unwrap();
        storage.set("number_key".to_string(), 42i32).unwrap();
        storage.set("bool_key".to_string(), true).unwrap();
        storage
            .set("json_key".to_string(), json!({"nested": "data"}))
            .unwrap();

        // Retrieve with correct types
        let string_val: Option<String> = storage.get("string_key").unwrap();
        assert_eq!(string_val, Some("Hello World".to_string()));

        let number_val: Option<i32> = storage.get("number_key").unwrap();
        assert_eq!(number_val, Some(42));

        let bool_val: Option<bool> = storage.get("bool_key").unwrap();
        assert_eq!(bool_val, Some(true));

        let json_val: Option<Value> = storage.get("json_key").unwrap();
        assert_eq!(json_val, Some(json!({"nested": "data"})));
    }

    #[test]
    fn test_memory_storage_serialization_errors() {
        let mut storage = MemoryStorage::new();

        // Instead, let's test deserialization errors by manually inserting invalid JSON
        storage.data.insert(
            "invalid_key".to_string(),
            serde_json::Value::String("not_a_number".to_string()),
        );

        // Try to deserialize as a number - this should fail
        let result: Result<Option<i32>, MemoryStorageError> = storage.get("invalid_key");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MemoryStorageError::DeserializationError(_)));

        // Test error message formatting
        let error_msg = error.to_string();
        assert!(error_msg.contains("Deserialization error"));
    }

    #[test]
    fn test_memory_storage_error_types() {
        // Test both error types can be created and displayed
        let ser_error =
            MemoryStorageError::SerializationError("test serialization error".to_string());
        let deser_error =
            MemoryStorageError::DeserializationError("test deserialization error".to_string());

        assert!(ser_error.to_string().contains("Serialization error"));
        assert!(deser_error.to_string().contains("Deserialization error"));

        // Test that they implement Error trait
        use std::error::Error;
        assert!(ser_error.source().is_none()); // Our errors don't have sources
        assert!(deser_error.source().is_none());
    }

    #[test]
    fn test_memory_storage_remove_deserialization_error() {
        let mut storage = MemoryStorage::new();

        // Manually insert invalid data for the expected type
        storage.data.insert(
            "bad_data".to_string(),
            serde_json::Value::String("not_a_struct".to_string()),
        );

        // Try to remove and deserialize as TestData - this should fail
        let result: Result<Option<TestData>, MemoryStorageError> = storage.remove("bad_data");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, MemoryStorageError::DeserializationError(_)));
        assert!(error.to_string().contains("Deserialization error"));
    }

    #[test]
    fn test_memory_storage_error_cloning() {
        let original_error = MemoryStorageError::SerializationError("test".to_string());
        let cloned_error = original_error.clone();

        // Both should have the same message
        assert_eq!(original_error.to_string(), cloned_error.to_string());
    }
}
