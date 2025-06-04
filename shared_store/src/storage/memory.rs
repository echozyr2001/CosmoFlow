use super::StorageBackend;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Simple in-memory storage backend using HashMap
///
/// Internally stores data as JSON Values for flexibility while providing
/// a generic interface for serializable/deserializable types.
#[derive(Debug, Clone, Default)]
pub struct MemoryStorage {
    data: HashMap<String, Value>,
}

/// Error type for in-memory storage operations
#[derive(Debug, Clone, Error)]
pub enum MemoryStorageError {
    /// JSON serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// JSON deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl MemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create a new in-memory storage with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }
}

impl StorageBackend for MemoryStorage {
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
