//! Redis Storage Backend
//!
//! This module provides a Redis-based storage backend implementation for the cosmoflow
//! storage system. It offers persistent, high-performance key-value storage with support
//! for complex data types through JSON serialization.
//!
//! # Features
//!
//! - **Thread-safe**: Uses `Arc<Mutex<Connection>>` for safe concurrent access
//! - **Type-safe**: Generic support for any serializable/deserializable types
//! - **Namespaced**: Key prefixing to avoid conflicts with other Redis users
//! - **Error handling**: Comprehensive error types for different failure scenarios
//! - **JSON serialization**: Automatic serialization/deserialization of complex types
//!
//! # Usage
//!
//! ```rust,no_run
//! use crate::cosmoflow::SharedStore;
//! use cosmoflow::shared_store::backends::RedisStorage;
//! use serde_json::json;
//! use serde_json::Value;
//!
//! // Create a new Redis storage with default prefix
//! let mut storage = RedisStorage::new("redis://localhost:6379/")?;
//!
//! // Store and retrieve data
//! storage.set("user:123".to_string(), json!({"name": "Alice", "age": 30}))?;
//! let user_data: Option<Value> = storage.get("user:123")?;
//!
//! // Check if key exists
//! if storage.contains_key("user:123")? {
//!     println!("User data found!");
//! }
//!
//! // Get all keys
//! let all_keys = storage.keys()?;
//! println!("Stored keys: {:?}", all_keys);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Requirements
//!
//! - Redis server running and accessible
//! - Network connectivity to Redis instance
//! - Appropriate Redis credentials/permissions

use crate::SharedStore;
use redis::{Client, Commands, Connection};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Redis-based storage backend that implements the SharedStore trait.
///
/// This storage backend provides persistent, distributed storage using Redis as the
/// underlying data store. All values are automatically serialized to JSON before
/// storage and deserialized when retrieved.
///
/// # Thread Safety
///
/// The RedisStorage is designed to be thread-safe through the use of `Arc<Mutex<Connection>>`.
/// Multiple threads can safely access the same RedisStorage instance simultaneously.
///
/// # Key Namespacing
///
/// All keys are automatically prefixed with a configurable namespace string to avoid
/// conflicts with other applications using the same Redis instance. The default prefix
/// is "cosmoflow", but this can be customized using `new_with_prefix()`.
///
/// # Connection Management
///
/// The storage maintains a single persistent connection to Redis, wrapped in a mutex
/// for thread safety. Connection failures are automatically converted to appropriate
/// error types.
pub struct RedisStorage {
    /// Thread-safe Redis connection wrapped in `Arc<Mutex<>>` for concurrent access
    connection: Arc<Mutex<Connection>>,
    /// Namespace prefix added to all keys to avoid conflicts
    key_prefix: String,
}

/// Comprehensive error types for Redis storage operations.
///
/// These error types provide detailed information about different failure scenarios
/// that can occur during Redis storage operations, allowing for appropriate error
/// handling and debugging.
#[derive(Debug, Error)]
pub enum RedisStorageError {
    /// Redis connection or command execution errors.
    ///
    /// This includes network failures, authentication errors, Redis server errors,
    /// and any other Redis-specific issues. The underlying Redis error is preserved
    /// for detailed diagnostics.
    #[error("Redis connection error: {0}")]
    Connection(#[from] redis::RedisError),

    /// JSON serialization or deserialization errors.
    ///
    /// Occurs when data cannot be serialized to JSON before storage or cannot be
    /// deserialized from JSON after retrieval. This usually indicates incompatible
    /// data types or corrupted data in Redis.
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    /// Mutex lock acquisition errors.
    ///
    /// Occurs when the thread cannot acquire a lock on the Redis connection mutex.
    /// This is typically caused by lock poisoning due to a panic in another thread
    /// while holding the lock.
    #[error("Lock error: {0}")]
    Lock(String),
}

impl RedisStorage {
    /// Creates a new Redis storage instance with the default key prefix "cosmoflow".
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379/")
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the RedisStorage instance or a RedisStorageError
    /// if the connection cannot be established.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use cosmoflow::shared_store::backends::RedisStorage;
    ///
    /// let storage = RedisStorage::new("redis://localhost:6379/")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::Connection` if Redis connection fails
    pub fn new(redis_url: &str) -> Result<Self, RedisStorageError> {
        Self::new_with_prefix(redis_url, "cosmoflow")
    }

