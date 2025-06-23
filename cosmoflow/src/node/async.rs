//! # Async Node Implementation
//!
//! This module contains the async version of the Node trait and related functionality.
//! It's only available when the `async` feature is enabled.

use async_trait::async_trait;
use std::time::Duration;

use super::{ExecutionContext, NodeError};
use crate::action::Action;
use crate::shared_store::SharedStore;

/// Node trait that defines the complete interface for nodes in CosmoFlow workflows (async version).
///
/// This trait combines all functionality needed for node execution in a single, cohesive
/// interface. It incorporates the three-phase execution model (prep/exec/post) with
/// built-in retry logic, error handling, and configuration methods.
///
/// This is the async version that requires the `async` feature flag (enabled by default).
/// For synchronous execution, disable the `async` feature.
#[async_trait]
pub trait Node<S: SharedStore>: Send + Sync {
    /// Result type from the preparation phase.
    type PrepResult: Send + Sync + Clone + 'static;
    /// Result type from the execution phase.
    type ExecResult: Send + Sync + 'static;
    /// Error type for all operations in this node.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Preparation phase: Read and preprocess data from shared storage (async).
    async fn prep(
        &mut self,
        store: &S,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error>;

    /// Execution phase: Perform the core computation logic (async).
    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error>;

    /// Post-processing phase: Write results and determine next action (async).
    async fn post(
        &mut self,
        store: &mut S,
        prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error>;

    /// Maximum number of retries for the exec phase.
    fn max_retries(&self) -> usize {
        3
    }

    /// Delay between retry attempts.
    fn retry_delay(&self) -> Duration {
        Duration::from_millis(100)
    }

    /// Get the node's name for debugging and logging.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Fallback execution when all retries are exhausted (async).
    async fn exec_fallback(
        &mut self,
        _prep_result: Self::PrepResult,
        error: Self::Error,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Err(error)
    }

    /// Run the complete node execution cycle (async).
    async fn run(&mut self, store: &mut S) -> Result<Action, NodeError> {
        let context = ExecutionContext::new(self.max_retries(), self.retry_delay());

        let prep_result = self
            .prep(store, &context)
            .await
            .map_err(|e| NodeError::PreparationError(e.to_string()))?;

        let exec_result = self
            .exec_with_retries(prep_result.clone(), context.clone())
            .await
            .map_err(|e| NodeError::ExecutionError(e.to_string()))?;

        self.post(store, prep_result, exec_result, &context)
            .await
            .map_err(|e| NodeError::PostProcessingError(e.to_string()))
    }

    /// Execute with retry logic (async).
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
                        context.next_retry();
                        tokio::time::sleep(context.retry_delay).await;
                    } else {
                        return self.exec_fallback(prep_result, error, &context).await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_store::backends::MemoryStorage;

    #[derive(Debug)]
    struct TestError;
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "test error")
        }
    }
    impl std::error::Error for TestError {}

    struct TestNode {
        prep_success: bool,
        exec_success: bool,
        post_success: bool,
    }

    impl TestNode {
        fn new(prep_success: bool, exec_success: bool, post_success: bool) -> Self {
            Self {
                prep_success,
                exec_success,
                post_success,
            }
        }
    }

    #[async_trait]
    impl<S: SharedStore + Send + Sync> Node<S> for TestNode {
        type PrepResult = String;
        type ExecResult = String;
        type Error = TestError;

        async fn prep(
            &mut self,
            _store: &S,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            if self.prep_success {
                Ok("prep_result".to_string())
            } else {
                Err(TestError)
            }
        }

        async fn exec(
            &mut self,
            prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            if self.exec_success {
                Ok(format!("exec_{}", prep_result))
            } else {
                Err(TestError)
            }
        }

        async fn post(
            &mut self,
            _store: &mut S,
            _prep_result: Self::PrepResult,
            exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            if self.post_success {
                Ok(Action::simple(&exec_result))
            } else {
                Err(TestError)
            }
        }
    }

    #[tokio::test]
    async fn test_node_successful_execution() {
        let mut node = TestNode::new(true, true, true);
        let mut store = MemoryStorage::new();

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_node_prep_failure() {
        let mut node = TestNode::new(false, true, true);
        let mut store = MemoryStorage::new();

        let result = node.run(&mut store).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NodeError::PreparationError(_)
        ));
    }
}
