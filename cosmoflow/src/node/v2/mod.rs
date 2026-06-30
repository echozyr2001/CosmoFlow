//! Experimental v2 node API.
//!
//! This module keeps the node core deliberately small: a node prepares input,
//! executes once, writes post-processing state, and returns an action. It does
//! not include retry, fallback, timeout, agent, tool, or LLM concepts.

/// Minimal node execution context.
pub mod context;
/// Phase-aware node errors.
pub mod error;
/// Node execution phases.
pub mod phase;

#[cfg(feature = "async")]
/// Async v2 node execution API.
pub mod r#async;
#[cfg(not(feature = "async"))]
/// Sync v2 node execution API.
pub mod sync;

pub use context::{ExecutionId, NodeContext, NodeId};
pub use error::NodeError;
pub use phase::NodePhase;

#[cfg(feature = "async")]
pub use r#async::Node;
#[cfg(not(feature = "async"))]
pub use sync::Node;
