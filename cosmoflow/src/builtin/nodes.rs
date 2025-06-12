//! Complete Node constructors for CosmoFlow workflows
//!
//! This module provides high-level Node constructors that create ready-to-use
//! Node instances for common workflow operations. These functions wrap the basic
//! Node implementations with sensible defaults and provide a convenient API
//! for quick workflow construction.
//!
//! # Purpose
//!
//! While the [`crate::basic`] module provides Node implementations that define
//! the core behavior, this module provides convenience functions that create
//! Node instances with:
//! - Sensible default configurations
//! - Common action patterns (typically "continue")
//! - Type-safe storage backend integration
//! - Minimal boilerplate for common use cases
//!
//! # Generic Functions
//!
//! The [`generic`] module provides node constructors that work with any storage
//! backend, making them suitable for custom storage implementations or when
//! storage backend selection needs to be deferred.
//!
//! # Examples
//!
//! ## Basic Node Creation
//!
//! ```rust
//! use cosmoflow::builtin::nodes::generic;
//! use cosmoflow::storage::MemoryStorage;
//! use std::time::Duration;
//! use serde_json::json;
//!
//! // Create nodes with explicit storage type
//! let log_node = generic::log_node::<MemoryStorage>("Processing started");
//! let delay_node = generic::delay_node::<MemoryStorage>(Duration::from_secs(1));
//! let set_value = generic::set_value_node::<MemoryStorage>("status", json!("ready"));
//! ```
//!
//! ## Workflow Integration
//!
//! ```rust
//! use cosmoflow::builtin::nodes::generic;
//! use serde_json::json;
//! use cosmoflow::storage::MemoryStorage;
//!
//! // Create nodes for workflow
//! let start_node = generic::log_node::<MemoryStorage>("Workflow started");
//! let set_data = generic::set_value_node::<MemoryStorage>("counter", json!(0));
//! let get_data = generic::get_value_node::<MemoryStorage>("counter", "current_value");
//!
//! // Nodes implement Node<MemoryStorage> and can be used in flows
//! println!("Created {} nodes for workflow", 3);
//! ```

use std::time::Duration;

use serde_json::Value;

use super::basic::*;

/// Generic node creation functions for custom storage backends
///
/// This module provides node constructors that work with any storage backend
/// implementing the [`storage::StorageBackend`] trait. These functions are useful when:
/// - Working with custom storage implementations
/// - Building reusable workflow components
/// - Deferring storage backend selection to runtime
///
/// All functions return [`Node`] instances that wrap the corresponding
/// node implementations with full execution capabilities.
pub mod generic {
    use crate::storage::StorageBackend;

    use super::*;

    /// Create a log node with a custom storage backend
    ///
    /// Creates a Node that logs a message during execution and continues
    /// to the next step. This is useful for debugging, monitoring, and
    /// providing user feedback during workflow execution.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to log during execution
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::nodes::generic;
    /// use cosmoflow::storage::MemoryStorage;
    ///
    /// let checkpoint = generic::log_node::<MemoryStorage>("Reached checkpoint 1");
    /// let status = generic::log_node::<MemoryStorage>("Processing completed");
    /// ```
    pub fn log_node<S: StorageBackend>(message: impl Into<String>) -> LogNode {
        log(message)
    }

    /// Create a set value node with a custom storage backend
    ///
    /// Creates a Node that stores a value in the shared store under the specified key.
    /// This is one of the most commonly used nodes for data persistence and sharing
    /// between workflow steps.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to store the value under in the shared store
    /// * `value` - The JSON value to store (can be any serializable type)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::nodes::generic;
    /// use cosmoflow::storage::MemoryStorage;
    /// use serde_json::json;
    ///
    /// // Store simple values
    /// let counter = generic::set_value_node::<MemoryStorage>("counter", json!(0));
    /// let name = generic::set_value_node::<MemoryStorage>("user_name", json!("Alice"));
    ///
    /// // Store complex data structures
    /// let config = generic::set_value_node::<MemoryStorage>("config", json!({
    ///     "timeout": 30,
    ///     "retries": 3,
    ///     "endpoints": ["api1.example.com", "api2.example.com"]
    /// }));
    /// ```
    pub fn set_value_node<S: StorageBackend>(key: impl Into<String>, value: Value) -> SetValueNode {
        set_value(key, value)
    }

    /// Create a delay node with a custom storage backend
    ///
    /// Creates a Node that introduces a delay in workflow execution. This is useful
    /// for rate limiting, implementing retry delays, or coordinating with external
    /// systems that have timing requirements.
    ///
    /// # Arguments
    ///
    /// * `duration` - The amount of time to delay execution
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::nodes::generic;
    /// use cosmoflow::storage::MemoryStorage;
    /// use std::time::Duration;
    ///
    /// // Short delays for rate limiting
    /// let rate_limit = generic::delay_node::<MemoryStorage>(Duration::from_millis(100));
    ///
    /// // Longer delays for retry backoff
    /// let retry_delay = generic::delay_node::<MemoryStorage>(Duration::from_secs(5));
    ///
    /// // Custom timing for external system coordination
    /// let sync_delay = generic::delay_node::<MemoryStorage>(Duration::from_secs(30));
    /// ```
    pub fn delay_node<S: StorageBackend>(duration: Duration) -> DelayNode {
        delay(duration)
    }

