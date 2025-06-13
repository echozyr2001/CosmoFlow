#![deny(missing_docs)]
//! # Flow Macros - CosmoFlow Workflow Construction Macros
//!
//! This module provides convenient macros for building CosmoFlow workflows with minimal boilerplate.
//! These macros are designed to work with the minimal feature set and don't depend on any builtin modules.
//!
//! ## Key Features
//!
//! - **flow!**: Declarative workflow construction with automatic routing
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cosmoflow::flow::macros::*;
//! use cosmoflow::storage::backends::MemoryStorage;
//!
//! // Build a workflow
//! let workflow = flow! {
//!     storage: MemoryStorage,
//!     "start" => StartNode,
//!     "end" => EndNode,
//! };
//! ```

/// Declarative workflow construction with automatic routing and type inference.
///
/// This macro provides a clean, declarative syntax for building CosmoFlow workflows.
/// It automatically handles node registration, routing setup, and storage configuration.
///
/// # Syntax
///
/// ## Structured syntax with custom action routing:
/// ```rust,ignore
/// flow! {
///     storage: StorageType,
///     start: "start_node_id",
///     nodes: {
///         "node_id" : NodeBackend,
///         "other_id" : OtherBackend,
///     },
///     routes: {
///         "node_id" - "custom_action" => "other_id",
///         "other_id" - "next" => "final_id",
///     }
/// }
/// ```
///
/// ## Structured syntax with default "next" action routing:
/// ```rust,ignore
/// flow! {
///     storage: StorageType,
///     start: "start_node_id",
///     nodes: {
///         "node_id" : NodeBackend,
///         "other_id" : OtherBackend,
///     },
///     routes: {
///         "node_id" - "next" => "other_id",
///     }
/// }
/// ```
///
/// ## Simplified syntax for linear workflows:
/// ```rust,ignore
/// flow! {
///     storage: StorageType,
///     "start" => StartBackend,
///     "process" => ProcessBackend,
///     "end" => EndBackend,
/// }
/// ```
///
/// # Arguments
///
/// * `storage` - The storage backend type to use
/// * `start` - The ID of the starting node (for structured syntax)
/// * `nodes` - Node definitions in the format: `"id" : Backend`
/// * `routes` - Route definitions in the format: `"from" - "action" => "to"`
///
/// # Features
///
/// - **Flexible Syntax**: Supports both structured and simplified syntax
/// - **Custom Actions**: Supports custom action names for routing (e.g., "default", "error", "success")
/// - **Type Safety**: Compile-time storage type checking
/// - **Clean Syntax**: Declarative workflow definition
/// - **Flexible**: Supports any Node implementation
///
/// # Examples
///
/// ## Simplified Linear Workflow
/// ```rust,ignore
/// use cosmoflow::flow::macros::flow;
/// use cosmoflow::storage::backends::MemoryStorage;
/// use cosmoflow::action::Action;
/// use cosmoflow::node::Node;
///
/// // Define some nodes (using standard Node implementations)
/// struct StartNode;
/// struct EndNode;
///
/// // Node implementations would be defined here...
///
/// // Build the workflow - nodes will be auto-routed in sequence using "next" action
/// let workflow = flow! {
///     storage: MemoryStorage,
///     "start" => StartNode,
///     "end" => EndNode,
/// };
/// ```
///
/// ## Structured Workflow with Custom Action Routing
/// ```rust,ignore
/// use cosmoflow::flow::macros::flow;
/// use cosmoflow::storage::backends::MemoryStorage;
/// use cosmoflow::action::Action;
/// use cosmoflow::node::Node;
///
/// // Custom node with complex logic
/// struct DataProcessor;
/// struct ErrorHandler;
/// struct SuccessHandler;
///
/// // Node implementations would be defined here...
///
/// let workflow = flow! {
///     storage: MemoryStorage,
///     start: "input",
///     nodes: {
///         "input" : DataProcessor,
///         "error" : ErrorHandler,
///         "success" : SuccessHandler,
///     },
///     routes: {
///         "input" - "default" => "success",
///         "input" - "error" => "error",
///     }
/// };
/// ```
///
/// ## Structured Workflow with Default "next" Action Routing
/// ```rust,ignore
/// use cosmoflow::flow::macros::flow;
/// use cosmoflow::storage::backends::MemoryStorage;
/// use cosmoflow::action::Action;
/// use cosmoflow::node::Node;
///
/// // Custom node with complex logic
/// struct DataProcessor;
/// struct EndNode;
///
/// // Node implementations would be defined here...
///
/// let workflow = flow! {
///     storage: MemoryStorage,
///     start: "input",
///     nodes: {
///         "input" : DataProcessor,
///         "output" : EndNode,
///     },
///     routes: {
///         "input" - "next" => "output",
///     }
/// };
/// ```
#[macro_export]
macro_rules! flow {
    // Structured syntax with explicit start, nodes, and routes using custom actions
    // Syntax: "from" : "action" => "to" (using literal tokens)
    (
        storage: $storage:ty,
        start: $start:expr,
        nodes: {
            $(
                $id:literal : $backend:expr
            ),* $(,)?
        },
        routes: {
            $(
                $from:literal - $action:literal => $to:literal
            ),* $(,)?
        } $(,)?
    ) => {
        {
            let mut builder = $crate::flow::FlowBuilder::<$storage>::new()
                .start_node($start);

            $(
                builder = builder.node($id, $backend);
            )*

            $(
                builder = builder.route($from, $action, $to);
            )*

            builder.build()
        }
    };
}

