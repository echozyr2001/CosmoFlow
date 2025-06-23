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
//! - **Execution Support**: Both async and sync execution modes (configurable via features)
//! - **Retry Logic**: Built-in retry mechanisms with configurable policies
//! - **Error Handling**: Comprehensive error types and propagation
//! - **Execution Context**: Rich context information for node execution
//! - **Automatic Integration**: Seamless integration with the flow system
//!
//! ## Feature Flags
//!
//! - `async` (default): Enables async/await support with tokio runtime
//! - Without `async`: Provides synchronous execution for minimal compilation
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
//! ### Async Implementation (default)
//!
//! ```rust
//! # #[cfg(feature = "async")]
//! # {
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
//!     S: SharedStore + Send + Sync
//! {
//!     type PrepResult = String;
//!     type ExecResult = String;
//!     type Error = NodeError;
//!
//!     async fn prep(
//!         &mut self,
//!         _store: &S,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::PrepResult, Self::Error> {
//!         Ok(format!("Preparing: {}", self.name))
//!     }
//!
//!     async fn exec(
//!         &mut self,
//!         prep_result: Self::PrepResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::ExecResult, Self::Error> {
//!         Ok(format!("Executed: {}", prep_result))
//!     }
//!
//!     async fn post(
//!         &mut self,
//!         _store: &mut S,
//!         _prep_result: Self::PrepResult,
//!         exec_result: Self::ExecResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Action, Self::Error> {
//!         println!("{}", exec_result);
//!         Ok(Action::simple("complete"))
//!     }
//! }
//! # }
//! ```
//!
//! ### Sync Implementation (with --no-default-features)
//!
//! ```rust
//! # #[cfg(not(feature = "async"))]
//! # {
//! use cosmoflow::node::{Node, ExecutionContext, NodeError};
//! use cosmoflow::shared_store::SharedStore;
//! use cosmoflow::action::Action;
//!
//! struct MyCustomNode {
//!     name: String,
//! }
//!
//! impl<S> Node<S> for MyCustomNode
//! where
//!     S: SharedStore + Send + Sync
//! {
//!     type PrepResult = String;
//!     type ExecResult = String;
//!     type Error = NodeError;
//!
//!     fn prep(
//!         &mut self,
//!         _store: &S,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::PrepResult, Self::Error> {
//!         Ok(format!("Preparing: {}", self.name))
//!     }
//!
//!     fn exec(
//!         &mut self,
//!         prep_result: Self::PrepResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::ExecResult, Self::Error> {
//!         Ok(format!("Executed: {}", prep_result))
//!     }
//!
//!     fn post(
//!         &mut self,
//!         _store: &mut S,
//!         _prep_result: Self::PrepResult,
//!         exec_result: Self::ExecResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Action, Self::Error> {
//!         println!("{}", exec_result);
//!         Ok(Action::simple("complete"))
//!     }
//! }
//! # }
//! ```

/// The errors module contains the error types for the node crate.
pub mod errors;

/// Async node implementation (available when async feature is enabled)
#[cfg(feature = "async")]
pub mod r#async;

pub use errors::NodeError;

#[cfg(feature = "async")]
pub use r#async::Node;

use std::{collections::HashMap, time::Duration};

#[cfg(not(feature = "async"))]
use crate::action::Action;
#[cfg(not(feature = "async"))]
use crate::shared_store::SharedStore;
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

// Sync version of the Node trait (when async feature is disabled)
#[cfg(not(feature = "async"))]
/// Node trait that defines the complete interface for nodes in CosmoFlow workflows (sync version).
///
/// This trait combines all functionality needed for node execution in a single, cohesive
/// interface. It incorporates the three-phase execution model (prep/exec/post) with
/// built-in retry logic, error handling, and configuration methods.
///
/// This is the synchronous version that's available when the `async` feature is disabled.
/// For async execution, enable the `async` feature flag.
pub trait Node<S: SharedStore>: Send + Sync {
    /// Result type from the preparation phase.
    type PrepResult: Send + Sync + Clone + 'static;
    /// Result type from the execution phase.
    type ExecResult: Send + Sync + 'static;
    /// Error type for all operations in this node.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Preparation phase: Read and preprocess data from shared storage (sync).
    fn prep(
        &mut self,
        store: &S,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error>;

    /// Execution phase: Perform the core computation logic (sync).
    fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error>;

    /// Post-processing phase: Write results and determine next action (sync).
    fn post(
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

    /// Fallback execution when all retries are exhausted (sync).
    fn exec_fallback(
        &mut self,
        _prep_result: Self::PrepResult,
        error: Self::Error,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Err(error)
    }

    /// Run the complete node execution cycle (sync).
    fn run(&mut self, store: &mut S) -> Result<Action, NodeError> {
        let context = ExecutionContext::new(self.max_retries(), self.retry_delay());

        let prep_result = self
            .prep(store, &context)
            .map_err(|e| NodeError::PreparationError(e.to_string()))?;

        let exec_result = self
            .exec_with_retries(prep_result.clone(), context.clone())
            .map_err(|e| NodeError::ExecutionError(e.to_string()))?;

        self.post(store, prep_result, exec_result, &context)
            .map_err(|e| NodeError::PostProcessingError(e.to_string()))
    }

    /// Execute with retry logic (sync).
    fn exec_with_retries(
        &mut self,
        prep_result: Self::PrepResult,
        mut context: ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        loop {
            match self.exec(prep_result.clone(), &context) {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if context.can_retry() {
                        context.next_retry();
                        std::thread::sleep(context.retry_delay);
                    } else {
                        return self.exec_fallback(prep_result, error, &context);
                    }
                }
            }
        }
    }
}

// Tests for both sync and async versions
#[cfg(all(test, not(feature = "async")))]
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

    impl<S: SharedStore + Send + Sync> Node<S> for TestNode {
        type PrepResult = String;
        type ExecResult = String;
        type Error = TestError;

        fn prep(
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

        fn exec(
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

        fn post(
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

    #[test]
    fn test_node_successful_execution() {
        let mut node = TestNode::new(true, true, true);
        let mut store = MemoryStorage::new();

        let result = node.run(&mut store);
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_prep_failure() {
        let mut node = TestNode::new(false, true, true);
        let mut store = MemoryStorage::new();

        let result = node.run(&mut store);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NodeError::PreparationError(_)
        ));
    }
}
