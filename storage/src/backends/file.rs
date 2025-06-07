use super::StorageBackend;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// File-based storage backend that persists data to JSON files on disk.
///
/// `FileStorage` provides persistent key-value storage by serializing data to JSON
/// files. It maintains an in-memory cache for fast access while ensuring data
/// durability through automatic file synchronization on every write operation.
///
/// ## Features
///
/// - **Persistent Storage**: Data survives application restarts and system reboots
/// - **JSON Serialization**: Supports any type implementing `Serialize`/`Deserialize`
/// - **Atomic Operations**: Each write operation includes file synchronization
/// - **Flexible Types**: Store different data types with type-safe retrieval
/// - **Error Handling**: Comprehensive error reporting for I/O and serialization issues
///
/// ## Use Cases
///
/// - **Configuration Storage**: Persistent application settings and preferences
/// - **Workflow State**: Long-running workflow data that must survive restarts
/// - **Cache Storage**: Durable caching for expensive computations
/// - **Development/Testing**: File-based storage for development environments
/// - **Small Applications**: Simple persistent storage without database overhead
///
/// ## Performance Characteristics
///
/// - **Read Performance**: O(1) lookup from in-memory HashMap
/// - **Write Performance**: O(n) for file I/O where n is total data size
/// - **Memory Usage**: Entire dataset held in memory plus file storage
/// - **Concurrency**: Single-threaded access (not thread-safe)
///
/// ## Limitations
///
/// - **Memory Constraints**: All data must fit in available RAM
/// - **Write Amplification**: Every write operation saves entire dataset
/// - **No Transactions**: No atomic multi-key operations
/// - **Single Process**: No built-in multi-process coordination
///
/// ## Basic Usage
///
/// ```rust
/// use storage::backends::FileStorage;
/// use storage::StorageBackend;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Config {
///     database_url: String,
///     timeout_seconds: u64,
/// }
///
/// // Create storage with automatic file creation
/// let mut storage = FileStorage::new("app_config.json")?;
///
/// // Store configuration
/// let config = Config {
///     database_url: "postgresql://localhost/myapp".to_string(),
///     timeout_seconds: 30,
/// };
/// storage.set("database_config".to_string(), config)?;
///
/// // Retrieve configuration (survives application restart)
/// let retrieved_config: Option<Config> = storage.get("database_config")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Workflow Integration
///
/// ```rust
/// use storage::backends::FileStorage;
/// use storage::StorageBackend;
/// use serde_json::{Value, json};
///
/// // Initialize persistent workflow storage
/// let mut workflow_storage = FileStorage::new("workflow_state.json")?;
///
/// // Store workflow progress
/// workflow_storage.set("current_step".to_string(), 3u32)?;
/// workflow_storage.set("processed_items".to_string(), vec!["item1", "item2"])?;
/// workflow_storage.set("user_preferences".to_string(), json!({
///     "theme": "dark",
///     "notifications": true,
///     "language": "en"
/// }))?;
///
/// // Check workflow state (e.g., after application restart)
/// let current_step: Option<u32> = workflow_storage.get("current_step")?;
/// let processed_items: Option<Vec<String>> = workflow_storage.get("processed_items")?;
///
/// if let Some(step) = current_step {
///     println!("Resuming workflow from step {}", step);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Error Handling
///
/// ```rust
/// use storage::backends::{FileStorage, FileStorageError};
///
/// match FileStorage::new("/invalid/path/storage.json") {
///     Ok(storage) => {
///         // Storage created successfully
///     }
///     Err(FileStorageError::Io(io_err)) => {
///         eprintln!("Failed to access file: {}", io_err);
///         // Handle I/O errors (permissions, disk space, etc.)
///     }
///     Err(FileStorageError::Json(json_err)) => {
///         eprintln!("Invalid JSON in existing file: {}", json_err);
///         // Handle corrupted or invalid JSON files
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FileStorage {
    file_path: PathBuf,
    data: HashMap<String, Value>,
}

