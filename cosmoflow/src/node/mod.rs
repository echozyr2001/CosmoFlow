#![deny(missing_docs)]
//! # Node - CosmoFlow Node Execution System
//!
//! This crate provides the core execution system for CosmoFlow workflows. It defines the
//! unified `Node` trait and execution infrastructure that enables workflows to run individual
//! processing units with proper error handling, retry logic, and execution context management.
//!
//! ## Key Features
//!
//! - **Unified Node Trait**: Single trait combining all node functionality
//! - **Type Safety**: Associated types provide compile-time guarantees
//! - **Async Execution**: Full async/await support for modern Rust applications
//! - **Retry Logic**: Built-in retry mechanisms with configurable policies
//! - **Error Handling**: Comprehensive error types and propagation
//! - **Execution Context**: Rich context information for node execution
//! - **Automatic Integration**: Seamless integration with the flow system
//!
//! ## Node Trait Design
//!
//! The Node trait provides a comprehensive interface that includes:
//! - Core execution methods (prep/exec/post) with associated types
//! - Configuration methods with sensible defaults
//! - Built-in retry logic and error handling
//! - Seamless integration with the flow system
//!
//! ## Quick Start
//!
//! ### Implementing a Custom Node
//!
//! ```rust
//! use cosmoflow::node::{Node, ExecutionContext, NodeError};
//! use cosmoflow::shared_store::SharedStore;
//! use cosmoflow::action::Action;
//! use async_trait::async_trait;
//!
//! struct MyCustomNode {
//!     name: String,
//! }
//!
//! #[async_trait]
//! impl<S> Node<S> for MyCustomNode
//! where
//!     S: cosmoflow::storage::StorageBackend + Send + Sync
//! {
//!     // Define associated types for type safety
//!     type PrepResult = String;
//!     type ExecResult = String;
//!     type Error = NodeError;
//!
//!     async fn prep(
//!         &mut self,
//!         _store: &SharedStore<S>,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::PrepResult, Self::Error> {
//!         // Prepare data for execution
//!         Ok(format!("Preparing: {}", self.name))
//!     }
//!
//!     async fn exec(
//!         &mut self,
//!         prep_result: Self::PrepResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::ExecResult, Self::Error> {
//!         // Core business logic
//!         Ok(format!("Executed: {}", prep_result))
//!     }
//!
//!     async fn post(
//!         &mut self,
//!         _store: &mut SharedStore<S>,
//!         _prep_result: Self::PrepResult,
//!         exec_result: Self::ExecResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Action, Self::Error> {
//!         // Handle results and determine next action
//!         println!("{}", exec_result);
//!         Ok(Action::simple("complete"))
//!     }
//! }
//! ```
//!
//! ### Using Execution Context
//!
//! ```rust
//! use cosmoflow::node::ExecutionContext;
//! use std::time::Duration;
//!
//! let context = ExecutionContext::new(3, Duration::from_millis(500));
//!
//! println!("Execution ID: {}", context.execution_id);
//! println!("Max retries: {}", context.max_retries);
//! println!("Can retry: {}", context.can_retry());
//! ```
//!
//! ## Core Types
//!
//! - [`ExecutionContext`]: Contains execution metadata and retry configuration
//! - [`Node`]: Unified trait for implementing custom node types with associated types
//! - [`NodeError`]: Comprehensive error types for node execution
//!
//! ## Three-Phase Execution
//!
//! The Node trait implements a three-phase execution model:
//!
//! 1. **Prep Phase**: Read and validate inputs (fast, side-effect free)
//! 2. **Exec Phase**: Perform core computation (idempotent, retryable)
//! 3. **Post Phase**: Write results and determine next action
//!
//! ## Error Handling
//!
//! The node system provides detailed error information:
//!
//! ```rust
//! use cosmoflow::node::NodeError;
//!
//! # let node_result: Result<(), NodeError> = Err(NodeError::ExecutionError("timeout".to_string()));
//! match node_result {
//!     Err(NodeError::ExecutionError(msg)) if msg.contains("timeout") => {
//!         println!("Node execution timed out");
//!     },
//!     Err(NodeError::ValidationError(message)) => {
//!         println!("Invalid input: {}", message);
//!     },
//!     _ => {}
//! }
//! ```

