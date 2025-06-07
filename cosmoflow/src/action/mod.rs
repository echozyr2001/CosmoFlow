#![deny(missing_docs)]
//! # CosmoFlow Action
//!
//! This module provides the action and condition types used to control workflow
//! execution in the CosmoFlow engine.
//!
//! It defines the [`Action`] and [`ActionCondition`] enums, which allow nodes
//! to return control flow decisions to the engine. This module is a low-level
//! component of CosmoFlow and is not intended to be used directly in most
//! applications.
//!
//! For more information, please see the main [`cosmoflow`](https://docs.rs/cosmoflow)
//! crate documentation.

/// The action module defines the `Action` enum and its variants.
pub mod action_core;
/// The condition module defines the `ActionCondition` enum and its variants.
pub mod condition;

pub use action_core::Action;
pub use condition::{ActionCondition, ComparisonOperator};
