//! Storage backends for CosmoFlow
//!
//! This crate provides various storage backend implementations:
//!
//! - Memory storage (feature: `memory`)
//! - File storage (feature: `file`)

pub mod backends;

pub use backends::StorageBackend;

#[cfg(feature = "memory")]
pub use backends::{MemoryStorage, MemoryStorageError};

#[cfg(feature = "file")]
pub use backends::{FileStorage, FileStorageError};

pub mod prelude {
    pub use crate::StorageBackend;

    #[cfg(feature = "memory")]
    pub use crate::backends::{MemoryStorage, MemoryStorageError};

    #[cfg(feature = "file")]
    pub use crate::backends::{FileStorage, FileStorageError};
}
