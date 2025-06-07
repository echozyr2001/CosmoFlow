#![deny(missing_docs)]
//! # Node - CosmoFlow Node Execution System
//!
//! This crate provides the core execution system for CosmoFlow workflows. It defines the
//! `Node` trait and execution infrastructure that enables workflows to run individual
//! processing units with proper error handling, retry logic, and execution context management.
//!
//! ## Key Features
//!
//! - **Async Execution**: Full async/await support for modern Rust applications
//! - **Retry Logic**: Built-in retry mechanisms with configurable policies
//! - **Error Handling**: Comprehensive error types and propagation
//! - **Execution Context**: Rich context information for node execution
//! - **Type Safety**: Compile-time guarantees for node implementations
//!
//! ## Quick Start
//!
//! ### Implementing a Custom Node
//!
//! ```rust
//! use cosmoflow::node::{Node, NodeBackend, ExecutionContext, NodeError};
//! use shared_store::SharedStore;
//! use action::Action;
//! use async_trait::async_trait;
//!
//! struct MyCustomNode {
//!     name: String,
//! }
//!
//! #[async_trait]
//! impl<S> NodeBackend<S> for MyCustomNode
//! where
//!     S: storage::StorageBackend + Send + Sync
//! {
//!     type PrepResult = ();
//!     type ExecResult = String;
//!     type Error = NodeError;
//!
//!     async fn prep(
//!         &mut self,
//!         _store: &SharedStore<S>,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::PrepResult, Self::Error> {
//!         Ok(())
//!     }
//!
//!     async fn exec(
//!         &mut self,
//!         _prep_result: Self::PrepResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::ExecResult, Self::Error> {
//!         Ok(format!("Executed node: {}", self.name))
//!     }
//!
//!     async fn post(
//!         &mut self,
//!         _store: &mut SharedStore<S>,
//!         _prep_result: Self::PrepResult,
//!         _exec_result: Self::ExecResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Action, Self::Error> {
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
//! ```
//!
//! ## Core Types
//!
//! - [`ExecutionContext`]: Contains execution metadata and retry configuration
//! - [`NodeBackend`]: Trait for implementing custom node types
//! - [`Node`]: Wrapper that provides retry logic and error handling
//! - [`NodeError`]: Comprehensive error types for node execution
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
/// The traits module contains the `NodeBackend` trait.
pub mod traits;

pub use errors::NodeError;
pub use traits::NodeBackend;

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

/// A concrete Node implementation that wraps a NodeBackend
pub struct Node<B, S>
where
    B: NodeBackend<S>,
    S: StorageBackend,
{
    backend: B,
    _phantom: std::marker::PhantomData<S>,
}

