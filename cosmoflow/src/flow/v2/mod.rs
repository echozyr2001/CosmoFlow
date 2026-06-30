//! Experimental v2 flow API.
//!
//! Flow v2 models a flow as a small state-machine graph over v2 nodes and
//! v2 actions. It validates graph structure at build time and executes nodes
//! sequentially until an action has no matching route.

mod analysis;
mod error;
mod execution;
mod route;

#[cfg(feature = "async")]
mod r#async;
#[cfg(not(feature = "async"))]
mod sync;

pub use analysis::FlowAnalysis;
pub use error::FlowError;
pub use execution::FlowExecution;
pub use route::Route;

#[cfg(feature = "async")]
pub use r#async::{Flow, FlowBuilder};
#[cfg(not(feature = "async"))]
pub use sync::{Flow, FlowBuilder};