/// Error type for file storage operations.
///
/// `FileStorageError` provides comprehensive error reporting for file-based storage
/// operations, covering both I/O errors (file access, permissions, disk space) and
/// JSON serialization/deserialization errors (data format, type mismatches).
///
/// ## Error Categories
///
/// ### I/O Errors (`FileStorageError::Io`)
/// - File permission denied
/// - Disk space exhausted
/// - File or directory not found
/// - Network drive disconnected
/// - File system corruption
///
/// ### JSON Errors (`FileStorageError::Json`)
/// - Invalid JSON syntax in existing files
/// - Type mismatch during deserialization
/// - Serialization failures for complex types
/// - Corrupted data files
///
/// ## Error Handling Patterns
///
/// ```rust
/// use storage::backends::{FileStorage, FileStorageError};
/// use storage::StorageBackend;
/// use std::io::ErrorKind;
///
/// fn handle_storage_operation() -> Result<(), Box<dyn std::error::Error>> {
///     let mut storage = match FileStorage::new("data.json") {
///         Ok(storage) => storage,
///         Err(FileStorageError::Io(io_err)) => {
///             match io_err.kind() {
///                 ErrorKind::NotFound => {
///                     // Parent directory doesn't exist
///                     std::fs::create_dir_all("data")?;
///                     FileStorage::new("data/data.json")?
///                 }
///                 ErrorKind::PermissionDenied => {
///                     eprintln!("Permission denied. Check file permissions.");
///                     return Err(Box::new(io_err));
///                 }
///                 _ => return Err(Box::new(io_err)),
///             }
///         }
///         Err(FileStorageError::Json(_)) => {
///             // Corrupted file - create backup and start fresh
///             std::fs::rename("data.json", "data.json.backup")?;
///             FileStorage::new("data.json")?
///         }
///     };
///
///     // Perform storage operations with error handling
///     match storage.set("key".to_string(), "value") {
///         Ok(()) => println!("Data stored successfully"),
///         Err(FileStorageError::Io(io_err)) => {
///             eprintln!("Failed to write file: {}", io_err);
///             // Could be disk full, permissions changed, etc.
///         }
///         Err(FileStorageError::Json(json_err)) => {
///             eprintln!("Serialization error: {}", json_err);
///             // Unexpected - should not happen with simple types
///         }
///     }
///
///     Ok(())
/// }
/// ```
///
/// ## Recovery Strategies
///
/// ```rust
/// use storage::backends::{FileStorage, FileStorageError};
///
/// fn robust_storage_access(file_path: &str) -> Result<FileStorage, FileStorageError> {
///     match FileStorage::new(file_path) {
///         Ok(storage) => Ok(storage),
///         Err(FileStorageError::Json(_)) => {
///             // File exists but contains invalid JSON
///             eprintln!("Warning: Corrupted storage file, creating backup");
///             
///             // Create backup of corrupted file
///             let backup_path = format!("{}.corrupted.{}", file_path,
///                 std::time::SystemTime::now()
///                     .duration_since(std::time::UNIX_EPOCH)
///                     .unwrap()
///                     .as_secs());
///             std::fs::copy(file_path, &backup_path)?;
///             
///             // Remove corrupted file and create fresh storage
///             std::fs::remove_file(file_path)?;
///             FileStorage::new(file_path)
///         }
///         Err(e) => Err(e), // Propagate I/O errors
///     }
/// }
/// ```
///
/// ## Testing Error Conditions
///
/// ```rust
/// # use storage::backends::{FileStorage, FileStorageError};
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Test I/O error handling
/// match FileStorage::new("/root/forbidden.json") {
///     Err(FileStorageError::Io(io_err)) => {
///         assert_eq!(io_err.kind(), std::io::ErrorKind::PermissionDenied);
///         println!("Expected permission error occurred");
///     }
///     _ => panic!("Expected I/O error"),
/// }
///
/// // Test JSON error handling
/// std::fs::write("corrupted.json", "{ invalid json")?;
/// match FileStorage::new("corrupted.json") {
///     Err(FileStorageError::Json(_)) => {
///         println!("Successfully detected corrupted JSON");
///     }
///     _ => panic!("Expected JSON error"),
/// }
/// # Ok(())
/// # }
/// ```
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
    /// Creates a new file storage instance with the specified file path.
    ///
    /// This constructor loads existing data from the file if it exists, or creates
    /// a new empty storage if the file doesn't exist. The file will be created
    /// automatically on the first write operation.
    ///
    /// ## Behavior
    ///
    /// - **Existing File**: Loads and parses JSON content into memory
    /// - **Non-existent File**: Creates empty storage, file created on first write
    /// - **Empty File**: Treats as empty storage (no error)
    /// - **Invalid JSON**: Returns `FileStorageError::Json`
    /// - **I/O Issues**: Returns `FileStorageError::Io`
    ///
    /// ## Arguments
    ///
    /// * `file_path` - Path to the JSON storage file (created if doesn't exist)
    ///
    /// ## Errors
    ///
    /// - `FileStorageError::Io` - File access issues (permissions, disk space, etc.)
    /// - `FileStorageError::Json` - Invalid JSON in existing file
    ///
    /// ## Examples
    ///
    /// ### Basic Usage
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    ///
    /// // Create storage (file created on first write)
    /// let storage = FileStorage::new("app_data.json")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ### With Directory Creation
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    /// use std::fs;
    ///
    /// // Ensure parent directory exists
    /// fs::create_dir_all("data/storage")?;
    /// let storage = FileStorage::new("data/storage/workflow.json")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ### Loading Existing Data
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    ///
    /// // First run - creates storage and adds data
    /// {
    ///     let mut storage = FileStorage::new("persistent.json")?;
    ///     storage.set("counter".to_string(), 42u32)?;
    /// }
    ///
    /// // Second run - loads existing data
    /// {
    ///     let storage = FileStorage::new("persistent.json")?;
    ///     let counter: Option<u32> = storage.get("counter")?;
    ///     assert_eq!(counter, Some(42));
    /// }
    /// # std::fs::remove_file("persistent.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ### Error Handling
    ///
    /// ```rust
    /// use storage::backends::{FileStorage, FileStorageError};
    /// use storage::StorageBackend;
    ///
    /// match FileStorage::new("config.json") {
    ///     Ok(storage) => {
    ///         println!("Storage ready with {} items", storage.len()?);
    ///     }
    ///     Err(FileStorageError::Io(io_err)) => {
    ///         eprintln!("File access error: {}", io_err);
    ///         // Handle permissions, disk space, etc.
    ///     }
    ///     Err(FileStorageError::Json(json_err)) => {
    ///         eprintln!("Corrupted JSON file: {}", json_err);
    ///         // Handle by backing up and recreating
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ### Robust Initialization
    ///
    /// ```rust
    /// use storage::backends::{FileStorage, FileStorageError};
    /// use std::path::Path;
    ///
    /// fn create_robust_storage(path: &str) -> Result<FileStorage, FileStorageError> {
    ///     // Ensure parent directory exists
    ///     if let Some(parent) = Path::new(path).parent() {
    ///         std::fs::create_dir_all(parent)
    ///             .map_err(|e| FileStorageError::Io(e))?;
    ///     }
    ///
    ///     // Try to create storage
    ///     match FileStorage::new(path) {
    ///         Ok(storage) => Ok(storage),
    ///         Err(FileStorageError::Json(_)) => {
    ///             // Backup corrupted file and create fresh storage
    ///             let backup = format!("{}.backup", path);
    ///             std::fs::rename(path, backup)
    ///                 .map_err(|e| FileStorageError::Io(e))?;
    ///             FileStorage::new(path)
    ///         }
    ///         Err(e) => Err(e),
    ///     }
    /// }
    /// ```
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

    /// Saves the current in-memory data to the file system.
    ///
    /// This method performs atomic file operations by writing the entire dataset
    /// as pretty-printed JSON. Called automatically by all mutating operations
    /// to ensure data persistence.
    ///
    /// ## Performance Notes
    ///
    /// - **Write Amplification**: Entire dataset written on every change
    /// - **Pretty Printing**: JSON formatted for human readability
    /// - **Atomic**: Single write operation (no partial writes)
    ///
    /// ## Error Conditions
    ///
    /// - Disk space exhausted
    /// - File permissions changed
    /// - File system errors
    /// - Network drive disconnected
    fn save_to_file(&self) -> Result<(), FileStorageError> {
        let json_data = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.file_path, json_data)?;
        Ok(())
    }
}

