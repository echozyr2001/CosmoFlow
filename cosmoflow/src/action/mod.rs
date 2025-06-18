#![deny(missing_docs)]
//! # CosmoFlow Action (Simplified)
//!
//! This module provides a simplified action system for controlling workflow
//! execution in the CosmoFlow engine. The design has been streamlined based
//! on usage analysis to focus on the most commonly used patterns.
//!
//! ## Key Simplifications
//!
//! - **Reduced Variants**: From 6 to 3 Action variants (Simple, Parameterized, Conditional)
//! - **Eliminated Complexity**: Removed Prioritized, WithMetadata, and Multiple variants
//! - **Simplified Conditions**: Replaced complex ActionCondition system with simple key-value comparisons
//! - **Performance Optimized**: Removed recursive operations and complex nested structures
//!
//! ## Usage Statistics (Based on Analysis)
//!
//! - **Simple Actions**: 53.3% of usage - Basic string-based routing
//! - **Parameterized Actions**: 1.0% of usage - Actions with additional data
//! - **Conditional Actions**: 0.3% of usage - Dynamic routing based on conditions
//!
//! ## Examples
//!
//! ### Simple Actions (Most Common)
//! ```rust
//! use cosmoflow::action::Action;
//!
//! let action = Action::simple("next_step");
//! ```
//!
//! ### Parameterized Actions
//! ```rust
//! use cosmoflow::action::Action;
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! let mut params = HashMap::new();
//! params.insert("retry_count".to_string(), json!(3));
//! let action = Action::with_params("retry", params);
//! ```
//!
//! ### Conditional Actions (Simplified)
//! ```rust
//! use cosmoflow::action::Action;
//! use serde_json::json;
//!
//! let action = Action::conditional("status", json!("ready"), "proceed", "wait");
//! ```
//!
//! For more information, please see the main [`cosmoflow`](https://docs.rs/cosmoflow)
//! crate documentation.

/// The simplified action module defines the `Action` enum and its variants.
pub mod action_core;

/// Test module for the simplified action system.
#[cfg(test)]
pub mod tests;

pub use action_core::Action;
