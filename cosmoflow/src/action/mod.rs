#![deny(missing_docs)]
//! Action APIs.
//!
//! The core action model treats an action as a state transition signal: the
//! action name is the routing identity, and parameters are optional carried data.
//!
//! The current minimal core model is available in `v2`. The legacy `Action`
//! type remains exported from this module until
//! the core API is promoted to the main module surface.

/// The simplified action module defines the `Action` enum and its variants.
pub mod action_core;

/// Test module for the simplified action system.
#[cfg(test)]
pub mod tests;

/// Action core API under active promotion to the main module surface.
pub mod v2;

pub use action_core::Action;
