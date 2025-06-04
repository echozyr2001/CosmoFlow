pub mod storage;

pub mod prelude {
    pub use crate::SharedStore;
    pub use crate::storage::StorageBackend;

    #[cfg(feature = "storage-memory")]
    pub use crate::storage::{MemoryStorage, MemoryStorageError};

    #[cfg(feature = "storage-file")]
    pub use crate::storage::{FileStorage, FileStorageError};
}

pub use storage::StorageBackend;

#[cfg(feature = "storage-memory")]
pub use storage::{MemoryStorage, MemoryStorageError};

#[cfg(feature = "storage-file")]
pub use storage::{FileStorage, FileStorageError};

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

#[cfg(feature = "storage-memory")]
impl SharedStore<crate::storage::MemoryStorage> {
    /// Creates a new SharedStore with a memory backend
    pub fn memory() -> Self {
        Self::with_storage(crate::storage::MemoryStorage::new())
    }

    /// Creates a new SharedStore with a memory backend with specified capacity
    pub fn memory_with_capacity(capacity: usize) -> Self {
        Self::with_storage(crate::storage::MemoryStorage::with_capacity(capacity))
    }
}

#[cfg(feature = "storage-file")]
impl SharedStore<crate::storage::FileStorage> {
    /// Creates a new SharedStore with a file backend
    pub fn file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, crate::storage::FileStorageError> {
        Ok(Self::with_storage(crate::storage::FileStorage::new(path)?))
    }
}