    /// Create a get value node with a custom storage backend
    ///
    /// Creates a Node that retrieves a value from the shared store and optionally
    /// stores it under a different key. This node provides the basic form that
    /// copies values without transformation. For more complex transformations,
    /// use the backend directly with custom transformation functions.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value from
    /// * `output_key` - The key to store the retrieved value under
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::nodes::generic;
    /// use cosmoflow::storage::MemoryStorage;
    ///
    /// // Copy values between keys
    /// let copy_user_data = generic::get_value_node::<MemoryStorage>(
    ///     "temp_user_data",
    ///     "user_data"
    /// );
    ///
    /// // Retrieve configuration for processing
    /// let get_config = generic::get_value_node::<MemoryStorage>(
    ///     "workflow_config",
    ///     "current_config"
    /// );
    ///
    /// // Move data through processing stages
    /// let stage_data = generic::get_value_node::<MemoryStorage>(
    ///     "raw_input",
    ///     "processing_input"
    /// );
    /// ```
    ///
    /// # Note
    ///
    /// This function creates a simple copy operation. For data transformation,
    /// validation, or complex processing, create a `GetValueNode` directly
    /// with a custom transformation function.
    pub fn get_value_node<S: StorageBackend>(
        key: impl Into<String>,
        output_key: impl Into<String>,
    ) -> GetValueNode<impl Fn(Option<Value>) -> Value + Send + Sync> {
        get_value(key, output_key)
    }

    /// Create a conditional node with a custom storage backend
    ///
    /// Creates a Node that executes conditional logic based on a custom function.
    /// The condition function receives access to the shared store and returns a
    /// boolean to determine which action to take.
    ///
    /// # Arguments
    ///
    /// * `condition` - A function that evaluates the condition based on store contents
    /// * `if_true` - The action to take when the condition evaluates to true
    /// * `if_false` - The action to take when the condition evaluates to false
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::nodes::generic;
    /// use cosmoflow::storage::MemoryStorage;
    /// use cosmoflow::action::Action;
    ///
    /// // Simple value comparison
    /// let score_check = generic::conditional_node::<_, MemoryStorage>(
    ///     |store| {
    ///         store.get("score")
    ///             .ok()
    ///             .flatten()
    ///             .and_then(|v: serde_json::Value| v.as_i64())
    ///             .map(|score| score > 80)
    ///             .unwrap_or(false)
    ///     },
    ///     Action::simple("high_score_path"),
    ///     Action::simple("low_score_path")
    /// );
    ///
    /// // Complex data validation
    /// let data_validation = generic::conditional_node::<_, MemoryStorage>(
    ///     |store| {
    ///         if let Ok(Some(data)) = store.get::<serde_json::Value>("user_data") {
    ///             data.as_object()
    ///                 .map(|obj| obj.contains_key("required_field") &&
    ///                            obj["required_field"].as_str().is_some())
    ///                 .unwrap_or(false)
    ///         } else {
    ///             false
    ///         }
    ///     },
    ///     Action::simple("process_data"),
    ///     Action::simple("request_missing_data")
    /// );
    ///
    /// // System state checks
    /// let system_ready = generic::conditional_node::<_, MemoryStorage>(
    ///     |store| {
    ///         let connections = store.get("active_connections")
    ///             .ok()
    ///             .flatten()
    ///             .and_then(|v: serde_json::Value| v.as_i64())
    ///             .unwrap_or(0);
    ///         let memory_usage = store.get("memory_usage")
    ///             .ok()
    ///             .flatten()
    ///             .and_then(|v: serde_json::Value| v.as_f64())
    ///             .unwrap_or(100.0);
    ///         connections > 0 && memory_usage < 80.0
    ///     },
    ///     Action::simple("proceed"),
    ///     Action::simple("wait_for_resources")
    /// );
    /// ```
    ///
    /// # Type Parameters
    ///
    /// * `F` - The condition function type, must be `Fn(&SharedStore<S>) -> bool + Send + Sync`
    /// * `S` - The storage backend type
    pub fn conditional_node<F, S: StorageBackend>(
        condition: F,
        if_true: crate::action::Action,
        if_false: crate::action::Action,
    ) -> ConditionalNode<F, S>
    where
        F: Fn(&crate::shared_store::SharedStore<S>) -> bool + Send + Sync,
    {
        ConditionalNode::new(condition, if_true, if_false)
    }
}
