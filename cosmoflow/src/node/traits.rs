use std::time::Duration;

use crate::action::Action;
use crate::shared_store::SharedStore;
use crate::storage::StorageBackend;
use async_trait::async_trait;

use crate::ExecutionContext;

/// Core trait for implementing custom node backends.
///
/// A Node represents the smallest building block in CosmoFlow workflows.
/// Each node has three execution phases:
/// 1. `prep` - Read and preprocess data from shared store
/// 2. `exec` - Execute compute logic (LLM calls, APIs, etc.)
/// 3. `post` - Postprocess and write results back to shared store
#[async_trait]
pub trait NodeBackend<S: StorageBackend>: Send + Sync {
    /// The type returned by the prep phase
    type PrepResult: Send + Sync + Clone + 'static;
    /// The type returned by the exec phase  
    type ExecResult: Send + Sync + 'static;
    /// Error type for this node
    type Error: std::error::Error + Send + Sync + 'static;

    /// Preparation phase: read and preprocess data from shared store
    ///
    /// This phase should:
    /// - Read necessary data from the shared store
    /// - Validate inputs
    /// - Prepare data for the execution phase
    ///
    /// Returns data that will be passed to `exec()`
    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error>;

    /// Execution phase: perform the main computation
    ///
    /// This phase should:
    /// - Perform the core logic (LLM calls, API requests, etc.)
    /// - Be idempotent (safe to retry)
    /// - NOT access the shared store directly
    ///
    /// Returns data that will be passed to `post()`
    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error>;

    /// Post-processing phase: write results back to shared store
    ///
    /// This phase should:
    /// - Write results to the shared store
    /// - Update state
    /// - Determine the next action
    ///
    /// Returns the action to take next
    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error>;

    /// Fallback handler for when exec() fails after all retries
    ///
    /// Override this to provide graceful error handling instead of propagating errors.
    /// By default, this re-raises the error.
    async fn exec_fallback(
        &mut self,
        _prep_result: Self::PrepResult,
        error: Self::Error,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Err(error)
    }

    /// Get the node's name/identifier for logging and debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Get maximum number of retries for this node
    fn max_retries(&self) -> usize {
        1 // Default: no retries
    }

    /// Get retry delay for this node
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(0) // Default: no delay
    }
}

#[cfg(all(test, feature = "storage-memory"))]
mod tests {
    use super::*;
    use crate::NodeError;
    use crate::shared_store::SharedStore;
    use crate::storage::MemoryStorage;
    use async_trait::async_trait;

    // Test implementation of NodeBackend
    struct MockNodeBackend {
        name: String,
        max_retries: usize,
        retry_delay: Duration,
    }

    impl MockNodeBackend {
        fn new(name: String) -> Self {
            Self {
                name,
                max_retries: 1,
                retry_delay: Duration::from_secs(0),
            }
        }

        fn with_retries(mut self, max_retries: usize) -> Self {
            self.max_retries = max_retries;
            self
        }

        fn with_delay(mut self, delay: Duration) -> Self {
            self.retry_delay = delay;
            self
        }
    }

    #[async_trait]
    impl NodeBackend<MemoryStorage> for MockNodeBackend {
        type PrepResult = String;
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<MemoryStorage>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok(format!("prep_{}", self.name))
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok(format!("exec_{}", prep_result))
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
    async fn test_node_backend_basic_functionality() {
        let mut backend = MockNodeBackend::new("test_node".to_string());
        let store = SharedStore::with_storage(MemoryStorage::new());
        let mut store_mut = SharedStore::with_storage(MemoryStorage::new());
        let context = ExecutionContext::new(1, Duration::from_millis(100));

        // Test prep
        let prep_result = backend.prep(&store, &context).await.unwrap();
        assert_eq!(prep_result, "prep_test_node");

        // Test exec
        let exec_result = backend.exec(prep_result.clone(), &context).await.unwrap();
        assert_eq!(exec_result, "exec_prep_test_node");

        // Test post
        let action = backend
            .post(&mut store_mut, prep_result, exec_result, &context)
            .await
            .unwrap();
        assert_eq!(action, Action::simple("continue"));
    }

    #[test]
    fn test_node_backend_configuration() {
        let backend = MockNodeBackend::new("test".to_string())
            .with_retries(5)
            .with_delay(Duration::from_secs(2));

        assert_eq!(backend.name(), "test");
        assert_eq!(backend.max_retries(), 5);
        assert_eq!(backend.retry_delay(), Duration::from_secs(2));
    }

    #[test]
    fn test_node_backend_default_name() {
        let backend = MockNodeBackend::new("test".to_string());
        // The default name() implementation returns the type name
        // Our implementation overrides it, so we test our custom implementation
        assert_eq!(backend.name(), "test");
    }

    // Test default fallback behavior
    struct FallbackTestBackend;

    #[async_trait]
    impl NodeBackend<MemoryStorage> for FallbackTestBackend {
        type PrepResult = String;
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<MemoryStorage>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok("prep".to_string())
        }

        async fn exec(
            &mut self,
            _prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok("exec".to_string())
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
    }

    #[tokio::test]
    async fn test_default_exec_fallback() {
        let mut backend = FallbackTestBackend;
        let context = ExecutionContext::new(1, Duration::from_millis(100));
        let error = NodeError::ExecutionError("test error".to_string());

        // Default fallback should re-raise the error
        let result = backend
            .exec_fallback("prep".to_string(), error, &context)
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NodeError::ExecutionError(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected ExecutionError"),
        }
    }

    #[test]
    fn test_default_configuration() {
        let backend = FallbackTestBackend;

        // Test default values
        assert_eq!(backend.max_retries(), 1);
        assert_eq!(backend.retry_delay(), Duration::from_secs(0));

        // Test default name (should be the type name)
        assert!(backend.name().contains("FallbackTestBackend"));
    }

    // Test custom fallback implementation
    struct CustomFallbackBackend {
        fallback_result: String,
    }

    impl CustomFallbackBackend {
        fn new(fallback_result: String) -> Self {
            Self { fallback_result }
        }
    }

    #[async_trait]
    impl NodeBackend<MemoryStorage> for CustomFallbackBackend {
        type PrepResult = String;
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<MemoryStorage>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok("prep".to_string())
        }

        async fn exec(
            &mut self,
            _prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            Ok("exec".to_string())
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
        let mut backend = CustomFallbackBackend::new("fallback_success".to_string());
        let context = ExecutionContext::new(1, Duration::from_millis(100));
        let error = NodeError::ExecutionError("test error".to_string());

        // Custom fallback should return the fallback result
        let result = backend
            .exec_fallback("prep".to_string(), error, &context)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fallback_success");
    }
}
