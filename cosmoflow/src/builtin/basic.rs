//! Basic node implementations for common workflow operations.
//!
//! This module provides fundamental node types that handle common workflow tasks
//! such as logging, data manipulation, and basic control flow. These nodes serve
//! as building blocks for more complex workflows and demonstrate best practices
//! for node implementation.
//!
//! # Available Nodes
//!
//! - [`LogNodeBackend`] - Logs messages during workflow execution
//! - [`SetValueNodeBackend`] - Sets values in the shared store
//! - [`GetValueNodeBackend`] - Retrieves and transforms values from the shared store
//! - [`ConditionalNodeBackend`] - Provides conditional routing based on store values
//! - [`DelayNodeBackend`] - Introduces delays for timing control
//! - [`DelayNodeBackend`] - Introduces delays for timing control
//!
//! # Examples
//!
//! ## Creating a Simple Log Node
//!
//! ```rust
//! use cosmoflow::builtin::basic::LogNodeBackend;
//! use cosmoflow::action::Action;
//!
//! let log_node = LogNodeBackend::new("Processing started", Action::simple("next"));
//! ```
//!
//! ## Data Manipulation Workflow
//!
//! ```rust
//! use cosmoflow::builtin::basic::{SetValueNodeBackend, GetValueNodeBackend};
//! use cosmoflow::action::Action;
//! use serde_json::json;
//!
//! // Set initial data
//! let set_node = SetValueNodeBackend::new(
//!     "user_count",
//!     json!(100),
//!     Action::simple("increment")
//! );
//!
//! // Transform data
//! let transform_node = GetValueNodeBackend::new(
//!     "user_count",
//!     "user_count_doubled",
//!     |value| match value {
//!         Some(v) if v.is_number() => json!(v.as_f64().unwrap_or(0.0) * 2.0),
//!         _ => json!(0),
//!     },
//!     Action::simple("complete")
//! );
//! ```

use std::time::Duration;

use crate::action::Action;
use crate::node::{ExecutionContext, NodeBackend, NodeError};
use crate::shared_store::SharedStore;
use crate::storage::StorageBackend;
use async_trait::async_trait;
use serde_json::Value;

/// A simple node that logs messages and passes through
///
/// The LogNodeBackend provides basic logging functionality for workflows,
/// allowing you to output messages at specific points in the execution flow.
/// This is particularly useful for debugging, monitoring, and providing
/// user feedback during long-running processes.
///
/// # Features
///
/// - Configurable log messages with execution context
/// - Retry support with customizable delays
/// - Pass-through behavior (doesn't modify data flow)
/// - Minimal performance overhead
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use cosmoflow::builtin::basic::LogNodeBackend;
/// use cosmoflow::action::Action;
///
/// let node = LogNodeBackend::new("Starting data processing", Action::simple("process"));
/// ```
///
/// ## With Retry Configuration
///
/// ```rust
/// use cosmoflow::builtin::basic::LogNodeBackend;
/// use cosmoflow::action::Action;
/// use std::time::Duration;
///
/// let node = LogNodeBackend::new("Critical checkpoint", Action::simple("continue"))
///     .with_retries(3)
///     .with_retry_delay(Duration::from_secs(1));
/// ```
pub struct LogNodeBackend {
    message: String,
    action: Action,
    max_retries: usize,
    retry_delay: Duration,
}

impl LogNodeBackend {
    /// Create a new log node
    ///
    /// Creates a LogNodeBackend that will output the specified message during
    /// execution and then route to the given action.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to log during execution
    /// * `action` - The action to return after logging
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::basic::LogNodeBackend;
    /// use cosmoflow::action::Action;
    ///
    /// let node = LogNodeBackend::new("Processing complete", Action::simple("finish"));
    /// ```
    pub fn new<S: Into<String>>(message: S, action: Action) -> Self {
        Self {
            message: message.into(),
            action,
            max_retries: 1,
            retry_delay: Duration::ZERO,
        }
    }