/// The errors module contains the error types for the node crate.
pub mod errors;

use async_trait::async_trait;
pub use errors::NodeError;

use std::{collections::HashMap, time::Duration};

use crate::action::Action;
use crate::shared_store::SharedStore;
use crate::storage::StorageBackend;
use serde_json::Value;
use uuid::Uuid;

/// Represents the execution context for a node, containing the current retry count
/// and other execution metadata.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Current retry attempt (0-based)
    pub current_retry: usize,
    /// Maximum number of retries allowed
    pub max_retries: usize,
    /// Wait duration between retries
    pub retry_delay: Duration,
    /// Unique execution ID for tracking
    pub execution_id: String,
    /// Additional metadata for the execution
    pub metadata: HashMap<String, Value>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(max_retries: usize, retry_delay: Duration) -> Self {
        Self {
            current_retry: 0,
            max_retries,
            retry_delay,
            execution_id: Uuid::new_v4().to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Check if more retries are available
    pub fn can_retry(&self) -> bool {
        self.current_retry < self.max_retries
    }

    /// Increment retry count
    pub fn next_retry(&mut self) {
        self.current_retry += 1;
    }

    /// Get the execution ID
    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Remove metadata value
    pub fn remove_metadata(&mut self, key: &str) -> Option<serde_json::Value> {
        self.metadata.remove(key)
    }

    /// Get all metadata
    pub fn metadata(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

/// Node trait that defines the complete interface for nodes in CosmoFlow workflows.
///
/// This trait combines all functionality needed for node execution in a single, cohesive
/// interface. It incorporates the three-phase execution model (prep/exec/post) with
/// built-in retry logic, error handling, and configuration methods.
///
/// ## Design Philosophy
///
/// The Node trait provides a comprehensive interface that includes:
/// - Core execution methods (prep/exec/post) with configurable associated types
/// - Configuration methods with sensible defaults
/// - Built-in retry logic and error handling
/// - Seamless integration with the flow system
/// - Type safety through associated types
///
/// ## Associated Types
///
/// Each node defines three associated types that provide compile-time type safety:
/// - `PrepResult`: Data type prepared in prep phase and passed to exec/post
/// - `ExecResult`: Data type returned by exec phase and passed to post
/// - `Error`: Error type for this node's operations
///
/// ## Three-Phase Execution Model
///
/// 1. **Prep Phase** (`prep`): Read and validate inputs from shared storage
///    - Should be fast and side-effect free
///    - Used for input validation and data preparation
///    - Results are passed to both exec and post phases
///
/// 2. **Exec Phase** (`exec`): Perform core computation logic
///    - Must be idempotent (safe to retry)
///    - Should not access shared storage directly
///    - Contains the main business logic (API calls, computations, etc.)
///
/// 3. **Post Phase** (`post`): Write results and determine next action
///    - Handles side effects (storage writes, notifications, etc.)
///    - Determines the next workflow action
///    - Has access to both prep and exec results
///
/// ## Error Handling and Retries
///
/// The trait includes built-in retry logic for the exec phase:
/// - Configurable retry count via `max_retries()`
/// - Configurable retry delay via `retry_delay()`
/// - Fallback mechanism via `exec_fallback()` when all retries are exhausted
/// - Automatic error wrapping and propagation
///
/// ## Type Parameters
///
/// * `S` - Storage backend type that implements `StorageBackend`
#[async_trait]
pub trait Node<S: StorageBackend>: Send + Sync {
    /// Result type from the preparation phase.
    ///
    /// This type must be `Clone` because it's passed to both the exec and post phases.
    /// It should contain all the data needed for the execution phase.
    type PrepResult: Send + Sync + Clone + 'static;

    /// Result type from the execution phase.
    ///
    /// This type contains the output of the core computation and is passed
    /// to the post phase for storage and further processing.
    type ExecResult: Send + Sync + 'static;

    /// Error type for all operations in this node.
    ///
    /// This should be a comprehensive error type that can represent all
    /// possible failure modes for this node's operations.
    type Error: std::error::Error + Send + Sync + 'static;

    // === Core Execution Methods (Must be implemented) ===

    /// Preparation phase: Read and preprocess data from shared storage.
    ///
    /// This phase is responsible for:
    /// - Reading necessary data from the shared store
    /// - Validating inputs and checking preconditions
    /// - Preparing data structures for the execution phase
    /// - Performing any setup operations that don't have side effects
    ///
    /// The preparation phase should be:
    /// - **Fast**: Avoid expensive operations here
    /// - **Side-effect free**: No writes to storage or external systems
    /// - **Deterministic**: Same inputs should produce same outputs
    ///
    /// The returned data will be passed to both `exec()` and `post()` phases.
    ///
    /// # Arguments
    ///
    /// * `store` - Immutable reference to shared storage for reading data
    /// * `context` - Execution context containing retry info and metadata
    ///
    /// # Returns
    ///
    /// * `Ok(PrepResult)` - Successfully prepared data for execution
    /// * `Err(Self::Error)` - Preparation failed (will abort the entire node execution)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cosmoflow::node::ExecutionContext;
    /// # use cosmoflow::shared_store::SharedStore;
    /// # use cosmoflow::storage::StorageBackend;
    /// #
    /// # struct MyPrepData { message: String }
    /// # #[derive(Debug)]
    /// # enum MyError { MissingConfig, InvalidTimeout }
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "error") }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// # struct MyNode;
    /// # impl MyNode {
    /// async fn prep(&mut self, _store: &SharedStore<impl StorageBackend>, _context: &ExecutionContext)
    ///     -> Result<MyPrepData, MyError> {
    ///     // Read configuration from storage
    ///     // Validate inputs
    ///     Ok(MyPrepData { message: "prepared".to_string() })
    /// }
    /// # }
    /// ```
    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error>;

    /// Execution phase: Perform the core computation logic.
    ///
    /// This phase contains the main business logic of the node:
    /// - API calls to external services
    /// - Data processing and transformations
    /// - Complex computations or algorithms
    /// - LLM interactions or other AI operations
    ///
    /// The execution phase must be:
    /// - **Idempotent**: Safe to retry multiple times with same inputs
    /// - **Stateless**: Should not modify node state or access shared storage
    /// - **Pure**: Given same prep_result, should produce same output
    ///
    /// This phase is automatically retried according to the node's retry configuration.
    /// If all retries fail, the `exec_fallback()` method will be called.
    ///
    /// # Arguments
    ///
    /// * `prep_result` - Data prepared in the prep phase
    /// * `context` - Execution context with current retry count and metadata
    ///
    /// # Returns
    ///
    /// * `Ok(ExecResult)` - Successfully computed result
    /// * `Err(Self::Error)` - Execution failed (will trigger retry or fallback)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cosmoflow::node::ExecutionContext;
    /// # use std::time::Duration;
    /// #
    /// # struct MyPrepData { config: Config }
    /// # struct Config { timeout: u64 }
    /// # #[derive(Debug)]
    /// # enum MyError { NetworkError }
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "error") }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// # struct HttpClient;
    /// # impl HttpClient {
    /// #     fn new() -> Self { HttpClient }
    /// #     fn post(&self, _url: &str) -> RequestBuilder { RequestBuilder }
    /// # }
    /// # struct RequestBuilder;
    /// # impl RequestBuilder {
    /// #     fn json<T>(self, _json: &T) -> Self { self }
    /// #     fn timeout(self, _duration: Duration) -> Self { self }
    /// #     async fn send(self) -> Result<Response, MyError> { Ok(Response) }
    /// # }
    /// # struct Response;
    /// # impl Response {
    /// #     async fn text(self) -> Result<String, MyError> { Ok("result".to_string()) }
    /// # }
    /// #
    /// # struct MyNode;
    /// # impl MyNode {
    /// async fn exec(&mut self, prep_result: MyPrepData, _context: &ExecutionContext)
    ///     -> Result<String, MyError> {
    ///     // Perform main computation
    ///     let client = HttpClient::new();
    ///     let response = client.post("https://api.example.com/process")
    ///         .json(&prep_result.config)
    ///         .timeout(Duration::from_secs(prep_result.config.timeout))
    ///         .send()
    ///         .await?;
    ///     
    ///     let result = response.text().await?;
    ///     Ok(result)
    /// }
    /// # }
    /// ```
    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error>;

    /// Post-processing phase: Write results and determine the next action.
    ///
    /// This phase is responsible for:
    /// - Writing results to shared storage
    /// - Updating external systems or databases
    /// - Sending notifications or triggering events
    /// - Determining what action the workflow should take next
    ///
    /// The post phase has access to both the prep and exec results, allowing
    /// for comprehensive result processing and decision making.
    ///
    /// # Arguments
    ///
    /// * `store` - Mutable reference to shared storage for writing results
    /// * `prep_result` - Data from the preparation phase
    /// * `exec_result` - Result from the execution phase
    /// * `context` - Execution context with metadata
    ///
    /// # Returns
    ///
    /// * `Ok(Action)` - The action that determines the next step in the workflow
    /// * `Err(Self::Error)` - Post-processing failed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cosmoflow::node::ExecutionContext;
    /// # use cosmoflow::shared_store::SharedStore;
    /// # use cosmoflow::storage::StorageBackend;
    /// # use cosmoflow::action::Action;
    /// #
    /// # struct MyPrepData;
    /// # #[derive(Debug)]
    /// # enum MyError { StorageError }
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "error") }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// # struct MyNode;
    /// # impl MyNode {
    /// async fn post(&mut self, _store: &mut SharedStore<impl StorageBackend>, _prep_result: MyPrepData,
    ///               exec_result: String, _context: &ExecutionContext)
    ///     -> Result<Action, MyError> {
    ///     // Store results and determine next action
    ///     if exec_result.contains("error") {
    ///         Ok(Action::simple("handle_error"))
    ///     } else {
    ///         Ok(Action::simple("continue"))
    ///     }
    /// }
    /// # }
    /// ```
    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error>;

    // === Configuration Methods (Default implementations provided) ===

    /// Returns the node's name/identifier for logging and debugging purposes.
    ///
    /// This name is used in:
    /// - Log messages and error reports
    /// - Workflow execution traces
    /// - Debugging and monitoring tools
    /// - Flow validation and visualization
    ///
    /// The default implementation returns the Rust type name, but nodes
    /// should typically override this to provide a more user-friendly name.
    ///
    /// # Returns
    ///
    /// A string slice containing the node's human-readable name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # struct MyNode { name: String }
    /// # impl MyNode {
    /// fn name(&self) -> &str {
    ///     "UserDataProcessor"  // Custom meaningful name
    /// }
    /// # }
    /// ```
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns the maximum number of retry attempts for the execution phase.
    ///
    /// When the `exec()` method fails, it will be retried up to this many times
    /// before calling the `exec_fallback()` method. The retry count does not
    /// include the initial attempt.
    ///
    /// **Default**: 1 (no retries - fail immediately on first error)
    ///
    /// # Returns
    ///
    /// The maximum number of retry attempts (0 = no retries, 1+ = retry count)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # struct MyNode;
    /// # impl MyNode {
    /// fn max_retries(&self) -> usize {
    ///     3  // Retry up to 3 times for network operations
    /// }
    /// # }
    /// ```
    fn max_retries(&self) -> usize {
        1 // Default: no retries
    }

    /// Returns the delay between retry attempts.
    ///
    /// This delay is applied before each retry attempt (not before the initial attempt).
    /// The delay helps prevent overwhelming external services and allows time for
    /// transient issues to resolve.
    ///
    /// **Default**: 0 seconds (no delay between retries)
    ///
    /// # Returns
    ///
    /// Duration to wait between retry attempts
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// # struct MyNode;
    /// # impl MyNode {
    /// fn retry_delay(&self) -> Duration {
    ///     Duration::from_millis(500)  // Wait 500ms between retries
    /// }
    /// # }
    /// ```
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(0) // Default: no delay
    }

    /// Fallback handler called when exec() fails after all retries are exhausted.
    ///
    /// This method provides a way to handle execution failures gracefully instead
    /// of propagating the error up the call stack. Common use cases include:
    /// - Providing default values when external services are unavailable
    /// - Implementing circuit breaker patterns
    /// - Logging detailed error information before failing
    /// - Attempting alternative computation methods
    ///
    /// **Default behavior**: Re-raises the original error (no fallback)
    ///
    /// # Arguments
    ///
    /// * `_prep_result` - The data from the prep phase (available for fallback logic)
    /// * `error` - The final error that caused all retries to fail
    /// * `_context` - Execution context with retry information
    ///
    /// # Returns
    ///
    /// * `Ok(ExecResult)` - Fallback succeeded, continue with this result
    /// * `Err(Self::Error)` - Fallback also failed, propagate this error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cosmoflow::node::ExecutionContext;
    /// #
    /// # #[derive(Debug)]
    /// # enum MyError { NetworkError }
    /// # impl std::fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "network error") }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// # struct MyNode;
    /// # impl MyNode {
    /// async fn exec_fallback(&mut self, _prep_result: String,
    ///                        error: MyError, _context: &ExecutionContext)
    ///     -> Result<String, MyError> {
    ///     // Log the failure for monitoring
    ///     eprintln!("Execution failed after retries: {}", error);
    ///     
    ///     // Provide a default result instead of failing
    ///     Ok("default_response".to_string())
    /// }
    /// # }
    /// ```
    async fn exec_fallback(
        &mut self,
        _prep_result: Self::PrepResult,
        error: Self::Error,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Err(error)
    }

    // === Convenience Methods (Default implementations provided) ===

    /// Complete execution flow: prep → exec → post.
    ///
    /// This is the main entry point for node execution and orchestrates the
    /// complete three-phase execution process:
    ///
    /// 1. **Preparation Phase**: Calls `prep()` to read and validate inputs
    /// 2. **Execution Phase**: Calls `exec_with_retries()` for core computation with retry logic
    /// 3. **Post-processing Phase**: Calls `post()` to store results and determine next action
    ///
    /// This method handles:
    /// - Automatic retry configuration setup
    /// - Error wrapping and propagation between phases
    /// - Execution context management
    /// - Phase coordination and data passing
    ///
    /// # Arguments
    ///
    /// * `store` - Mutable reference to shared storage (immutable for prep, mutable for post)
    ///
    /// # Returns
    ///
    /// * `Ok(Action)` - The action returned by the post phase
    /// * `Err(NodeError)` - Wrapped error from any phase that failed
    ///
    /// # Error Handling
    ///
    /// Errors from each phase are wrapped in specific `NodeError` variants:
    /// - Prep errors → `NodeError::PrepError`
    /// - Exec errors → `NodeError::ExecutionError` (after all retries and fallback)
    /// - Post errors → `NodeError::ExecutionError`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cosmoflow::node::{ExecutionContext, NodeError};
    /// # use cosmoflow::shared_store::SharedStore;
    /// # use cosmoflow::storage::StorageBackend;
    /// # use cosmoflow::action::Action;
    /// #
    /// # struct MyCustomNode;
    /// #
    /// # async fn example() -> Result<(), NodeError> {
    /// #     let mut my_node = MyCustomNode;
    /// #     let action = Action::simple("complete");
    /// #     println!("Node completed with action: {}", action.name());
    /// #     Ok(())
    /// # }
    /// ```
    async fn run(&mut self, store: &mut SharedStore<S>) -> Result<Action, NodeError> {
        let context = ExecutionContext::new(self.max_retries(), self.retry_delay());

        // Prep phase: read and validate inputs
        let prep_result = self
            .prep(store, &context)
            .await
            .map_err(|e| NodeError::PrepError(format!("Prep failed: {e}")))?;

        // Exec phase with retries: perform core computation
        let exec_result = self
            .exec_with_retries(prep_result.clone(), context.clone())
            .await
            .map_err(|e| NodeError::ExecutionError(format!("Exec failed: {e}")))?;

        // Post phase: store results and determine next action
        let action = self
            .post(store, prep_result, exec_result, &context)
            .await
            .map_err(|e| NodeError::ExecutionError(format!("Post failed: {e}")))?;

        Ok(action)
    }

    /// Execution phase with built-in retry logic and fallback handling.
    ///
    /// This method implements the retry mechanism for the execution phase:
    ///
    /// 1. **Initial Attempt**: Calls `exec()` with the prep result
    /// 2. **Retry Loop**: If exec fails and retries are available:
    ///    - Waits for the configured retry delay
    ///    - Increments retry count in execution context
    ///    - Attempts exec again
    /// 3. **Fallback**: If all retries are exhausted, calls `exec_fallback()`
    ///
    /// The retry logic is designed to handle transient failures while avoiding
    /// infinite loops or overwhelming external services.
    ///
    /// # Arguments
    ///
    /// * `prep_result` - Data from the preparation phase (cloned for each retry)
    /// * `context` - Mutable execution context to track retry attempts
    ///
    /// # Returns
    ///
    /// * `Ok(ExecResult)` - Execution succeeded (either normally or via fallback)
    /// * `Err(Self::Error)` - All attempts failed including fallback
    ///
    /// # Retry Behavior
    ///
    /// - **Retry Count**: Controlled by `max_retries()` method
    /// - **Retry Delay**: Controlled by `retry_delay()` method
    /// - **Fallback**: Controlled by `exec_fallback()` method
    /// - **Context Updates**: Retry count and metadata are updated automatically
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cosmoflow::node::ExecutionContext;
    /// # use std::time::Duration;
    /// #
    /// # async fn example() {
    /// // This method is typically called internally by run(), but can be used directly:
    /// let context = ExecutionContext::new(3, Duration::from_millis(500));
    /// println!("Context created with {} max retries", context.max_retries);
    /// # }
    /// ```
    async fn exec_with_retries(
        &mut self,
        prep_result: Self::PrepResult,
        mut context: ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        loop {
            match self.exec(prep_result.clone(), &context).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if context.can_retry() {
                        // Wait before retry if delay is configured
                        if context.retry_delay > Duration::ZERO {
                            tokio::time::sleep(context.retry_delay).await;
                        }
                        context.next_retry();
                        continue;
                    } else {
                        // All retries exhausted, try fallback
                        return self.exec_fallback(prep_result, error, &context).await;
                    }
                }
            }
        }
    }
}

