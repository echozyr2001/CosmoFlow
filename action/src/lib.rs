//! Action system for CosmoFlow workflows
//!
//! This crate provides types and functionality for representing and evaluating
//! actions that control flow between nodes in CosmoFlow workflows.
//!
//! # Core Types
//!
//! - [`Action`] - Represents various types of actions (simple, parameterized, conditional, etc.)
//! - [`ActionCondition`] - Represents conditions for conditional actions
//! - [`ComparisonOperator`] - Operators for numeric comparisons in conditions
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```
//! use action::Action;
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! // Create a simple action
//! let action = Action::simple("next_step");
//!
//! // Create a parameterized action
//! let mut params = HashMap::new();
//! params.insert("retry_count".to_string(), json!(3));
//! let param_action = Action::with_params("retry", params);
//!
//! // Create a conditional action
//! use action::ActionCondition;
//! let condition = ActionCondition::key_equals("status", json!("ready"));
//! let conditional = Action::conditional(
//!     condition,
//!     Action::simple("proceed"),
//!     Action::simple("wait")
//! );
//! ```

pub mod action;
pub mod condition;

pub use action::Action;
pub use condition::{ActionCondition, ComparisonOperator};
