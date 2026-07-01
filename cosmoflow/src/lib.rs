#![deny(missing_docs)]
//! # CosmoFlow
//!
//! A lightweight framework for modeling workflows as state machines.
//!
//! CosmoFlow core follows two principles:
//!
//! *   Every program can be modeled as a state machine.
//! *   The framework core should stay small; retry, fallback, timeout, LLM/tool
//!     integration, memory, and tracing belong in user composition or optional
//!     extensions.
//!
//! ## Core Concepts
//!
//! *   **Action**: A state transition signal. Its name is the routing identity,
//!     and parameters are optional carried data.
//! *   **Node**: A user-defined `prep -> exec -> post` behavior unit.
//! *   **Flow**: A state-machine graph with build-time validation and a single
//!     sequential executor.
//! *   **State**: The runtime state value `S` passed through node and flow
//!     execution. It may be a strong typed struct or a shared store.
//! *   **Shared Store**: An optional key-value state model with memory, file, and
//!     Redis backends.
//!
//! ## API Promotion Note
//!
//! The current core model is available under the `action::v2`, `node::v2`, and
//! `flow::v2` modules. The crate root and legacy module exports are intentionally
//! not changed in this documentation-only pass.
//!
//! # Quick Start
//!
//! This example shows the intended main API after the current core model is
//! promoted from the v2 modules.
//!
//! ```rust,ignore
//! use cosmoflow::action::Action;
//! use cosmoflow::flow::FlowBuilder;
//! use cosmoflow::node::{Node, NodeContext};
//!
//! #[derive(Default)]
//! struct AppState {
//!     visits: Vec<String>,
//! }
//!
//! struct MyNode;
//!
//! impl Node<AppState> for MyNode {
//!     type Prep = ();
//!     type Output = ();
//!     type Error = std::convert::Infallible;
//!
//!     fn prep(&mut self, _state: &AppState, _ctx: &NodeContext) -> Result<Self::Prep, Self::Error> {
//!         Ok(())
//!     }
//!
//!     fn exec(&mut self, _prep: &Self::Prep, _ctx: &NodeContext) -> Result<Self::Output, Self::Error> {
//!         Ok(())
//!     }
//!
//!     fn post(
//!         &mut self,
//!         state: &mut AppState,
//!         _prep: Self::Prep,
//!         _output: Self::Output,
//!         ctx: &NodeContext,
//!     ) -> Result<Action, Self::Error> {
//!         state.visits.push(ctx.node_id.as_str().to_string());
//!         Ok(Action::new("done"))
//!     }
//! }
//!
//! let mut flow = FlowBuilder::new()
//!     .node("start", MyNode)
//!     .build()?;
//!
//! let mut state = AppState::default();
//! let action = flow.run(&mut state)?;
//! assert_eq!(action.as_str(), "done");
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
//! *   `minimal`: Core engine only (bring your own storage).
//! *   `basic`: Basic usable configuration with memory storage.
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

/// Optional key-value state model for dynamic workflow context.
pub mod shared_store;
pub use shared_store::SharedStore;

/// Action types for workflow transition signals.
pub mod action;
pub use action::Action;

/// Flow graph definition and execution.
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

/// Node execution traits and context types.
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
