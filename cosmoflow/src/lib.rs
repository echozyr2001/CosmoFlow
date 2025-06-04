pub use shared_store::SharedStore;

pub mod storage {
    pub use shared_store::StorageBackend;

    #[cfg(feature = "storage-memory")]
    pub use shared_store::{MemoryStorage, MemoryStorageError};

    #[cfg(feature = "storage-file")]
    pub use shared_store::{FileStorage, FileStorageError};
}
