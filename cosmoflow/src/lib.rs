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
//! use cosmoflow::flow::FlowBackend;
//! use std::time::Duration;
//!
//! // Create a shared store with memory backend
//! let mut store = MemoryStorage::new();
//!
//! // Create a flow
//! let mut flow = Flow::new();
//!
//! // Execute the flow
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

// ============================================================================
// CORE EXPORTS
// ============================================================================

/// Shared store for data communication between workflow nodes
pub mod shared_store;
pub use shared_store::SharedStore;

/// Action definition and condition evaluation
pub mod action;
pub use action::{Action, ActionCondition};

/// Flow definition and execution
pub mod flow;
pub use flow::{
    Flow, FlowBackend, FlowBuilder, FlowConfig, FlowExecutionResult, errors::FlowError,
    route::Route,
};

/// Node execution system and traits
pub mod node;
pub use node::{ExecutionContext, Node, NodeError};

// ============================================================================
// FEATURE-GATED EXPORTS
// ============================================================================

/// Built-in node implementations (optional)
#[cfg(feature = "builtin")]
pub mod builtin;

// ============================================================================
// CONVENIENCE TYPE ALIAS
// ============================================================================

/// A convenient result type alias for CosmoFlow operations
pub type Result<T> = std::result::Result<T, FlowError>;

// ============================================================================
// PRELUDE MODULE
// ============================================================================

/// The prelude module for commonly used types and traits.
///
/// This module provides a convenient way to import the most commonly used
/// types and traits from CosmoFlow. Import this module to get started quickly:
///
/// ```rust
/// use cosmoflow::prelude::*;
/// ```
pub mod prelude {
    // Core types
    pub use crate::{
        Action, ActionCondition, ExecutionContext, Flow, FlowBackend, FlowBuilder, FlowConfig,
        FlowExecutionResult, Node, NodeError, SharedStore,
    };

    // Feature-gated re-exports
    #[cfg(feature = "builtin")]
    pub use crate::builtin::*;

    #[cfg(feature = "storage-memory")]
    pub use crate::shared_store::backends::MemoryStorage;

    #[cfg(feature = "storage-file")]
    pub use crate::shared_store::backends::FileStorage;

    #[cfg(feature = "storage-redis")]
    pub use crate::shared_store::backends::RedisStorage;
}