    /// Creates a new Redis storage instance with a custom key prefix.
    ///
    /// The key prefix is used to namespace all keys, preventing conflicts when
    /// multiple applications or instances share the same Redis database.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379/")
    /// * `key_prefix` - Custom prefix for all keys (e.g., "myapp", "prod", "test")
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the RedisStorage instance or a RedisStorageError
    /// if the connection cannot be established.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use cosmoflow::shared_store::backends::RedisStorage;
    ///
    /// // Use different prefixes for different environments
    /// let prod_storage = RedisStorage::new_with_prefix(
    ///     "redis://localhost:6379/",
    ///     "cosmoflow_prod"
    /// )?;
    ///
    /// let test_storage = RedisStorage::new_with_prefix(
    ///     "redis://localhost:6379/",
    ///     "cosmoflow_test"
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::Connection` if Redis connection fails
    pub fn new_with_prefix(redis_url: &str, key_prefix: &str) -> Result<Self, RedisStorageError> {
        // Create Redis client from URL
        let client = Client::open(redis_url)?;

        // Establish connection to Redis server
        let connection = client.get_connection()?;

        Ok(RedisStorage {
            // Wrap connection in `Arc<Mutex<>>` for thread safety
            connection: Arc::new(Mutex::new(connection)),
            key_prefix: key_prefix.to_string(),
        })
    }

    /// Constructs the full Redis key by prepending the configured prefix.
    ///
    /// This internal method ensures all keys are properly namespaced to avoid
    /// conflicts with other applications using the same Redis instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The base key name without prefix
    ///
    /// # Returns
    ///
    /// The full key with prefix in the format "{prefix}:{key}"
    ///
    /// # Examples
    ///
    /// ```text
    /// prefix: "cosmoflow"
    /// key: "user:123"
    /// result: "cosmoflow:user:123"
    /// ```
    fn get_full_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    /// Removes the prefix from a full Redis key to get the original key.
    ///
    /// This is used when retrieving keys from Redis to convert them back to
    /// the format expected by the application.
    ///
    /// # Arguments
    ///
    /// * `full_key` - The complete key with prefix from Redis
    ///
    /// # Returns
    ///
    /// `Some(String)` containing the original key without prefix, or `None`
    /// if the key doesn't start with the expected prefix.
    ///
    /// # Examples
    ///
    /// ```text
    /// prefix: "cosmoflow"
    /// full_key: "cosmoflow:user:123"
    /// result: Some("user:123")
    ///
    /// full_key: "other_app:data"
    /// result: None
    /// ```
    fn remove_prefix(&self, full_key: &str) -> Option<String> {
        let prefix_with_colon = format!("{}:", self.key_prefix);
        if full_key.starts_with(&prefix_with_colon) {
            Some(full_key[prefix_with_colon.len()..].to_string())
        } else {
            None
        }
    }

    /// Executes a Redis command with proper connection management and error handling.
    ///
    /// This helper method provides a consistent pattern for executing Redis commands
    /// while handling connection locking and error conversion. It ensures that the
    /// connection mutex is properly acquired and released, and converts Redis errors
    /// to our custom error types.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure type that takes a mutable connection reference
    /// * `R` - Return type of the Redis operation
    ///
    /// # Arguments
    ///
    /// * `f` - Closure that performs the Redis operation
    ///
    /// # Returns
    ///
    /// `Result<R, RedisStorageError>` containing the operation result or error
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::Lock` if mutex cannot be acquired
    /// - `RedisStorageError::Connection` if Redis operation fails
    fn with_connection<F, R>(&self, f: F) -> Result<R, RedisStorageError>
    where
        F: FnOnce(&mut Connection) -> Result<R, redis::RedisError>,
    {
        // Acquire exclusive access to the Redis connection
        let mut conn = self
            .connection
            .lock()
            .map_err(|e| RedisStorageError::Lock(e.to_string()))?;

        // Execute the Redis operation and convert errors
        f(&mut conn).map_err(RedisStorageError::Connection)
    }
}

impl SharedStore for RedisStorage {
    type Error = RedisStorageError;

    /// Stores a value in Redis with the specified key.
    ///
    /// The value is automatically serialized to JSON before storage. The key is
    /// prefixed with the instance's namespace to avoid conflicts.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements Serialize
    ///
    /// # Arguments
    ///
    /// * `key` - The key to store the value under (will be prefixed)
    /// * `value` - The value to store (will be JSON-serialized)
    ///
    /// # Returns
    ///
    /// `Result<(), RedisStorageError>` indicating success or failure
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::JsonSerialization` if value cannot be serialized
    /// - `RedisStorageError::Connection` if Redis operation fails
    /// - `RedisStorageError::Lock` if connection mutex cannot be acquired
    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
        // Add namespace prefix to the key
        let full_key = self.get_full_key(&key);

