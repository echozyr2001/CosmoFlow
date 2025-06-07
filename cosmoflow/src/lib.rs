#![deny(missing_docs)]
//! # CosmoFlow
//!
//! A Rust-native, lightweight, and extensible workflow engine for building
//! complex, stateful, and event-driven applications. Inspired by PocketFlow and
//! optimized for LLM applications.
//!
//! ## Core Concepts
//!
//! *   **Flow**: A collection of nodes and the routes between them, representing a
//!     complete workflow.
//! *   **Node**: A single unit of work in a workflow.
//! *   **Action**: The result of a node's execution, used to determine the next
//!     step in the flow.
//! *   **Shared Store**: A key-value store used to share data between nodes.
//! *   **Storage Backend**: A pluggable storage mechanism for the shared store.
//!
//! ## Quick Start
//!
//! ```rust
//! # #[cfg(feature = "storage-memory")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use cosmoflow::prelude::*;
//! use cosmoflow::flows::FlowBackend;
//! use std::time::Duration;
//!
//! // Create a shared store with memory backend
//! let mut store = SharedStore::with_storage(MemoryStorage::new());
//!
//! // Create a flow
//! let mut flow = Flow::new();
//!
//! // Execute the flow
//! let context = ExecutionContext::new(3, Duration::from_secs(1));
//! let result = flow.execute(&mut store).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! CosmoFlow uses a feature flag system to keep the core library lightweight
//! and allow users to opt-in to additional functionality.
//!
//! ### Storage Backends
//!
//! *   `storage-memory`: In-memory storage backend (default).
//! *   `storage-file`: File-based storage backend.
//!
//! ### Built-in Nodes
//!
//! *   `builtin`: A collection of pre-built nodes for common tasks.
//!
//! ### Convenience Features
//!
//! *   `standard`: Enables `storage-memory` and `builtin` features.
//! *   `full`: Enables all features.

// Core Exports
pub use action::{Action, ActionCondition};
pub use flow::errors::FlowError;
pub use flow::route::Route;
pub use flow::{Flow, FlowBuilder, FlowConfig, FlowExecutionResult};
pub use node::{ExecutionContext, Node, NodeBackend, NodeError};

pub mod shared_store;
pub use shared_store::SharedStore;

// Module Exports

/// Action definition and condition evaluation
pub mod actions {
    pub use ::action::*;
}

/// Built-in node implementations (optional)
#[cfg(feature = "builtin")]
pub mod builtin {
    pub use builtin::*;
}

/// Flow definition and execution
pub mod flows {
    pub use flow::*;
}

/// Node execution system and traits
pub mod nodes {
    pub use node::*;
}

/// Storage backend abstractions and implementations
pub mod storage;
// FIXME: re-export
// pub mod storage {
//     pub use storage::StorageBackend;
//     #[cfg(feature = "storage-file")]
//     pub use storage::backends::FileStorage;
//     #[cfg(feature = "storage-memory")]
//     pub use storage::backends::MemoryStorage;
// }

/// A convenient result type alias for CosmoFlow operations
pub type Result<T> = std::result::Result<T, FlowError>;

/// The prelude module for commonly used types and traits.
pub mod prelude {
    pub use crate::{
        Action, ActionCondition, ExecutionContext, Flow, FlowBuilder, FlowConfig,
        FlowExecutionResult, Node, NodeBackend, SharedStore,
    };

    #[cfg(feature = "builtin")]
    pub use crate::builtin::*;

    pub use crate::storage::StorageBackend;

    #[cfg(feature = "storage-file")]
    pub use crate::storage::FileStorage;
    #[cfg(feature = "storage-memory")]
    pub use crate::storage::MemoryStorage;
}