#[cfg(all(test, feature = "storage-memory"))]
mod tests {
    use super::*;
    use crate::shared_store::SharedStore;
    use crate::storage::MemoryStorage;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct TestNode {
        name: String,
        message: String,
        action: Action,
        max_retries: usize,
        retry_delay: Duration,
        fail_count: Arc<Mutex<usize>>,
        should_fail_prep: bool,
        should_fail_exec: bool,
        should_fail_post: bool,
    }

    impl TestNode {
        pub fn new(name: impl Into<String>, message: impl Into<String>, action: Action) -> Self {
            Self {
                name: name.into(),
                message: message.into(),
                action,
                max_retries: 1,
                retry_delay: Duration::from_secs(0),
                fail_count: Arc::new(Mutex::new(0)),
                should_fail_prep: false,
                should_fail_exec: false,
                should_fail_post: false,
            }
        }

        pub fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }

        pub fn with_delay(mut self, delay: Duration) -> Self {
            self.retry_delay = delay;
            self
        }

        pub fn with_prep_failure(mut self) -> Self {
            self.should_fail_prep = true;
            self
        }

        pub fn with_exec_failure(mut self, fail_times: usize) -> Self {
            self.should_fail_exec = true;
            *self.fail_count.lock().unwrap() = fail_times;
            self
        }