impl<B, S> Node<B, S>
where
    B: NodeBackend<S>,
    S: StorageBackend,
{
    /// Create a new node with the given backend
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Run the complete node execution cycle: prep -> exec -> post
    pub async fn run(&mut self, store: &mut SharedStore<S>) -> Result<Action, errors::NodeError> {
        let context = ExecutionContext::new(self.backend.max_retries(), self.backend.retry_delay());

        // Prep phase
        let prep_result = self
            .backend
            .prep(store, &context)
            .await
            .map_err(|e| errors::NodeError::PrepError(format!("Prep failed: {e}")))?;

        // Exec phase with retries
        let exec_result = self
            .exec_with_retries(prep_result.clone(), context.clone())
            .await
            .map_err(|e| errors::NodeError::ExecutionError(format!("Exec failed: {e}")))?;

        // Post phase
        let action = self
            .backend
            .post(store, prep_result, exec_result, &context)
            .await
            .map_err(|e| errors::NodeError::ExecutionError(format!("Post failed: {e}")))?;

        Ok(action)
    }

    /// Execute the exec phase with retry logic
    async fn exec_with_retries(
        &mut self,
        prep_result: B::PrepResult,
        mut context: ExecutionContext,
    ) -> Result<B::ExecResult, B::Error> {
        loop {
            match self.backend.exec(prep_result.clone(), &context).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if context.can_retry() {
                        // Wait before retry
                        if context.retry_delay > Duration::ZERO {
                            tokio::time::sleep(context.retry_delay).await;
                        }
                        context.next_retry();
                        continue;
                    } else {
                        // All retries exhausted, try fallback
                        match self
                            .backend
                            .exec_fallback(prep_result, error, &context)
                            .await
                        {
                            Ok(result) => return Ok(result),
                            Err(fallback_error) => {
                                return Err(fallback_error);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get the underlying backend
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Get mutable reference to the underlying backend
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

#[cfg(all(test, feature = "storage-memory"))]
mod tests {
    use crate::shared_store::SharedStore;
    use crate::storage::MemoryStorage;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    use super::*;

    // Test helper structures
    struct TestNode {
        message: String,
        action: Action,
        max_retries: usize,
        fail_count: Arc<Mutex<usize>>,
        should_fail_prep: bool,
        should_fail_exec: bool,
        should_fail_post: bool,
    }

    impl TestNode {
        pub fn new<S: Into<String>>(message: S, action: Action) -> Self {
            Self {
                message: message.into(),
                action,
                max_retries: 1,
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
    impl NodeBackend<MemoryStorage> for TestNode {
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
            _store: &mut SharedStore<MemoryStorage>,
            _prep_result: Self::PrepResult,
            _exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            if self.should_fail_post {
                return Err(NodeError::ExecutionError(
                    "Intentional post failure".to_string(),
                ));
            }
            Ok(self.action.clone())
        }

        fn max_retries(&self) -> usize {
            self.max_retries
        }

        // fn retry_delay(&self) -> Duration {
        //     self.retry_delay
        // }
    }

    // Test ExecutionContext
    #[test]
    fn test_execution_context_creation() {
        let context = ExecutionContext::new(3, Duration::from_secs(1));

        assert_eq!(context.current_retry, 0);
        assert_eq!(context.max_retries, 3);
        assert_eq!(context.retry_delay, Duration::from_secs(1));
        assert!(!context.execution_id.is_empty());
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_execution_context_retry_logic() {
        let mut context = ExecutionContext::new(2, Duration::from_millis(100));

        // Initially can retry
        assert!(context.can_retry());

        // After first retry
        context.next_retry();
        assert_eq!(context.current_retry, 1);
        assert!(context.can_retry());

        // After second retry
        context.next_retry();
        assert_eq!(context.current_retry, 2);
        assert!(!context.can_retry());
    }

    #[test]
    fn test_execution_context_metadata() {
        let mut context = ExecutionContext::new(1, Duration::from_millis(100));

        // Test setting and getting metadata
        context.set_metadata(
            "key1".to_string(),
            serde_json::Value::String("value1".to_string()),
        );
        context.set_metadata(
            "key2".to_string(),
            serde_json::Value::Number(serde_json::Number::from(42)),
        );

        assert_eq!(
            context.get_metadata("key1"),
            Some(&serde_json::Value::String("value1".to_string()))
        );
        assert_eq!(
            context.get_metadata("key2"),
            Some(&serde_json::Value::Number(serde_json::Number::from(42)))
        );
        assert_eq!(context.get_metadata("nonexistent"), None);

        // Test removing metadata
        let removed = context.remove_metadata("key1");
        assert_eq!(
            removed,
            Some(serde_json::Value::String("value1".to_string()))
        );
        assert_eq!(context.get_metadata("key1"), None);

        // Test metadata access
        assert_eq!(context.metadata().len(), 1);
    }

    #[tokio::test]
    async fn test_node_successful_execution() {
        let backend = TestNode::new("test message", Action::simple("continue"));
        let mut node = Node::new(backend);
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_node_prep_failure() {
        let backend = TestNode::new("test message", Action::simple("continue")).with_prep_failure();
        let mut node = Node::new(backend);
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
        // Fail once, then succeed
        let backend = TestNode::new("test message", Action::simple("continue"))
            .with_exec_failure(1)
            .with_retries(2);
        let mut node = Node::new(backend);
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_node_exec_retry_exhausted() {
        // Fail more times than retries available
        let backend = TestNode::new("test message", Action::simple("continue"))
            .with_exec_failure(3)
            .with_retries(2);
        let mut node = Node::new(backend);
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
        let backend = TestNode::new("test message", Action::simple("continue")).with_post_failure();
        let mut node = Node::new(backend);
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NodeError::ExecutionError(msg) => assert!(msg.contains("Post failed")),
            _ => panic!("Expected ExecutionError"),
        }
    }

    #[tokio::test]
    async fn test_node_backend_access() {
        let backend = TestNode::new("test message", Action::simple("continue"));
        let mut node = Node::new(backend);

        // Test immutable access
        assert_eq!(node.backend().message, "test message");

        // Test mutable access
        node.backend_mut().message = "modified message".to_string();
        assert_eq!(node.backend().message, "modified message");
    }

    #[test]
    fn test_node_error_types() {
        let exec_error = NodeError::ExecutionError("test exec error".to_string());
        assert_eq!(exec_error.to_string(), "Execution error: test exec error");

        let storage_error = NodeError::StorageError("test storage error".to_string());
        assert_eq!(
            storage_error.to_string(),
            "Storage error: test storage error"
        );

        let validation_error = NodeError::ValidationError("test validation error".to_string());
        assert_eq!(
            validation_error.to_string(),
            "Validation error: test validation error"
        );

        let prep_error = NodeError::PrepError("test prep error".to_string());
        assert_eq!(prep_error.to_string(), "Preparation error: test prep error");
    }

    #[test]
    fn test_node_error_from_string() {
        let error: NodeError = "test error".into();
        match error {
            NodeError::ExecutionError(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected ExecutionError"),
        }

        let error: NodeError = "test error".to_string().into();
        match error {
            NodeError::ExecutionError(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected ExecutionError"),
        }
    }

    // Test custom NodeBackend implementation with fallback
    struct FallbackTestNode {
        should_fail: bool,
        fallback_result: String,
    }

    impl FallbackTestNode {
        fn new(should_fail: bool, fallback_result: String) -> Self {
            Self {
                should_fail,
                fallback_result,
            }
        }
    }

    #[async_trait]
    impl NodeBackend<MemoryStorage> for FallbackTestNode {
        type PrepResult = String;
        type ExecResult = String;
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<MemoryStorage>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            Ok("prep_result".to_string())
        }

        async fn exec(
            &mut self,
            _prep_result: Self::PrepResult,
            _context: &ExecutionContext,
        ) -> Result<Self::ExecResult, Self::Error> {
            if self.should_fail {
                Err(NodeError::ExecutionError("exec failed".to_string()))
            } else {
                Ok("exec_result".to_string())
            }
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

        fn max_retries(&self) -> usize {
            0 // No retries, go straight to fallback
        }
    }

    #[tokio::test]
    async fn test_node_exec_fallback() {
        let backend = FallbackTestNode::new(true, "fallback_result".to_string());
        let mut node = Node::new(backend);
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_execution_context_in_node_execution() {
        struct ContextTestNode {
            execution_ids: Arc<Mutex<Vec<String>>>,
        }

        impl ContextTestNode {
            fn new() -> Self {
                Self {
                    execution_ids: Arc::new(Mutex::new(Vec::new())),
                }
            }
        }

        #[async_trait]
        impl NodeBackend<MemoryStorage> for ContextTestNode {
            type PrepResult = String;
            type ExecResult = String;
            type Error = NodeError;

            async fn prep(
                &mut self,
                _store: &SharedStore<MemoryStorage>,
                context: &ExecutionContext,
            ) -> Result<Self::PrepResult, Self::Error> {
                self.execution_ids
                    .lock()
                    .unwrap()
                    .push(context.execution_id().to_string());
                Ok("prep".to_string())
            }

            async fn exec(
                &mut self,
                _prep_result: Self::PrepResult,
                context: &ExecutionContext,
            ) -> Result<Self::ExecResult, Self::Error> {
                // Verify the same execution ID is used throughout
                let ids = self.execution_ids.lock().unwrap();
                assert_eq!(ids.last().unwrap(), context.execution_id());
                Ok("exec".to_string())
            }

            async fn post(
                &mut self,
                _store: &mut SharedStore<MemoryStorage>,
                _prep_result: Self::PrepResult,
                _exec_result: Self::ExecResult,
                context: &ExecutionContext,
            ) -> Result<Action, Self::Error> {
                // Verify the same execution ID is used throughout
                let ids = self.execution_ids.lock().unwrap();
                assert_eq!(ids.last().unwrap(), context.execution_id());
                Ok(Action::simple("continue"))
            }
        }

        let backend = ContextTestNode::new();
        let mut node = Node::new(backend);
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let result = node.run(&mut store).await;
        assert!(result.is_ok());
    }
}
