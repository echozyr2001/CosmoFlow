#![deny(missing_docs)]
//! Flow API.
//!
//! A flow is a small state-machine graph over nodes and actions. It validates
//! graph structure at build time and executes nodes sequentially until an action
//! has no matching route.

mod analysis;
mod error;
mod execution;
/// Declarative construction macro for flows.
pub mod macros;
mod route;

#[cfg(feature = "async")]
mod r#async;
#[cfg(not(feature = "async"))]
mod sync;

pub use analysis::FlowAnalysis;
pub use error::FlowError;
pub use execution::FlowExecution;
pub use macros::flow;
pub use route::Route;

#[cfg(feature = "async")]
pub use r#async::{Flow, FlowBuilder};
#[cfg(not(feature = "async"))]
pub use sync::{Flow, FlowBuilder};