        pub fn with_post_failure(mut self) -> Self {
            self.should_fail_post = true;
            self
        }
    }

    #[async_trait]
    impl Node<MemoryStorage> for TestNode {
        type PrepResult = String;
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<MemoryStorage>,
            context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            if self.should_fail_prep {
                return Err(NodeError::PrepError("Intentional prep failure".to_string()));
            }
            Ok(format!(
                "Execution {}: {}",
                context.execution_id, self.message
            ))
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            if self.should_fail_exec {
                let mut count = self.fail_count.lock().unwrap();
                if *count > 0 {
                    *count -= 1;
                    return Err(NodeError::ExecutionError(
                        "Intentional exec failure".to_string(),
                    ));
                }
            }
            Ok(prep_result)
        }

        async fn post(
            &mut self,
            store: &mut SharedStore<MemoryStorage>,
            _prep_result: Self::PrepResult,
            exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            if self.should_fail_post {
                return Err(NodeError::ExecutionError(
                    "Intentional post failure".to_string(),
                ));
            }

            store
                .set("last_result".to_string(), exec_result)
                .map_err(|e| NodeError::StorageError(e.to_string()))?;

            Ok(self.action.clone())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }

        fn retry_delay(&self) -> Duration {
            self.retry_delay
        }
    }

    #[tokio::test]
    async fn test_node_successful_execution() {
        let mut node = TestNode::new("test_node", "test message", Action::simple("continue"));
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Action::simple("continue"));

        let stored_result: Option<String> = store.get("last_result").unwrap();
        assert!(stored_result.is_some());
        assert!(stored_result.unwrap().contains("test message"));
    }

    #[tokio::test]
    async fn test_node_prep_failure() {
        let mut node = TestNode::new("test_node", "test message", Action::simple("continue"))
            .with_prep_failure();
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NodeError::PrepError(msg) => assert!(msg.contains("Prep failed")),
            _ => panic!("Expected PrepError"),
        }
    }

    #[tokio::test]
    async fn test_node_exec_retry_success() {
        let mut node = TestNode::new("test_node", "test message", Action::simple("continue"))
            .with_exec_failure(1)
            .with_retries(2);
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_node_exec_retry_exhausted() {
        let mut node = TestNode::new("test_node", "test message", Action::simple("continue"))
            .with_exec_failure(3)
            .with_retries(2);
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NodeError::ExecutionError(msg) => assert!(msg.contains("Exec failed")),
            _ => panic!("Expected ExecutionError"),
        }
    }

    #[tokio::test]
    async fn test_node_post_failure() {
        let mut node = TestNode::new("test_node", "test message", Action::simple("continue"))
            .with_post_failure();
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NodeError::ExecutionError(msg) => assert!(msg.contains("Post failed")),
            _ => panic!("Expected ExecutionError"),
        }
    }

    #[test]
    fn test_node_configuration() {
        let node = TestNode::new("test_node", "test message", Action::simple("continue"))
            .with_retries(5)
            .with_delay(Duration::from_secs(2));

        assert_eq!(node.name(), "test_node");
        assert_eq!(node.max_retries(), 5);
        assert_eq!(node.retry_delay(), Duration::from_secs(2));
    }

    #[test]
    fn test_node_default_name() {
        let node = TestNode::new("custom_name", "test message", Action::simple("continue"));
        assert_eq!(node.name(), "custom_name");
    }

    struct CustomFallbackNode {
        fallback_result: String,
    }

    impl CustomFallbackNode {
        fn new(fallback_result: String) -> Self {
            Self { fallback_result }
        }
    }

    #[async_trait]
    impl Node<MemoryStorage> for CustomFallbackNode {
        type PrepResult = ();
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<MemoryStorage>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok(())
        }

        async fn exec(
            &mut self,
            _prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Err(NodeError::ExecutionError("Always fails".to_string()))
        }

        async fn post(
            &mut self,
            _store: &mut SharedStore<MemoryStorage>,
            _prep_result: Self::PrepResult,
            _exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(Action::simple("continue"))
        }

        async fn exec_fallback(
            &mut self,
            _prep_result: Self::PrepResult,
            _error: Self::Error,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok(self.fallback_result.clone())
        }
    }

    #[tokio::test]
    async fn test_custom_exec_fallback() {
        let mut node = CustomFallbackNode::new("fallback_success".to_string());
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
    }
}