// Re-export macro for easier access
pub use flow;

#[cfg(all(test, feature = "storage-memory"))]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::flow::FlowBackend;
    use crate::node::Node;
    use crate::shared_store::SharedStore;
    use crate::storage::backends::MemoryStorage;

    // Test node implementations
    struct TestStartNode;

    #[async_trait::async_trait]
    impl<S: crate::storage::StorageBackend + Send + Sync> Node<S> for TestStartNode {
        type PrepResult = ();
        type ExecResult = ();
        type Error = crate::node::NodeError;

        async fn prep(
            &mut self,
            _: &crate::shared_store::SharedStore<S>,
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn exec(
            &mut self,
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn post(
            &mut self,
            _: &mut crate::shared_store::SharedStore<S>,
            _: (),
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(Action::simple("next"))
        }
    }

    struct TestProcessNode;

    #[async_trait::async_trait]
    impl<S: crate::storage::StorageBackend + Send + Sync> Node<S> for TestProcessNode {
        type PrepResult = ();
        type ExecResult = ();
        type Error = crate::node::NodeError;

        async fn prep(
            &mut self,
            _: &crate::shared_store::SharedStore<S>,
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn exec(
            &mut self,
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn post(
            &mut self,
            _: &mut crate::shared_store::SharedStore<S>,
            _: (),
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(Action::simple("next"))
        }
    }

    struct TestEndNode;

    #[async_trait::async_trait]
    impl<S: crate::storage::StorageBackend + Send + Sync> Node<S> for TestEndNode {
        type PrepResult = ();
        type ExecResult = ();
        type Error = crate::node::NodeError;

        async fn prep(
            &mut self,
            _: &crate::shared_store::SharedStore<S>,
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn exec(
            &mut self,
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn post(
            &mut self,
            _: &mut crate::shared_store::SharedStore<S>,
            _: (),
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(Action::simple("complete"))
        }
    }

    struct TestCustomNode {
        action: String,
    }

    impl TestCustomNode {
        fn new(action: impl Into<String>) -> Self {
            Self {
                action: action.into(),
            }
        }
    }

    #[async_trait::async_trait]
    impl<S: crate::storage::StorageBackend + Send + Sync> Node<S> for TestCustomNode {
        type PrepResult = ();
        type ExecResult = ();
        type Error = crate::node::NodeError;

        async fn prep(
            &mut self,
            _: &crate::shared_store::SharedStore<S>,
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn exec(
            &mut self,
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn post(
            &mut self,
            _: &mut crate::shared_store::SharedStore<S>,
            _: (),
            _: (),
            _: &crate::node::ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(Action::simple(&self.action))
        }
    }

    // Test 8b: Structured syntax with colon syntax for action routing (literal version)
    #[test]
    fn test_flow_macro_colon_action_routing() {
        let _workflow = flow! {
            storage: MemoryStorage,
            start: "entry",
            nodes: {
                "entry" : TestCustomNode::new("default"),
                "process" : TestCustomNode::new("success"),
                "error" : TestEndNode,
                "success" : TestEndNode,
            },
            routes: {
                "entry" - "default" => "process",
                "process" - "success" => "success",
                "entry" - "error" => "error",
            }
        };
        // Test colon syntax with literal tokens
    }

    // Test 13a: Colon syntax execution test
    #[tokio::test]
    async fn test_flow_macro_colon_syntax_execution() {
        let mut workflow = flow! {
            storage: MemoryStorage,
            start: "entry",
            nodes: {
                "entry" : TestCustomNode::new("default"),
                "process" : TestCustomNode::new("continue"),
                "end" : TestEndNode,
            },
            routes: {
                "entry" - "default" => "process",
                "process" - "continue" => "end",
            }
        };

        let mut store = SharedStore::with_storage(MemoryStorage::new());
        let result = workflow.execute(&mut store).await;

        assert!(result.is_ok());
        let execution_result = result.unwrap();
        assert!(execution_result.success);
        assert_eq!(execution_result.steps_executed, 3);
        assert_eq!(
            execution_result.execution_path,
            vec!["entry", "process", "end"]
        );
    }
}