    /// Set maximum retries
    ///
    /// Configures how many times this node should be retried if it fails.
    /// This is useful for nodes that might fail due to temporary conditions.
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of retry attempts (minimum 1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::basic::LogNodeBackend;
    /// use cosmoflow::action::Action;
    ///
    /// let node = LogNodeBackend::new("Unreliable operation", Action::simple("next"))
    ///     .with_retries(5);
    /// ```
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set retry delay
    ///
    /// Configures the delay between retry attempts. This helps avoid
    /// overwhelming systems and provides backoff behavior for failed operations.
    ///
    /// # Arguments
    ///
    /// * `delay` - Duration to wait between retry attempts
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::basic::LogNodeBackend;
    /// use cosmoflow::action::Action;
    /// use std::time::Duration;
    ///
    /// let node = LogNodeBackend::new("Network operation", Action::simple("next"))
    ///     .with_retries(3)
    ///     .with_retry_delay(Duration::from_millis(500));
    /// ```
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> NodeBackend<S> for LogNodeBackend {
    type PrepResult = String;
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(format!(
            "Execution {}: {}",
            context.execution_id, &self.message
        ))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("{}", prep_result);
        Ok(prep_result)
    }

    async fn post(
        &mut self,
        _store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "LogNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }

    fn retry_delay(&self) -> Duration {
        self.retry_delay
    }
}

/// A node that sets a value in the shared store
///
/// The SetValueNodeBackend allows workflows to store data in the shared store
/// for use by subsequent nodes. This is essential for passing data between
/// workflow steps and maintaining state across the execution flow.
///
/// # Features
///
/// - Store any JSON-serializable value
/// - Atomic operations (value is set during post-processing)
/// - Error handling for storage failures
/// - Configurable retry behavior
///
/// # Examples
///
/// ## Setting Simple Values
///
/// ```rust
/// use cosmoflow::builtin::basic::SetValueNodeBackend;
/// use cosmoflow::action::Action;
/// use serde_json::json;
///
/// // Set a user ID
/// let set_user = SetValueNodeBackend::new(
///     "current_user_id",
///     json!(12345),
///     Action::simple("load_user_data")
/// );
///
/// // Set configuration
/// let set_config = SetValueNodeBackend::new(
///     "config",
///     json!({
///         "max_connections": 100,
///         "timeout": 30,
///         "debug": true
///     }),
///     Action::simple("start_service")
/// );
/// ```
///
/// ## With Error Handling
///
/// ```rust
/// use cosmoflow::builtin::basic::SetValueNodeBackend;
/// use cosmoflow::action::Action;
/// use serde_json::json;
///
/// let node = SetValueNodeBackend::new(
///     "critical_data",
///     json!("important_value"),
///     Action::simple("continue")
/// ).with_retries(3); // Retry up to 3 times if storage fails
/// ```
pub struct SetValueNodeBackend {
    key: String,
    value: Value,
    action: Action,
    max_retries: usize,
}

impl SetValueNodeBackend {
    /// Create a new set value node
    ///
    /// Creates a SetValueNodeBackend that will store the specified key-value
    /// pair in the shared store during the post-processing phase.
    ///
    /// # Arguments
    ///
    /// * `key` - The key under which to store the value
    /// * `value` - The value to store (must be JSON-serializable)
    /// * `action` - The action to return after setting the value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::basic::SetValueNodeBackend;
    /// use cosmoflow::action::Action;
    /// use serde_json::json;
    ///
    /// let node = SetValueNodeBackend::new(
    ///     "process_status",
    ///     json!("completed"),
    ///     Action::simple("cleanup")
    /// );
    /// ```
    pub fn new<S: Into<String>>(key: S, value: Value, action: Action) -> Self {
        Self {
            key: key.into(),
            value,
            action,
            max_retries: 1,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> NodeBackend<S> for SetValueNodeBackend {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Ok(())
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        store
            .set(self.key.clone(), self.value.clone())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "SetValueNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// A node that gets a value from the shared store and optionally transforms it
///
/// The GetValueNodeBackend retrieves data from the shared store and applies
/// an optional transformation function before storing the result under a new key.
/// This is useful for data processing, format conversion, and computation steps
/// within workflows.
///
/// # Type Parameters
///
/// * `F` - A function that takes an `Option<Value>` and returns a `Value`.
///   This function is applied to transform the retrieved data.
///
/// # Features
///
/// - Retrieve values by key from the shared store
/// - Apply custom transformation functions
/// - Handle missing values gracefully
/// - Store transformed results under different keys
/// - Configurable retry behavior
///
/// # Examples
///
/// ## Simple Value Retrieval
///
/// ```rust
/// use cosmoflow::builtin::basic::GetValueNodeBackend;
/// use cosmoflow::action::Action;
/// use serde_json::{json, Value};
///
/// // Pass through values unchanged
/// let pass_through = GetValueNodeBackend::new(
///     "input_data",
///     "output_data",
///     |value| value.unwrap_or(json!(null)),
///     Action::simple("continue")
/// );
/// ```
///
/// ## Data Transformation
///
/// ```rust
/// use cosmoflow::builtin::basic::GetValueNodeBackend;
/// use cosmoflow::action::Action;
/// use serde_json::{json, Value};
///
/// // Double numeric values
/// let doubler = GetValueNodeBackend::new(
///     "number",
///     "doubled_number",
///     |value| match value {
///         Some(v) if v.is_number() => {
///             json!(v.as_f64().unwrap_or(0.0) * 2.0)
///         },
///         _ => json!(0),
///     },
///     Action::simple("next")
/// );
///
/// // Convert to uppercase string
/// let uppercase = GetValueNodeBackend::new(
///     "text",
///     "uppercase_text",
///     |value| match value {
///         Some(v) if v.is_string() => {
///             json!(v.as_str().unwrap_or("").to_uppercase())
///         },
///         _ => json!(""),
///     },
///     Action::simple("format_complete")
/// );
/// ```
///
/// ## Complex Data Processing
///
/// ```rust
/// use cosmoflow::builtin::basic::GetValueNodeBackend;
/// use cosmoflow::action::Action;
/// use serde_json::{json, Value};
///
/// // Extract and process array data
/// let array_processor = GetValueNodeBackend::new(
///     "user_list",
///     "active_users",
///     |value| {
///         match value {
///             Some(Value::Array(users)) => {
///                 let active: Vec<_> = users.into_iter()
///                     .filter(|user| user.get("active").and_then(Value::as_bool).unwrap_or(false))
///                     .collect();
///                 json!(active)
///             },
///             _ => json!([]),
///         }
///     },
///     Action::simple("process_users")
/// );
/// ```
pub struct GetValueNodeBackend<F>
where
    F: Fn(Option<Value>) -> Value + Send + Sync,
{
    key: String,
    output_key: String,
    transform: F,
    action: Action,
    max_retries: usize,
}

impl<F> GetValueNodeBackend<F>
where
    F: Fn(Option<Value>) -> Value + Send + Sync,
{
    /// Create a new get value node
    ///
    /// Creates a GetValueNodeBackend that retrieves a value from the shared store,
    /// applies a transformation function, and stores the result under a new key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve from the shared store
    /// * `output_key` - The key under which to store the transformed result
    /// * `transform` - Function to transform the retrieved value
    /// * `action` - The action to return after processing
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::basic::GetValueNodeBackend;
    /// use cosmoflow::action::Action;
    /// use serde_json::{json, Value};
    ///
    /// // Create a node that increments a counter
    /// let increment_node = GetValueNodeBackend::new(
    ///     "counter",
    ///     "counter",
    ///     |value| {
    ///         let current = value.unwrap_or(json!(0));
    ///         json!(current.as_i64().unwrap_or(0) + 1)
    ///     },
    ///     Action::simple("continue")
    /// );
    /// ```
    pub fn new<S1: Into<String>, S2: Into<String>>(
        key: S1,
        output_key: S2,
        transform: F,
        action: Action,
    ) -> Self {
        Self {
            key: key.into(),
            output_key: output_key.into(),
            transform,
            action,
            max_retries: 1,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S, F> NodeBackend<S> for GetValueNodeBackend<F>
where
    S: StorageBackend + Send + Sync,
    F: Fn(Option<Value>) -> Value + Send + Sync,
{
    type PrepResult = Option<Value>;
    type ExecResult = Value;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        store
            .get(&self.key)
            .map_err(|e| NodeError::StorageError(e.to_string()))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Ok((self.transform)(prep_result))
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        store
            .set(self.output_key.clone(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "GetValueNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// A conditional node that chooses actions based on store content
///
/// The ConditionalNodeBackend evaluates a condition function against the
/// shared store and returns different actions based on the result. This
/// enables dynamic workflow routing based on runtime data.
///
/// # Type Parameters
///
/// * `F` - A function that takes a reference to the SharedStore and returns a boolean
/// * `S` - The storage backend type
///
/// # Features
///
/// - Dynamic action selection based on store state
/// - Custom condition functions for maximum flexibility
/// - Support for complex conditional logic
/// - Configurable retry behavior
///
/// # Examples
///
/// ## Simple Value Checks
///
/// ```rust
/// use cosmoflow::builtin::basic::ConditionalNodeBackend;
/// use cosmoflow::action::Action;
/// use serde_json::json;
/// use cosmoflow::storage::MemoryStorage;
///
/// let condition_node = ConditionalNodeBackend::<_, MemoryStorage>::new(
///     |store| {
///         store.get("user_authenticated")
///             .ok()
///             .flatten()
///             .and_then(|v: serde_json::Value| v.as_bool())
///             .unwrap_or(false)
///     },
///     Action::simple("dashboard"),     // If authenticated
///     Action::simple("login_page")     // If not authenticated
/// );
/// ```
///
/// ## Complex Conditions
///
/// ```rust
/// use cosmoflow::builtin::basic::ConditionalNodeBackend;
/// use cosmoflow::action::Action;
/// use serde_json::json;
/// use cosmoflow::storage::MemoryStorage;
///
/// let complex_condition = ConditionalNodeBackend::<_, MemoryStorage>::new(
///     |store| {
///         let user_count = store.get("active_users")
///             .ok()
///             .flatten()
///             .and_then(|v: serde_json::Value| v.as_i64())
///             .unwrap_or(0);
///         
///         let server_load = store.get("cpu_usage")
///             .ok()
///             .flatten()
///             .and_then(|v: serde_json::Value| v.as_f64())
///             .unwrap_or(0.0);
///         
///         user_count > 100 && server_load < 80.0
///     },
///     Action::simple("scale_up"),      // High usage, low load
///     Action::simple("maintain")       // Normal operation
/// );
/// ```
pub struct ConditionalNodeBackend<F, S>
where
    F: Fn(&SharedStore<S>) -> bool + Send + Sync,
    S: StorageBackend,
{
    condition: F,
    if_true: Action,
    if_false: Action,
    max_retries: usize,
    _phantom: std::marker::PhantomData<S>,
}

impl<F, S> ConditionalNodeBackend<F, S>
where
    F: Fn(&SharedStore<S>) -> bool + Send + Sync,
    S: StorageBackend,
{
    /// Create a new conditional node
    ///
    /// Creates a ConditionalNodeBackend that evaluates the provided condition
    /// function against the shared store and returns the appropriate action.
    ///
    /// # Arguments
    ///
    /// * `condition` - Function that evaluates the store and returns a boolean
    /// * `if_true` - Action to return when condition is true
    /// * `if_false` - Action to return when condition is false
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::basic::ConditionalNodeBackend;
    /// use cosmoflow::action::Action;
    /// use cosmoflow::storage::MemoryStorage;
    ///
    /// let node = ConditionalNodeBackend::<_, MemoryStorage>::new(
    ///     |store| store.get("ready").ok().flatten().and_then(|v: serde_json::Value| v.as_bool()).unwrap_or(false),
    ///     Action::simple("proceed"),
    ///     Action::simple("wait")
    /// );
    /// ```
    pub fn new(condition: F, if_true: Action, if_false: Action) -> Self {
        Self {
            condition,
            if_true,
            if_false,
            max_retries: 1,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S, F> NodeBackend<S> for ConditionalNodeBackend<F, S>
where
    S: StorageBackend + Send + Sync,
    F: Fn(&SharedStore<S>) -> bool + Send + Sync,
{
    type PrepResult = bool;
    type ExecResult = bool;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok((self.condition)(store))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Ok(prep_result)
    }

    async fn post(
        &mut self,
        _store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        if exec_result {
            Ok(self.if_true.clone())
        } else {
            Ok(self.if_false.clone())
        }
    }

    fn name(&self) -> &str {
        "ConditionalNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// A delay node that waits for a specified duration
///
/// The DelayNodeBackend introduces a pause in workflow execution for the
/// specified duration. This is useful for rate limiting, scheduling delays,
/// waiting for external systems, or implementing backoff strategies.
///
/// # Features
///
/// - Configurable delay duration
/// - Non-blocking async implementation
/// - Precise timing using tokio::time::sleep
/// - Configurable retry behavior
/// - Minimal resource usage during delay
///
/// # Examples
///
/// ## Basic Delay
///
/// ```rust
/// use cosmoflow::builtin::basic::DelayNodeBackend;
/// use cosmoflow::action::Action;
/// use std::time::Duration;
///
/// // Wait 5 seconds before continuing
/// let delay_node = DelayNodeBackend::new(
///     Duration::from_secs(5),
///     Action::simple("continue")
/// );
/// ```
///
/// ## Rate Limiting
///
/// ```rust
/// use cosmoflow::builtin::basic::DelayNodeBackend;
/// use cosmoflow::action::Action;
/// use std::time::Duration;
///
/// // Rate limit API calls to 1 per second
/// let rate_limit = DelayNodeBackend::new(
///     Duration::from_secs(1),
///     Action::simple("api_call")
/// ).with_retries(3);
/// ```
///
/// ## Backoff Strategy
///
/// ```rust
/// use cosmoflow::builtin::basic::DelayNodeBackend;
/// use cosmoflow::action::Action;
/// use std::time::Duration;
///
/// // Exponential backoff delay
/// let backoff_delay = DelayNodeBackend::new(
///     Duration::from_millis(500), // Base delay
///     Action::simple("retry_operation")
/// );
/// ```
pub struct DelayNodeBackend {
    duration: Duration,
    action: Action,
    max_retries: usize,
}

impl DelayNodeBackend {
    /// Create a new delay node
    ///
    /// Creates a DelayNodeBackend that will pause execution for the specified
    /// duration before returning the given action.
    ///
    /// # Arguments
    ///
    /// * `duration` - How long to wait before continuing
    /// * `action` - The action to return after the delay
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::builtin::basic::DelayNodeBackend;
    /// use cosmoflow::action::Action;
    /// use std::time::Duration;
    ///
    /// // Wait 30 seconds before proceeding
    /// let node = DelayNodeBackend::new(
    ///     Duration::from_secs(30),
    ///     Action::simple("timeout_complete")
    /// );
    /// ```
    pub fn new(duration: Duration, action: Action) -> Self {
        Self {
            duration,
            action,
            max_retries: 1,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> NodeBackend<S> for DelayNodeBackend {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        tokio::time::sleep(self.duration).await;
        Ok(())
    }

    async fn post(
        &mut self,
        _store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "DelayNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// Helper function to create a simple log node with default settings
///
/// Creates a LogNodeBackend with default configuration and a "continue" action.
/// This is a convenience function for quickly adding logging to workflows.
///
/// # Arguments
///
/// * `message` - The message to log
///
/// # Examples
///
/// ```rust
/// use cosmoflow::builtin::basic::log;
///
/// let checkpoint = log("Reached checkpoint 1");
/// let status = log("Processing completed successfully");
/// ```
pub fn log<S: Into<String>>(message: S) -> LogNodeBackend {
    LogNodeBackend::new(message, Action::simple("continue"))
}

/// Helper function to create a simple set value node
///
/// Creates a SetValueNodeBackend with default configuration and a "continue" action.
/// This is a convenience function for quickly setting values in workflows.
///
/// # Arguments
///
/// * `key` - The key under which to store the value
/// * `value` - The value to store
///
/// # Examples
///
/// ```rust
/// use cosmoflow::builtin::basic::set_value;
/// use serde_json::json;
///
/// let set_status = set_value("process_status", json!("started"));
/// let set_config = set_value("max_retries", json!(3));
/// ```
pub fn set_value<S: Into<String>>(key: S, value: Value) -> SetValueNodeBackend {
    SetValueNodeBackend::new(key, value, Action::simple("continue"))
}

/// Helper function to create a simple delay node
///
/// Creates a DelayNodeBackend with default configuration and a "continue" action.
/// This is a convenience function for quickly adding delays to workflows.
///
/// # Arguments
///
/// * `duration` - How long to wait
///
/// # Examples
///
/// ```rust
/// use cosmoflow::builtin::basic::delay;
/// use std::time::Duration;
///
/// let short_pause = delay(Duration::from_millis(100));
/// let long_pause = delay(Duration::from_secs(10));
/// ```
pub fn delay(duration: Duration) -> DelayNodeBackend {
    DelayNodeBackend::new(duration, Action::simple("continue"))
}

/// Helper function to create a get value node with identity transform
///
/// Creates a GetValueNodeBackend that copies a value from one key to another
/// without modification. This is useful for renaming keys or creating backups
/// of values in the shared store.
///
/// # Arguments
///
/// * `key` - The key to retrieve from
/// * `output_key` - The key to store the result under
///
/// # Examples
///
/// ```rust
/// use cosmoflow::builtin::basic::get_value;
///
/// // Copy a value to a new key
/// let copy_value = get_value("original_data", "backup_data");
///
/// // Rename a value (use with subsequent delete of original)
/// let rename_value = get_value("temp_result", "final_result");
/// ```
pub fn get_value<S1: Into<String>, S2: Into<String>>(
    key: S1,
    output_key: S2,
) -> GetValueNodeBackend<impl Fn(Option<Value>) -> Value + Send + Sync> {
    GetValueNodeBackend::new(
        key,
        output_key,
        |value| value.unwrap_or(Value::Null),
        Action::simple("continue"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_store::SharedStore;
    use crate::storage::MemoryStorage;
    use serde_json::json;

    #[tokio::test]
    async fn test_optimized_log_node() {
        let mut node: LogNodeBackend = log("test message");
        let store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        let context = ExecutionContext::new(1, Duration::ZERO);

        let prep_result =
            <LogNodeBackend as NodeBackend<MemoryStorage>>::prep(&mut node, &store, &context)
                .await
                .unwrap();
        assert!(prep_result.contains("test message"));

        let exec_result = <LogNodeBackend as NodeBackend<MemoryStorage>>::exec(
            &mut node,
            prep_result.clone(),
            &context,
        )
        .await
        .unwrap();
        assert_eq!(exec_result, prep_result);
    }

    #[tokio::test]
    async fn test_optimized_set_value_node() {
        let mut node: SetValueNodeBackend = set_value("test_key", json!("test_value"));
        let mut store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        let context = ExecutionContext::new(1, Duration::ZERO);

        <SetValueNodeBackend as NodeBackend<MemoryStorage>>::prep(&mut node, &store, &context)
            .await
            .unwrap();
        <SetValueNodeBackend as NodeBackend<MemoryStorage>>::exec(&mut node, (), &context)
            .await
            .unwrap();
        let action = <SetValueNodeBackend as NodeBackend<MemoryStorage>>::post(
            &mut node,
            &mut store,
            (),
            (),
            &context,
        )
        .await
        .unwrap();

        assert_eq!(action, Action::simple("continue"));
        assert_eq!(store.get("test_key").unwrap(), Some(json!("test_value")));
    }

    #[tokio::test]
    async fn test_optimized_delay_node() {
        let mut node: DelayNodeBackend = delay(Duration::from_millis(1));
        let mut store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        let context = ExecutionContext::new(1, Duration::ZERO);

        let start = std::time::Instant::now();
        <DelayNodeBackend as NodeBackend<MemoryStorage>>::prep(&mut node, &store, &context)
            .await
            .unwrap();
        <DelayNodeBackend as NodeBackend<MemoryStorage>>::exec(&mut node, (), &context)
            .await
            .unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(1));
        let action = <DelayNodeBackend as NodeBackend<MemoryStorage>>::post(
            &mut node,
            &mut store,
            (),
            (),
            &context,
        )
        .await
        .unwrap();
        assert_eq!(action, Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_optimized_get_value_node() {
        let mut store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        store
            .set("input_key".to_string(), json!("input_value"))
            .unwrap();

        let mut node = get_value("input_key", "output_key");
        let context = ExecutionContext::new(1, Duration::ZERO);

        let prep_result = <GetValueNodeBackend<_> as NodeBackend<MemoryStorage>>::prep(
            &mut node, &store, &context,
        )
        .await
        .unwrap();
        assert_eq!(prep_result, Some(json!("input_value")));

        let exec_result = <GetValueNodeBackend<_> as NodeBackend<MemoryStorage>>::exec(
            &mut node,
            prep_result,
            &context,
        )
        .await
        .unwrap();
        assert_eq!(exec_result, json!("input_value"));

        let action = <GetValueNodeBackend<_> as NodeBackend<MemoryStorage>>::post(
            &mut node,
            &mut store,
            Some(json!("input_value")),
            exec_result,
            &context,
        )
        .await
        .unwrap();
        assert_eq!(action, Action::simple("continue"));
        assert_eq!(store.get("output_key").unwrap(), Some(json!("input_value")));
    }
}
