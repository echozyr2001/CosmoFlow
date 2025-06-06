use super::StorageBackend;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// File-based storage backend that persists data to JSON files
///
/// Internally stores data as JSON Values for flexibility while providing
/// a generic interface for serializable/deserializable types.
#[derive(Debug, Clone)]
pub struct FileStorage {
    file_path: PathBuf,
    data: HashMap<String, Value>,
}

/// Error type for file storage operations
#[derive(Debug, Error)]
pub enum FileStorageError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl FileStorage {
    /// Create a new file storage with the specified file path
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, FileStorageError> {
        let file_path = file_path.as_ref().to_path_buf();
        let data = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            if content.trim().is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(&content)?
            }
        } else {
            HashMap::new()
        };

        Ok(Self { file_path, data })
    }

    /// Save the current data to file
    fn save_to_file(&self) -> Result<(), FileStorageError> {
        let json_data = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, json_data)?;
        Ok(())
    }
}

impl StorageBackend for FileStorage {
    type Error = FileStorageError;

    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
        let json_value = serde_json::to_value(value)?;
        self.data.insert(key, json_value);
        self.save_to_file()
    }

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.get(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value.clone())?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.remove(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value)?;
                self.save_to_file()?;
                Ok(Some(deserialized))
            }
            None => {
                self.save_to_file()?;
                Ok(None)
            }
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
        self.save_to_file()
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
    use std::error::Error;
    use std::fs;
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestData {
        id: i32,
        name: String,
    }

    #[test]
    fn test_file_storage_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_storage.json");

        let mut storage = FileStorage::new(&file_path).unwrap();

        // Test set and get with JSON Value
        storage.set("key1".to_string(), json!("value1")).unwrap();
        let retrieved: Option<Value> = storage.get("key1").unwrap();
        assert_eq!(retrieved, Some(json!("value1")));

        // Test persistence by creating a new instance
        let storage2 = FileStorage::new(&file_path).unwrap();
        let retrieved2: Option<Value> = storage2.get("key1").unwrap();
        assert_eq!(retrieved2, Some(json!("value1")));

        // Clean up
        fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_storage_with_structs() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_structs.json");

        let mut storage = FileStorage::new(&file_path).unwrap();
        let test_data = TestData {
            id: 42,
            name: "Test Item".to_string(),
        };

        // Test set and get with custom struct
        storage.set("struct_key".to_string(), &test_data).unwrap();
        let retrieved: Option<TestData> = storage.get("struct_key").unwrap();
        assert_eq!(retrieved, Some(test_data.clone()));

        // Test persistence with structs
        let storage2 = FileStorage::new(&file_path).unwrap();
        let retrieved2: Option<TestData> = storage2.get("struct_key").unwrap();
        assert_eq!(retrieved2, Some(test_data));

        // Clean up
        fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_storage_persistence() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_persistence.json");

        // Create storage and add data
        {
            let mut storage = FileStorage::new(&file_path).unwrap();
            storage
                .set(
                    "persistent_key".to_string(),
                    json!({"data": "persistent_value"}),
                )
                .unwrap();
        }

        // Create new storage instance and verify data persisted
        {
            let storage = FileStorage::new(&file_path).unwrap();
            let retrieved: Option<Value> = storage.get("persistent_key").unwrap();
            assert_eq!(retrieved, Some(json!({"data": "persistent_value"})));
            assert_eq!(storage.len().unwrap(), 1);
        }

        // Clean up
        fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_storage_remove_with_type() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_remove.json");

        let mut storage = FileStorage::new(&file_path).unwrap();

        // Set some data
        storage
            .set("string_key".to_string(), "test_string".to_string())
            .unwrap();
        storage.set("number_key".to_string(), 42i32).unwrap();

        // Remove with type specification
        let removed_string: Option<String> = storage.remove("string_key").unwrap();
        assert_eq!(removed_string, Some("test_string".to_string()));

        let removed_number: Option<i32> = storage.remove("number_key").unwrap();
        assert_eq!(removed_number, Some(42));

        // Verify they're gone
        let check_string: Option<String> = storage.get("string_key").unwrap();
        assert_eq!(check_string, None);

        // Clean up
        fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_storage_io_errors() {
        // Test reading from a directory instead of a file (should cause I/O error)
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().join("not_a_file");
        fs::create_dir(&dir_path).unwrap();

        let result = FileStorage::new(&dir_path);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, FileStorageError::Io(_)));

        // Test error message formatting
        let error_msg = error.to_string();
        assert!(error_msg.contains("I/O error"));

        // Test error source chaining
        assert!(error.source().is_some());
    }

    #[test]
    fn test_file_storage_json_errors() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("corrupted.json");

        // Create a file with invalid JSON
        fs::write(&file_path, "{ this is not valid json }").unwrap();

        let result = FileStorage::new(&file_path);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, FileStorageError::Json(_)));

        // Test error message formatting
        let error_msg = error.to_string();
        assert!(error_msg.contains("JSON error"));

        // Test error source chaining
        assert!(error.source().is_some());

        // Clean up
        fs::remove_file(&file_path).ok();
    }

    #[test]
    fn test_file_storage_error_conversion() {
        // Test that io::Error is properly converted to FileStorageError
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
        let file_error: FileStorageError = io_error.into();
        assert!(matches!(file_error, FileStorageError::Io(_)));

        // Test that serde_json::Error is properly converted to FileStorageError
        let json_str = "{ invalid json";
        let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let file_error: FileStorageError = json_error.into();
        assert!(matches!(file_error, FileStorageError::Json(_)));
    }

    #[test]
    fn test_file_storage_write_permission_error() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("readonly.json");

        // Create file and make it read-only
        fs::write(&file_path, "{}").unwrap();
        let mut permissions = fs::metadata(&file_path).unwrap().permissions();
        permissions.set_mode(0o444); // Read-only
        fs::set_permissions(&file_path, permissions).unwrap();

        // Try to create storage which should fail when trying to write
        let mut storage = FileStorage::new(&file_path).unwrap();
        let result = storage.set("key".to_string(), "value");

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, FileStorageError::Io(_)));
        assert!(error.to_string().contains("I/O error"));

        // Clean up - restore write permissions first
        let mut permissions = fs::metadata(&file_path).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&file_path, permissions).unwrap();
        fs::remove_file(&file_path).ok();
    }
}