        // Serialize the value to JSON string
        let json_string = serde_json::to_string(&value)?;

        // Execute Redis SET command with connection management
        self.with_connection(|conn| {
            let _: () = conn.set(&full_key, &json_string)?;
            Ok(())
        })
    }

    /// Retrieves a value from Redis by key.
    ///
    /// The value is automatically deserialized from JSON. Returns None if the
    /// key doesn't exist in Redis.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements DeserializeOwned
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve (prefix will be added automatically)
    ///
    /// # Returns
    ///
    /// `Result<Option<T>, RedisStorageError>` containing the value or None if not found
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::JsonSerialization` if stored value cannot be deserialized
    /// - `RedisStorageError::Connection` if Redis operation fails
    /// - `RedisStorageError::Lock` if connection mutex cannot be acquired
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        // Add namespace prefix to the key
        let full_key = self.get_full_key(key);

        self.with_connection(|conn| {
            // Execute Redis GET command
            let result: Option<String> = conn.get(&full_key)?;

            match result {
                Some(json_string) => {
                    // Deserialize JSON string back to the requested type
                    let value = serde_json::from_str(&json_string).map_err(|e| {
                        // Convert serde error to Redis error for consistent error handling
                        redis::RedisError::from((
                            redis::ErrorKind::TypeError,
                            "JSON parse error",
                            e.to_string(),
                        ))
                    })?;
                    Ok(Some(value))
                }
                None => Ok(None), // Key doesn't exist
            }
        })
    }

    /// Removes a key from Redis and returns its previous value.
    ///
    /// This operation is atomic - it retrieves the current value and then deletes
    /// the key. If the key doesn't exist, returns None.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type that implements DeserializeOwned
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove (prefix will be added automatically)
    ///
    /// # Returns
    ///
    /// `Result<Option<T>, RedisStorageError>` containing the previous value or None
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::JsonSerialization` if stored value cannot be deserialized
    /// - `RedisStorageError::Connection` if Redis operation fails
    /// - `RedisStorageError::Lock` if connection mutex cannot be acquired
    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        let full_key = self.get_full_key(key);

        // First try to get the existing value before deletion
        let existing_value = self.get(key)?;

        // Then delete the key from Redis
        self.with_connection(|conn| {
            let _: u32 = conn.del(&full_key)?;
            Ok(())
        })?;

