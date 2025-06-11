//! # Unified Node Trait
//!
//! This module contains the unified Node trait that combines the functionality
//! of the original NodeBackend and Node wrapper into a single, simplified interface.
//! This design eliminates the need for separate backend implementations and
//! provides a more intuitive API for creating workflow nodes.
//!
//! ## Key Features
//!
//! - **Three-Phase Execution**: prep/exec/post pattern for clear separation of concerns
//! - **Built-in Retry Logic**: Configurable retry mechanisms with delays and fallbacks
//! - **Error Handling**: Comprehensive error propagation and recovery
//! - **Type Safety**: Strong typing for all phases and results
//! - **Async Support**: Full async/await support throughout the execution pipeline
//!
//! ## Usage Example
//!
//! ```rust
//! use async_trait::async_trait;
//! use cosmoflow::node::unified_trait::Node;
//! use cosmoflow::storage::MemoryStorage;
//!
//! struct MyNode {
//!     data: String,
//! }
//!
//! #[async_trait]
//! impl Node<MemoryStorage> for MyNode {
//!     type PrepResult = String;
//!     type ExecResult = String;
//!     type Error = NodeError;
//!     
//!     async fn prep(&mut self, store: &SharedStore<MemoryStorage>, context: &ExecutionContext)
//!         -> Result<String, NodeError> {
//!         // Preparation logic here
//!         Ok(self.data.clone())
//!     }
//!     
//!     async fn exec(&mut self, prep_result: String, context: &ExecutionContext)
//!         -> Result<String, NodeError> {
//!         // Core computation here
//!         Ok(format!("Processed: {}", prep_result))
//!     }
//!     
//!     async fn post(&mut self, store: &mut SharedStore<MemoryStorage>,
//!                   prep_result: String, exec_result: String, context: &ExecutionContext)
//!         -> Result<Action, NodeError> {
//!         // Store results and determine next action
//!         store.set("result".to_string(), exec_result)?;
//!         Ok(Action::simple("complete"))
//!     }
//! }
//! ```

use std::time::Duration;

use crate::action::Action;
use crate::shared_store::SharedStore;
use crate::storage::StorageBackend;
use async_trait::async_trait;

use crate::ExecutionContext;
use crate::node::errors::NodeError;

/// Unified Node trait that combines NodeBackend and Node functionality.
///
/// This trait defines the complete interface for nodes in CosmoFlow workflows,
/// incorporating the three-phase execution model with built-in retry logic,
/// error handling, and configuration methods.
///
/// ## Design Philosophy
///
/// The unified Node trait eliminates the complexity of the original two-layer
/// design (NodeBackend + Node wrapper) by providing a single trait that includes:
/// - Core execution methods (prep/exec/post)
/// - Configuration methods with sensible defaults
/// - Built-in retry logic and error handling
/// - Convenience methods for complete execution flows
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
///
/// ## Associated Types
///
/// * `PrepResult` - Data type returned by prep phase and passed to exec/post
/// * `ExecResult` - Data type returned by exec phase and passed to post
/// * `Error` - Error type for this node's operations
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
    /// async fn prep(&mut self, store: &SharedStore<S>, context: &ExecutionContext)
    ///     -> Result<MyPrepData, MyError> {
    ///     // Read configuration from storage
    ///     let config = store.get("config")?
    ///         .ok_or(MyError::MissingConfig)?;
    ///     
    ///     // Validate inputs
    ///     if config.timeout <= 0 {
    ///         return Err(MyError::InvalidTimeout);
    ///     }
    ///     
    ///     Ok(MyPrepData { config, timestamp: context.execution_id.clone() })
    /// }
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
    /// async fn exec(&mut self, prep_result: MyPrepData, context: &ExecutionContext)
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
    /// async fn post(&mut self, store: &mut SharedStore<S>, prep_result: MyPrepData,
    ///               exec_result: String, context: &ExecutionContext)
    ///     -> Result<Action, MyError> {
    ///     // Store the result
    ///     store.set("last_result".to_string(), &exec_result)?;
    ///     
    ///     // Update metrics
    ///     let metrics = store.get::<Metrics>("metrics")?.unwrap_or_default();
    ///     metrics.increment_success_count();
    ///     store.set("metrics".to_string(), metrics)?;
    ///     
    ///     // Determine next action based on result
    ///     if exec_result.contains("error") {
    ///         Ok(Action::simple("handle_error"))
    ///     } else {
    ///         Ok(Action::simple("continue"))
    ///     }
    /// }
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
    /// fn name(&self) -> &str {
    ///     "UserDataProcessor"  // Custom meaningful name
    /// }
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
    /// fn max_retries(&self) -> usize {
    ///     3  // Retry up to 3 times for network operations
    /// }
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
    /// fn retry_delay(&self) -> Duration {
    ///     Duration::from_millis(500)  // Wait 500ms between retries
    /// }
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
    /// async fn exec_fallback(&mut self, _prep_result: Self::PrepResult,
    ///                        error: Self::Error, _context: &ExecutionContext)
    ///     -> Result<Self::ExecResult, Self::Error> {
    ///     // Log the failure for monitoring
    ///     log::warn!("Execution failed after retries: {}", error);
    ///     
    ///     // Provide a default result instead of failing
    ///     Ok("default_response".to_string())
    /// }
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
    /// let mut my_node = MyCustomNode::new();
    /// let mut store = SharedStore::with_storage(MemoryStorage::new());
    ///
    /// match my_node.run(&mut store).await {
    ///     Ok(action) => println!("Node completed with action: {}", action.name()),
    ///     Err(NodeError::PrepError(msg)) => eprintln!("Preparation failed: {}", msg),
    ///     Err(NodeError::ExecutionError(msg)) => eprintln!("Execution failed: {}", msg),
    ///     Err(e) => eprintln!("Other error: {}", e),
    /// }
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
    /// // This method is typically called internally by run(), but can be used directly:
    /// let context = ExecutionContext::new(3, Duration::from_millis(500));
    /// let prep_data = self.prep(&store, &context).await?;
    /// let result = self.exec_with_retries(prep_data, context).await?;
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