#[cfg(all(test, any(feature = "storage-memory", feature = "storage-file")))]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    #[cfg(feature = "storage-file")]
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestStruct {
        id: i32,
        name: String,
        active: bool,
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_convenience_constructors() {
        // Test memory constructor
        let mut store = SharedStore::memory();
        store.set("test_key".to_string(), "test_value").unwrap();
        let result: Option<String> = store.get("test_key").unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        // Test memory with capacity
        let mut store_with_cap = SharedStore::memory_with_capacity(100);
        store_with_cap.set("cap_key".to_string(), 42i32).unwrap();
        let result: Option<i32> = store_with_cap.get("cap_key").unwrap();
        assert_eq!(result, Some(42));

        // Test file constructor if available
        #[cfg(feature = "storage-file")]
        {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("convenience_test.json");

            let mut store = SharedStore::file(&file_path).unwrap();
            store
                .set("file_key".to_string(), json!({"test": "data"}))
                .unwrap();
            let result: Option<Value> = store.get("file_key").unwrap();
            assert_eq!(result, Some(json!({"test": "data"})));
        }
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_set_get_with_json_value() {
        let mut store = SharedStore::memory();
        let key = "json_key".to_string();
        let value = json!({"message": "hello", "count": 42});

        // Test with JSON Value
        store.set(key.clone(), value.clone()).unwrap();
        let retrieved: Option<Value> = store.get(&key).unwrap();
        assert_eq!(retrieved, Some(value));
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_set_get_with_primitive_types() {
        let mut store = SharedStore::memory();

        // Test String
        store
            .set("string_key".to_string(), "Hello World".to_string())
            .unwrap();
        let string_val: Option<String> = store.get("string_key").unwrap();
        assert_eq!(string_val, Some("Hello World".to_string()));

        // Test i32
        store.set("int_key".to_string(), 42i32).unwrap();
        let int_val: Option<i32> = store.get("int_key").unwrap();
        assert_eq!(int_val, Some(42));

        // Test bool
        store.set("bool_key".to_string(), true).unwrap();
        let bool_val: Option<bool> = store.get("bool_key").unwrap();
        assert_eq!(bool_val, Some(true));

        // Test f64
        store.set("float_key".to_string(), 9.11f64).unwrap();
        let float_val: Option<f64> = store.get("float_key").unwrap();
        assert_eq!(float_val, Some(9.11));
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_set_get_with_custom_struct() {
        let mut store = SharedStore::memory();
        let test_data = TestStruct {
            id: 123,
            name: "Test Item".to_string(),
            active: true,
        };

        // Test with custom struct
        store
            .set("struct_key".to_string(), test_data.clone())
            .unwrap();
        let retrieved: Option<TestStruct> = store.get("struct_key").unwrap();
        assert_eq!(retrieved, Some(test_data));
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_get_nonexistent_key() {
        let store = SharedStore::memory();

        // Test with different return types
        let string_result: Option<String> = store.get("nonexistent").unwrap();
        assert_eq!(string_result, None);

        let json_result: Option<Value> = store.get("nonexistent").unwrap();
        assert_eq!(json_result, None);

        let struct_result: Option<TestStruct> = store.get("nonexistent").unwrap();
        assert_eq!(struct_result, None);
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_remove_with_type_specification() {
        let mut store = SharedStore::memory();
        let key = "remove_key".to_string();
        let value = "test_value".to_string();

        store.set(key.clone(), value.clone()).unwrap();

        // Remove and get the value with type specification
        let removed: Option<String> = store.remove(&key).unwrap();
        assert_eq!(removed, Some(value));

        // Verify it's gone
        let after_remove: Option<String> = store.get(&key).unwrap();
        assert_eq!(after_remove, None);
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_utility_methods() {
        let mut store = SharedStore::memory();

        // Test with empty store
        assert_eq!(store.len().unwrap(), 0);
        assert!(store.is_empty().unwrap());
        assert!(store.keys().unwrap().is_empty());
        assert!(!store.contains_key("any_key").unwrap());

        // Add some data
        store.set("key1".to_string(), "value1".to_string()).unwrap();
        store.set("key2".to_string(), 42i32).unwrap();
        store
            .set("key3".to_string(), json!({"nested": "data"}))
            .unwrap();

        // Test utility methods
        assert_eq!(store.len().unwrap(), 3);
        assert!(!store.is_empty().unwrap());
        assert!(store.contains_key("key1").unwrap());
        assert!(store.contains_key("key2").unwrap());
        assert!(store.contains_key("key3").unwrap());
        assert!(!store.contains_key("key4").unwrap());

        let mut keys = store.keys().unwrap();
        keys.sort();
        assert_eq!(keys, vec!["key1", "key2", "key3"]);

        // Test clear
        store.clear().unwrap();
        assert_eq!(store.len().unwrap(), 0);
        assert!(store.is_empty().unwrap());
    }

    #[test]
    #[cfg(feature = "storage-memory")]
    fn test_type_flexibility() {
        let mut store = SharedStore::memory();

        // Store the same key with different types (in separate tests conceptually)
        store
            .set("flexible_key".to_string(), "string_value".to_string())
            .unwrap();
        let as_string: Option<String> = store.get("flexible_key").unwrap();
        assert_eq!(as_string, Some("string_value".to_string()));

        // Clear and store as JSON
        store.clear().unwrap();
        store
            .set("flexible_key".to_string(), json!("json_value"))
            .unwrap();
        let as_json: Option<Value> = store.get("flexible_key").unwrap();
        assert_eq!(as_json, Some(json!("json_value")));

        // The JSON string should also deserializable as String
        let as_string_from_json: Option<String> = store.get("flexible_key").unwrap();
        assert_eq!(as_string_from_json, Some("json_value".to_string()));
    }

    #[test]
    #[cfg(feature = "storage-file")]
    fn test_file_shared_store() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_shared_store.json");
        let mut store = SharedStore::file(file_path.clone()).unwrap();

        // Test basic operations
        store
            .set("file_key".to_string(), json!("file_value"))
            .unwrap();
        assert_eq!(store.get("file_key").unwrap(), Some(json!("file_value")));

        // Test persistence by creating a new store with the same file
        let store2 = SharedStore::file(file_path).unwrap();
        assert_eq!(store2.get("file_key").unwrap(), Some(json!("file_value")));
    }
}
