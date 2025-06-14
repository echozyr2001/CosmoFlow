#![deny(missing_docs)]
//! # CosmoFlow Storage
//!
//! This module provides the storage backend abstractions for the CosmoFlow engine.
//!
//! It defines the `StorageBackend` trait, which is implemented by various storage
//! providers (e.g., in-memory, file-based). This module is a low-level component
//! of CosmoFlow and is not intended to be used directly in most applications.
//!
//! For more information, please see the main [`cosmoflow`](https://docs.rs/cosmoflow)
//! crate documentation.

pub mod backends;

// Re-export the new unified SharedStore trait
pub use crate::shared_store::new_design::SharedStore;

#[cfg(feature = "storage-memory")]
pub use backends::{MemoryStorage, MemoryStorageError};

#[cfg(feature = "storage-file")]
pub use backends::{FileStorage, FileStorageError};

#[cfg(feature = "storage-redis")]
pub use backends::{RedisStorage, RedisStorageError};
