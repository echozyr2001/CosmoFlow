#![deny(missing_docs)]
//! # CosmoFlow Action (Streamlined)
//!
//! This module provides a streamlined action system for controlling workflow
//! execution in the CosmoFlow engine. The design focuses on essential functionality
//! while eliminating redundant and rarely-used features.
//!
//! ## Key Features
//!
//! - **Consistent Naming**: Standardized `is_*` and `has_*` method patterns
//! - **Convenience Methods**: Reduced boilerplate with `with_param()` for single parameters
//! - **Essential Functionality**: Focused API with only the most commonly used methods
//! - **Clean Design**: Removed redundant aliases and utility methods
//!
//! ## Core API
//!
//! ### Creation Methods
//! - `simple(name)` - Create simple action
//! - `with_params(name, params)` - Create action with multiple parameters
//! - `with_param(name, key, value)` - Convenience method for single parameter
//!
//! ### Type Checking
//! - `is_simple()` - Check if action is simple
//! - `is_parameterized()` - Check if action has parameters
//!
//! ### Parameter Access
//! - `params()` - Get all parameters
//! - `get_param(key)` - Get specific parameter value
//! - `has_param(key)` - Check if specific parameter exists
//! - `param_count()` - Get number of parameters
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
//! assert!(action.is_simple());
//! ```
//!
//! ### Single Parameter Actions (Convenience)
//! ```rust
//! use cosmoflow::action::Action;
//! use serde_json::json;
//!
//! let action = Action::with_param("retry", "count", json!(3));
//! assert!(action.has_param("count"));
//! assert_eq!(action.get_param("count"), Some(&json!(3)));
//! ```
//!
//! ### Multiple Parameter Actions
//! ```rust
//! use cosmoflow::action::Action;
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! let mut params = HashMap::new();
//! params.insert("timeout".to_string(), json!(30));
//! params.insert("retries".to_string(), json!(3));
//! let action = Action::with_params("process", params);
//! assert_eq!(action.param_count(), 2);
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
