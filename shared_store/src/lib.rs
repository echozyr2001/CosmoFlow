use serde::{Serialize, de::DeserializeOwned};
use storage::StorageBackend;

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
