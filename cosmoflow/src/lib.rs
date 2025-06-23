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
//! # Quick Start
//!
//! ## Synchronous Usage (default)
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "storage-memory", not(feature = "async")))]
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use cosmoflow::prelude::*;
//!
//! // Create a shared store with memory backend
//! let mut store = MemoryStorage::new();
//!
//! // Define a simple node
//! struct MyNode;
//! impl<S: SharedStore> Node<S> for MyNode {
//!     type PrepResult = String;
//!     type ExecResult = ();
//!     type Error = NodeError;
//!     fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<String, Self::Error> {
//!         Ok("prepared".to_string())
//!     }
//!     fn exec(&mut self, _prep_result: String, _context: &ExecutionContext) -> Result<(), Self::Error> {
//!         Ok(())
//!     }
//!     fn post(&mut self, _store: &mut S, _prep_result: String, _exec_result: (), _context: &ExecutionContext) -> Result<Action, Self::Error> {
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
//! let result = flow.execute(&mut store)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Asynchronous Usage (with async feature)
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "async", feature = "storage-memory"))]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use cosmoflow::prelude::*;
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
//! *   `minimal`: Just the core engine (bring your own storage) - sync only.
//! *   `basic`: Core + memory storage - sync only.
//! *   `standard`: Core + memory storage + async support.
//! *   `full`: All storage backends + async support enabled.
//!
//! ### Sync/Async Mode
//!
//! *   `async`: Enable async/await support (requires tokio runtime).
//! *   Without `async`: Synchronous execution only (lighter weight).

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

// Sync exports
#[cfg(not(feature = "async"))]
pub use flow::{
    Flow, FlowBackend, FlowBuilder, FlowConfig, FlowExecutionResult, errors::FlowError,
    route::Route,
};

// Async exports
#[cfg(feature = "async")]
pub use flow::{
    FlowConfig, FlowExecutionResult,
    r#async::{Flow, FlowBackend, FlowBuilder},
    errors::FlowError,
    route::Route,
};

/// Node execution system and traits
pub mod node;

// Sync Node exports
#[cfg(not(feature = "async"))]
pub use node::{ExecutionContext, Node, NodeError};

// Async Node exports
#[cfg(feature = "async")]
pub use node::{ExecutionContext, NodeError, r#async::Node};

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
    // Core types (always available)
    pub use crate::{Action, ExecutionContext, Node, NodeError, SharedStore};

    // Flow types (always available)
    pub use crate::{Flow, FlowBackend, FlowBuilder, FlowConfig, FlowExecutionResult};

    // Re-export async_trait when async feature is enabled
    #[cfg(feature = "async")]
    pub use async_trait::async_trait;

    // Storage backends
    #[cfg(feature = "storage-memory")]
    pub use crate::shared_store::backends::MemoryStorage;

    #[cfg(feature = "storage-file")]
    pub use crate::shared_store::backends::FileStorage;

    #[cfg(feature = "storage-redis")]
    pub use crate::shared_store::backends::RedisStorage;
}
