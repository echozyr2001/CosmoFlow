#![deny(missing_docs)]
//! # CosmoFlow Storage
//!
//! This crate provides the storage backend abstractions for the CosmoFlow engine.
//!
//! It defines the `StorageBackend` trait, which is implemented by various storage
//! providers (e.g., in-memory, file-based). This crate is a low-level component
//! of CosmoFlow and is not intended to be used directly in most applications.
//!
//! For more information, please see the main [`cosmoflow`](https://docs.rs/cosmoflow)
//! crate documentation.

pub mod backends;

pub use backends::StorageBackend;

#[cfg(feature = "storage-memory")]
pub use backends::{MemoryStorage, MemoryStorageError};

#[cfg(feature = "storage-file")]
pub use backends::{FileStorage, FileStorageError};