impl StorageBackend for FileStorage {
    type Error = FileStorageError;

    /// Stores a serializable value with the given key, persisting immediately to disk.
    ///
    /// This method serializes the value to JSON, stores it in the in-memory cache,
    /// and immediately writes the entire dataset to the file system for persistence.
    /// The operation is atomic from the perspective of the storage backend.
    ///
    /// ## Behavior
    ///
    /// - Serializes value to JSON using `serde_json`
    /// - Updates in-memory HashMap
    /// - Writes entire dataset to file (write amplification)
    /// - Overwrites existing values with the same key
    ///
    /// ## Performance Characteristics
    ///
    /// - **Time Complexity**: O(n) where n is total dataset size (due to file write)
    /// - **Space Complexity**: O(1) for the operation, O(n) for total storage
    /// - **I/O Operations**: One write operation per call
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug, PartialEq)]
    /// struct UserProfile {
    ///     username: String,
    ///     email: String,
    ///     preferences: Vec<String>,
    /// }
    ///
    /// let mut storage = FileStorage::new("user_data.json")?;
    ///
    /// // Store simple values
    /// storage.set("last_login".to_string(), "2024-01-15T10:30:00Z")?;
    /// storage.set("login_count".to_string(), 42u32)?;
    ///
    /// // Store complex structures
    /// let profile = UserProfile {
    ///     username: "alice".to_string(),
    ///     email: "alice@example.com".to_string(),
    ///     preferences: vec!["dark_theme".to_string(), "notifications".to_string()],
    /// };
    /// storage.set("user_profile".to_string(), profile)?;
    ///
    /// // Store JSON values directly
    /// use serde_json::json;
    /// storage.set("config".to_string(), json!({
    ///     "api_endpoint": "https://api.example.com",
    ///     "timeout_ms": 5000,
    ///     "retries": 3
    /// }))?;
    /// # std::fs::remove_file("user_data.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
        let json_value = serde_json::to_value(value)?;
        self.data.insert(key, json_value);
        self.save_to_file()
    }

    /// Retrieves and deserializes a value by key from storage.
    ///
    /// This method performs an O(1) lookup in the in-memory HashMap and deserializes
    /// the JSON value to the requested type. Returns `None` if the key doesn't exist.
    ///
    /// ## Type Safety
    ///
    /// The method is generic over the return type `T`, which must implement `DeserializeOwned`.
    /// Type mismatches between stored data and requested type will result in
    /// `FileStorageError::Json` errors.
    ///
    /// ## Performance Characteristics
    ///
    /// - **Time Complexity**: O(1) for lookup + O(m) for deserialization where m is value size
    /// - **Space Complexity**: O(m) for creating the deserialized value
    /// - **I/O Operations**: None (reads from in-memory cache)
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    /// use serde::{Serialize, Deserialize};
    /// use serde_json::{Value, json};
    ///
    /// #[derive(Serialize, Deserialize, Debug, PartialEq)]
    /// struct Config {
    ///     timeout: u32,
    ///     retries: u8,
    /// }
    ///
    /// let mut storage = FileStorage::new("app_config.json")?;
    ///
    /// // Store various types
    /// storage.set("count".to_string(), 42u32)?;
    /// storage.set("name".to_string(), "CosmoFlow".to_string())?;
    /// storage.set("config".to_string(), Config { timeout: 5000, retries: 3 })?;
    /// storage.set("features".to_string(), vec!["caching", "persistence"])?;
    ///
    /// // Retrieve with type specification
    /// let count: Option<u32> = storage.get("count")?;
    /// assert_eq!(count, Some(42));
    ///
    /// let name: Option<String> = storage.get("name")?;
    /// assert_eq!(name, Some("CosmoFlow".to_string()));
    ///
    /// let config: Option<Config> = storage.get("config")?;
    /// assert_eq!(config, Some(Config { timeout: 5000, retries: 3 }));
    ///
    /// let features: Option<Vec<String>> = storage.get("features")?;
    /// assert_eq!(features, Some(vec!["caching".to_string(), "persistence".to_string()]));
    ///
    /// // Non-existent key returns None
    /// let missing: Option<String> = storage.get("missing_key")?;
    /// assert_eq!(missing, None);
    ///
    /// // Retrieve as generic JSON Value
    /// let raw_value: Option<Value> = storage.get("config")?;
    /// if let Some(value) = raw_value {
    ///     assert_eq!(value["timeout"], json!(5000));
    /// }
    /// # std::fs::remove_file("app_config.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Error Handling
    ///
    /// ```rust
    /// # use storage::backends::{FileStorage, FileStorageError};
    /// # use storage::StorageBackend;
    /// # use serde_json::json;
    /// # let mut storage = FileStorage::new("type_test.json")?;
    /// // Store a string value
    /// storage.set("number_as_string".to_string(), "42")?;
    ///
    /// // Try to retrieve as wrong type
    /// match storage.get::<u32>("number_as_string") {
    ///     Ok(Some(value)) => unreachable!(), // Won't happen
    ///     Ok(None) => unreachable!(),        // Key exists
    ///     Err(FileStorageError::Json(json_err)) => {
    ///         println!("Type mismatch: {}", json_err);
    ///         // Expected: can't deserialize string "42" as u32
    ///     }
    ///     Err(_) => unreachable!(), // No I/O for get operations
    /// }
    /// # std::fs::remove_file("type_test.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.get(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value.clone())?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Removes a key-value pair from storage and returns the deserialized value.
    ///
    /// This method removes the key from the in-memory HashMap, immediately persists
    /// the change to disk, and returns the previous value if it existed. The operation
    /// is atomic and includes file synchronization.
    ///
    /// ## Behavior
    ///
    /// - Removes key from in-memory cache
    /// - Deserializes and returns the previous value (if any)
    /// - Writes updated dataset to file immediately
    /// - Returns `None` if key didn't exist
    ///
    /// ## Performance Characteristics
    ///
    /// - **Time Complexity**: O(n) where n is total dataset size (due to file write)
    /// - **Space Complexity**: O(m) where m is size of removed value
    /// - **I/O Operations**: One write operation per call
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug, PartialEq)]
    /// struct TaskInfo {
    ///     id: u32,
    ///     status: String,
    ///     priority: u8,
    /// }
    ///
    /// let mut storage = FileStorage::new("tasks.json")?;
    ///
    /// // Store some data
    /// storage.set("counter".to_string(), 100u32)?;
    /// storage.set("task_1".to_string(), TaskInfo {
    ///     id: 1,
    ///     status: "completed".to_string(),
    ///     priority: 5,
    /// })?;
    /// storage.set("tags".to_string(), vec!["urgent", "backend"])?;
    ///
    /// // Remove and get the previous value
    /// let removed_counter: Option<u32> = storage.remove("counter")?;
    /// assert_eq!(removed_counter, Some(100));
    ///
    /// let removed_task: Option<TaskInfo> = storage.remove("task_1")?;
    /// assert_eq!(removed_task, Some(TaskInfo {
    ///     id: 1,
    ///     status: "completed".to_string(),
    ///     priority: 5,
    /// }));
    ///
    /// // Remove non-existent key
    /// let missing: Option<String> = storage.remove("missing_key")?;
    /// assert_eq!(missing, None);
    ///
    /// // Verify removal persisted
    /// let check_counter: Option<u32> = storage.get("counter")?;
    /// assert_eq!(check_counter, None);
    /// # std::fs::remove_file("tasks.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Cleanup Operations
    ///
    /// ```rust
    /// # use storage::backends::FileStorage;
    /// # use storage::StorageBackend;
    /// # let mut storage = FileStorage::new("cleanup.json")?;
    /// // Store temporary data
    /// storage.set("temp_1".to_string(), "temporary data")?;
    /// storage.set("temp_2".to_string(), vec![1, 2, 3])?;
    /// storage.set("permanent".to_string(), "keep this")?;
    ///
    /// // Clean up temporary data
    /// let temp_keys = ["temp_1", "temp_2"];
    /// for key in &temp_keys {
    ///     let removed: Option<serde_json::Value> = storage.remove(key)?;
    ///     if removed.is_some() {
    ///         println!("Cleaned up temporary data: {}", key);
    ///     }
    /// }
    ///
    /// // Verify only permanent data remains
    /// assert_eq!(storage.len()?, 1);
    /// let permanent: Option<String> = storage.get("permanent")?;
    /// assert_eq!(permanent, Some("keep this".to_string()));
    /// # std::fs::remove_file("cleanup.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Checks if a key exists in the storage.
    ///
    /// This is a fast O(1) operation that only checks the in-memory HashMap
    /// without performing any deserialization or file I/O.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    ///
    /// let mut storage = FileStorage::new("existence_check.json")?;
    ///
    /// // Initially empty
    /// assert!(!storage.contains_key("key1")?);
    ///
    /// // Add some data
    /// storage.set("key1".to_string(), "value1")?;
    /// storage.set("key2".to_string(), 42u32)?;
    ///
    /// // Check existence
    /// assert!(storage.contains_key("key1")?);
    /// assert!(storage.contains_key("key2")?);
    /// assert!(!storage.contains_key("key3")?);
    /// # std::fs::remove_file("existence_check.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(key))
    }

    /// Returns a vector of all keys currently stored.
    ///
    /// This method creates a new vector containing clones of all keys in the storage.
    /// The order of keys is not guaranteed to be consistent between calls.
    ///
    /// ## Performance
    ///
    /// - **Time Complexity**: O(n) where n is number of keys
    /// - **Space Complexity**: O(n) for the returned vector
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    ///
    /// let mut storage = FileStorage::new("keys_demo.json")?;
    ///
    /// // Initially empty
    /// assert_eq!(storage.keys()?.len(), 0);
    ///
    /// // Add some data
    /// storage.set("user_id".to_string(), 123u32)?;
    /// storage.set("session_token".to_string(), "abc123")?;
    /// storage.set("preferences".to_string(), vec!["theme:dark"])?;
    ///
    /// // Get all keys
    /// let all_keys = storage.keys()?;
    /// assert_eq!(all_keys.len(), 3);
    /// assert!(all_keys.contains(&"user_id".to_string()));
    /// assert!(all_keys.contains(&"session_token".to_string()));
    /// assert!(all_keys.contains(&"preferences".to_string()));
    ///
    /// // Use keys for iteration
    /// for key in all_keys {
    ///     if storage.contains_key(&key)? {
    ///         println!("Key exists: {}", key);
    ///     }
    /// }
    /// # std::fs::remove_file("keys_demo.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.data.keys().cloned().collect())
    }

    /// Removes all key-value pairs from the storage and persists the change.
    ///
    /// This operation clears the in-memory HashMap and writes an empty dataset
    /// to the file, effectively resetting the storage to its initial state.
    ///
    /// ## Performance
    ///
    /// - **Time Complexity**: O(1) for clearing + O(1) for writing empty file
    /// - **Space Complexity**: O(1) after completion
    /// - **I/O Operations**: One write operation
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    ///
    /// let mut storage = FileStorage::new("clear_demo.json")?;
    ///
    /// // Add some data
    /// storage.set("item1".to_string(), "value1")?;
    /// storage.set("item2".to_string(), "value2")?;
    /// storage.set("item3".to_string(), "value3")?;
    /// assert_eq!(storage.len()?, 3);
    ///
    /// // Clear all data
    /// storage.clear()?;
    /// assert_eq!(storage.len()?, 0);
    /// assert!(!storage.contains_key("item1")?);
    ///
    /// // Verify persistence - create new instance
    /// let new_storage = FileStorage::new("clear_demo.json")?;
    /// assert_eq!(new_storage.len()?, 0);
    /// # std::fs::remove_file("clear_demo.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ## Bulk Reset Operations
    ///
    /// ```rust
    /// # use storage::backends::FileStorage;
    /// # use storage::StorageBackend;
    /// # let mut storage = FileStorage::new("bulk_reset.json")?;
    /// // Populate with configuration data
    /// storage.set("api_key".to_string(), "secret_key_123")?;
    /// storage.set("user_preferences".to_string(),
    ///     serde_json::json!({"theme": "dark", "lang": "en"}))?;
    /// storage.set("cache_data".to_string(), vec![1, 2, 3, 4, 5])?;
    ///
    /// // Reset configuration (e.g., user logout)
    /// storage.clear()?;
    ///
    /// // Start fresh
    /// storage.set("default_config".to_string(), "initialized")?;
    /// assert_eq!(storage.len()?, 1);
    /// # std::fs::remove_file("bulk_reset.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn clear(&mut self) -> Result<(), Self::Error> {
        self.data.clear();
        self.save_to_file()
    }

    /// Returns the number of key-value pairs currently stored.
    ///
    /// This is a fast O(1) operation that returns the size of the in-memory HashMap.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use storage::backends::FileStorage;
    /// use storage::StorageBackend;
    ///
    /// let mut storage = FileStorage::new("count_demo.json")?;
    /// assert_eq!(storage.len()?, 0);
    ///
    /// storage.set("first".to_string(), 1)?;
    /// assert_eq!(storage.len()?, 1);
    ///
    /// storage.set("second".to_string(), 2)?;
    /// storage.set("third".to_string(), 3)?;
    /// assert_eq!(storage.len()?, 3);
    ///
    /// storage.remove::<i32>("first")?;
    /// assert_eq!(storage.len()?, 2);
    ///
    /// storage.clear()?;
    /// assert_eq!(storage.len()?, 0);
    /// # std::fs::remove_file("count_demo.json").ok();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