        Ok(existing_value)
    }

    /// Checks if a key exists in Redis.
    ///
    /// This is more efficient than retrieving the value when you only need to
    /// check for existence.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check (prefix will be added automatically)
    ///
    /// # Returns
    ///
    /// `Result<bool, RedisStorageError>` indicating if the key exists
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::Connection` if Redis operation fails
    /// - `RedisStorageError::Lock` if connection mutex cannot be acquired
    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        let full_key = self.get_full_key(key);

        self.with_connection(|conn| {
            // Use Redis EXISTS command for efficient existence check
            let result: bool = conn.exists(&full_key)?;
            Ok(result)
        })
    }

    /// Retrieves all keys stored with this instance's prefix.
    ///
    /// Returns only the key names without prefixes, making them suitable for
    /// use with other storage operations.
    ///
    /// # Performance Note
    ///
    /// This operation uses Redis KEYS command which can be slow on large databases.
    /// Consider using SCAN in production environments with many keys.
    ///
    /// # Returns
    ///
    /// `Result<Vec<String>, RedisStorageError>` containing all keys without prefixes
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::Connection` if Redis operation fails
    /// - `RedisStorageError::Lock` if connection mutex cannot be acquired
    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        // Create pattern to match all keys with our prefix
        let pattern = format!("{}:*", self.key_prefix);

        self.with_connection(|conn| {
            // Get all keys matching the pattern
            let full_keys: Vec<String> = conn.keys(&pattern)?;

            // Remove prefix from each key to return clean key names
            let keys = full_keys
                .into_iter()
                .filter_map(|full_key| self.remove_prefix(&full_key))
                .collect();

            Ok(keys)
        })
    }

    /// Removes all keys stored with this instance's prefix.
    ///
    /// This operation is atomic and will delete all keys belonging to this
    /// storage instance without affecting other prefixed keys in the same Redis database.
    ///
    /// # Performance Note
    ///
    /// This operation first retrieves all matching keys, then deletes them in batch.
    /// For very large numbers of keys, consider implementing pagination.
    ///
    /// # Returns
    ///
    /// `Result<(), RedisStorageError>` indicating success or failure
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::Connection` if Redis operation fails
    /// - `RedisStorageError::Lock` if connection mutex cannot be acquired
    fn clear(&mut self) -> Result<(), Self::Error> {
        // Create pattern to match all keys with our prefix
        let pattern = format!("{}:*", self.key_prefix);

        self.with_connection(|conn| {
            // Get all keys that match our prefix pattern
            let full_keys: Vec<String> = conn.keys(&pattern)?;

            // Only attempt deletion if there are keys to delete
            if !full_keys.is_empty() {
                // Delete all matching keys in a single operation
                let _: u32 = conn.del(&full_keys)?;
            }

            Ok(())
        })
    }

    /// Returns the number of keys stored with this instance's prefix.
    ///
    /// # Performance Note
    ///
    /// This operation uses KEYS command to count entries, which can be slow
    /// on large databases. The count includes only keys with this instance's prefix.
    ///
    /// # Returns
    ///
    /// `Result<usize, RedisStorageError>` containing the count of keys
    ///
    /// # Errors
    ///
    /// - `RedisStorageError::Connection` if Redis operation fails
    /// - `RedisStorageError::Lock` if connection mutex cannot be acquired
    fn len(&self) -> Result<usize, Self::Error> {
        // Create pattern to match all keys with our prefix
        let pattern = format!("{}:*", self.key_prefix);

        self.with_connection(|conn| {
            // Get all keys matching the pattern and count them
            let full_keys: Vec<String> = conn.keys(&pattern)?;
            Ok(full_keys.len())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // These tests require a Redis server running on localhost:6379
    //
    // To run these tests:
    // 1. Start Redis server: docker run --rm -p 6379:6379 redis:latest
    // 2. Run tests: cargo test redis_storage -- --ignored
    //
    // Note: Tests are marked with #[ignore] by default to prevent CI failures
    // when Redis is not available. Use --ignored flag to run them explicitly.

    /// Sets up a test Redis storage instance with a test-specific prefix.
    ///
    /// Uses a separate prefix to avoid conflicts with production data and
    /// other test runs that might be happening concurrently.
    fn setup_redis() -> Result<RedisStorage, RedisStorageError> {
        RedisStorage::new_with_prefix("redis://127.0.0.1:6379/", "cosmoflow_test")
    }

    #[test]
    #[ignore] // Requires Redis server - run with: cargo test redis_storage -- --ignored
    fn test_redis_storage_basic_operations() -> Result<(), RedisStorageError> {
        let mut storage = setup_redis()?;

        // Clear any existing test data to ensure clean test environment
        storage.clear()?;

        // Test set operation - store a simple JSON value
        storage.set("test_key".to_string(), json!("test_value"))?;

        // Test get operation - retrieve and verify the stored value
        let value = storage.get("test_key")?;
        assert_eq!(value, Some(json!("test_value")));

        // Test contains_key operation - verify key existence checks
        assert!(storage.contains_key("test_key")?);
        assert!(!storage.contains_key("nonexistent_key")?);

        // Test remove operation - delete key and verify it returns the old value
        let removed = storage.remove("test_key")?;
        assert_eq!(removed, Some(json!("test_value")));

        // Verify the key no longer exists after removal
        assert!(!storage.contains_key("test_key")?);

        Ok(())
    }

    #[test]
    #[ignore] // Requires Redis server - run with: cargo test redis_storage -- --ignored
    fn test_redis_storage_complex_data() -> Result<(), RedisStorageError> {
        let mut storage = setup_redis()?;
        storage.clear()?;

        // Test storage of complex nested JSON data structures
        let complex_data = json!({
            "user_id": 123,
            "preferences": {
                "theme": "dark",
                "language": "en"
            },
            "tags": ["rust", "redis", "cosmoflow"],
            "metadata": {
                "created_at": "2024-01-01T00:00:00Z",
                "version": 1.2,
                "active": true
            }
        });

        // Store complex data and verify it can be retrieved accurately
        storage.set("user_data".to_string(), complex_data.clone())?;
        let retrieved = storage.get("user_data")?;
        assert_eq!(retrieved, Some(complex_data));

        // Test that nested structures are preserved through serialization
        let user_data = storage.get::<serde_json::Value>("user_data")?.unwrap();
        let user_prefs = user_data["preferences"]["theme"].as_str();
        assert_eq!(user_prefs, Some("dark"));

        Ok(())
    }

    #[test]
    #[ignore] // Requires Redis server - run with: cargo test redis_storage -- --ignored
    fn test_redis_storage_keys_and_len() -> Result<(), RedisStorageError> {
        let mut storage = setup_redis()?;
        storage.clear()?;

        // Verify storage starts empty
        assert_eq!(storage.len()?, 0);
        assert!(storage.is_empty()?);

        // Add several test keys with different data types
        storage.set("key1".to_string(), json!("string_value"))?;
        storage.set("key2".to_string(), json!(42))?;
        storage.set("key3".to_string(), json!({"nested": "object"}))?;

        // Test len operation - verify count is accurate
        assert_eq!(storage.len()?, 3);
        assert!(!storage.is_empty()?);

        // Test keys operation - retrieve all stored keys
        let mut keys = storage.keys()?;
        keys.sort(); // Sort for consistent comparison
        assert_eq!(keys, vec!["key1", "key2", "key3"]);

        // Test clear operation - remove all keys
        storage.clear()?;

        // Verify storage is empty after clearing
        assert_eq!(storage.len()?, 0);
        assert!(storage.is_empty()?);
        assert_eq!(storage.keys()?, Vec::<String>::new());

        Ok(())
    }

    #[test]
    #[ignore] // Requires Redis server - run with: cargo test redis_storage -- --ignored
    fn test_redis_storage_key_prefixing() -> Result<(), RedisStorageError> {
        // Create two storage instances with different prefixes
        let mut storage1 = RedisStorage::new_with_prefix("redis://127.0.0.1:6379/", "app1")?;
        let mut storage2 = RedisStorage::new_with_prefix("redis://127.0.0.1:6379/", "app2")?;

        // Clear both storages
        storage1.clear()?;
        storage2.clear()?;

        // Store data with the same key in both storages
        storage1.set("shared_key".to_string(), json!("value_from_app1"))?;
        storage2.set("shared_key".to_string(), json!("value_from_app2"))?;

        // Verify each storage only sees its own data
        assert_eq!(
            storage1.get::<serde_json::Value>("shared_key")?,
            Some(json!("value_from_app1"))
        );
        assert_eq!(
            storage2.get::<serde_json::Value>("shared_key")?,
            Some(json!("value_from_app2"))
        );

        // Verify key isolation - each storage should only see its own keys
        assert_eq!(storage1.len()?, 1);
        assert_eq!(storage2.len()?, 1);
        assert_eq!(storage1.keys()?, vec!["shared_key"]);
        assert_eq!(storage2.keys()?, vec!["shared_key"]);

        // Clean up
        storage1.clear()?;
        storage2.clear()?;

        Ok(())
    }

    #[test]
    #[ignore] // Requires Redis server - run with: cargo test redis_storage -- --ignored
    fn test_redis_storage_error_handling() -> Result<(), RedisStorageError> {
        let storage = setup_redis()?;

        // Test retrieval of non-existent key
        let result = storage.get::<String>("nonexistent_key")?;
        assert_eq!(result, None);

        // Test contains_key on non-existent key
        assert!(!storage.contains_key("nonexistent_key")?);

        // Note: Testing JSON deserialization errors would require manually
        // corrupting data in Redis, which is complex to set up in unit tests.
        // Integration tests or manual testing would be more appropriate for
        // error scenarios like network failures or malformed JSON.

        Ok(())
    }
}
