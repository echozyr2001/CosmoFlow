#![deny(missing_docs)]
//! # CosmoFlow
//!
//! A lightweight, type-safe workflow engine for Rust, optimized for LLM applications.
//!
//! CosmoFlow provides a minimal yet powerful framework for building complex workflows
//! with clean abstractions and excellent performance.
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
//! ```rust,no_run
//! # #[cfg(feature = "storage-memory")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use cosmoflow::prelude::*;
//! use cosmoflow::flow::FlowBackend;
//! use async_trait::async_trait;
//!
//! // Create a shared store with memory backend
//! let mut store = MemoryStorage::new();
//!
//! // Define a simple node
//! struct MyNode;
//! #[async_trait]
//! impl<S: SharedStore> Node<S> for MyNode {
//!     type PrepResult = String;
//!     type ExecResult = ();
//!     type Error = NodeError;
//!     async fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<String, Self::Error> {
//!         Ok("prepared".to_string())
//!     }
//!     async fn exec(&mut self, _prep_result: String, _context: &ExecutionContext) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     async fn post(&mut self, _store: &mut S, _prep_result: String, _exec_result: (), _context: &ExecutionContext) -> Result<Action, Self::Error> {
//!         Ok(Action::simple("complete"))
//!     }
//! }
//!
//! // Create a flow
//! let mut flow = FlowBuilder::new()
//!     .node("start", MyNode)
//!     .terminal_route("start", "complete")
//!     .build();
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
//! *   `storage-memory`: In-memory storage backend.
//! *   `storage-file`: File-based storage backend.
//! *   `storage-redis`: Redis storage backend for distributed workflows.
//!
//! ### Convenience Features
//!
//! *   `minimal`: Just the core engine (bring your own storage).
//! *   `basic`: Core + memory storage (perfect for development).
//! *   `standard`: Core + memory storage.
//! *   `full`: All storage backends enabled.

// ============================================================================
// CORE EXPORTS
// ============================================================================

/// Shared store for data communication between workflow nodes
pub mod shared_store;
pub use shared_store::SharedStore;

/// Action definition and condition evaluation
pub mod action;
pub use action::Action;

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
        Action, ExecutionContext, Flow, FlowBackend, FlowBuilder, FlowConfig, FlowExecutionResult,
        Node, NodeError, SharedStore,
    };

    // Storage backends
    #[cfg(feature = "storage-memory")]
    pub use crate::shared_store::backends::MemoryStorage;

    #[cfg(feature = "storage-file")]
    pub use crate::shared_store::backends::FileStorage;

    #[cfg(feature = "storage-redis")]
    pub use crate::shared_store::backends::RedisStorage;
}
