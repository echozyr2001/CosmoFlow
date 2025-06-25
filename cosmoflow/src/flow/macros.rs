#![deny(missing_docs)]
//! # Flow Macros - CosmoFlow Workflow Construction Macros
//!
//! This module provides convenient macros for building CosmoFlow workflows with minimal boilerplate.
//! These macros are designed to work with the minimal feature set and don't depend on any builtin modules.
//!
//! ## Key Features
//!
//! - **flow!**: Declarative workflow construction with automatic routing
//! - **Explicit Terminal Routes**: Support for the new terminal route system
//! - **Type-Safe Workflow Termination**: No hidden terminal actions
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cosmoflow::flow::macros::*;
//! use cosmoflow::shared_store::backends::MemoryStorage;
//!
//! // Build a workflow with explicit termination
//! let workflow = flow! {
//!     storage: MemoryStorage,
//!     start: "start",
//!     nodes: {
//!         "start": StartNode,
//!         "end": EndNode,
//!     },
//!     routes: {
//!         "start" - "next" => "end",
//!     },
//!     terminals: {
//!         "end" - "complete",
//!     }
//! };
//! ```

/// Declarative workflow construction with explicit terminal routes and type inference.
///
/// This macro provides a clean, declarative syntax for building CosmoFlow workflows.
/// It automatically handles node registration, routing setup, and explicit terminal route
/// configuration with the new type-safe termination system.
///
/// # Syntax
///
/// ## Structured syntax with explicit terminal routes:
/// ```rust,ignore
/// flow! {
///     storage: StorageType,
///     start: "start_node_id",
///     nodes: {
///         "node_id" : NodeBackend,
///         "other_id" : OtherBackend,
///         "final_id" : FinalBackend,
///     },
///     routes: {
///         "node_id" - "custom_action" => "other_id",
///         "other_id" - "next" => "final_id",
///     },
///     terminals: {
///         "final_id" - "complete",
///         "other_id" - "error",
///     }
/// }
/// ```
///
/// ## Structured syntax with conditional terminal routes:
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
///     },
///     terminals: {
///         "other_id" - "complete",
///     }
/// }
/// ```
///
/// # Arguments
///
/// * `storage` - The storage backend type to use
/// * `start` - The ID of the starting node (for structured syntax)
/// * `nodes` - Node definitions in the format: `"id" : Backend`
/// * `routes` - Route definitions in the format: `"from" - "action" => "to"`
/// * `terminals` - Terminal route definitions in the format: `"from" - "action"`
///
/// # Features
///
/// - **Explicit Terminal Routes**: Full support for the new terminal route system
/// - **Type-Safe Termination**: No hidden terminal actions; all termination is explicit
/// - **Custom Actions**: Supports custom action names for routing (e.g., "default", "error", "success")
/// - **Type Safety**: Compile-time storage type checking
/// - **Clean Syntax**: Declarative workflow definition
/// - **Flexible**: Supports any Node implementation
///
/// # Examples
///
/// ## Structured Workflow with Explicit Terminal Routes
/// ```rust,ignore
/// use cosmoflow::flow::macros::flow;
/// use cosmoflow::shared_store::backends::MemoryStorage;
/// use cosmoflow::action::Action;
/// use cosmoflow::node::Node;
///
/// // Define some nodes (using standard Node implementations)
/// struct StartNode;
/// struct ProcessNode;
/// struct EndNode;
///
/// // Node implementations would be defined here...
///
/// // Build the workflow with explicit termination
/// let workflow = flow! {
///     storage: MemoryStorage,
///     start: "start",
///     nodes: {
///         "start": StartNode,
///         "process": ProcessNode,
///         "end": EndNode,
///     },
///     routes: {
///         "start" - "next" => "process",
///         "process" - "next" => "end",
///     },
///     terminals: {
///         "end" - "complete",
///     }
/// };
/// ```
///
/// ## Workflow with Error Handling and Multiple Terminal Routes
/// ```rust,ignore
/// use cosmoflow::flow::macros::flow;
/// use cosmoflow::shared_store::backends::MemoryStorage;
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
///         "input": DataProcessor,
///         "error": ErrorHandler,
///         "success": SuccessHandler,
///     },
///     routes: {
///         "input" - "success" => "success",
///         "input" - "error" => "error",
///     },
///     terminals: {
///         "success" - "complete",
///         "error" - "failed",
///     }
/// };
/// ```
///
/// ## Simple Linear Workflow
/// ```rust,ignore
/// use cosmoflow::flow::macros::flow;
/// use cosmoflow::shared_store::backends::MemoryStorage;
/// use cosmoflow::action::Action;
/// use cosmoflow::node::Node;
///
/// // Simple workflow with linear progression
/// struct ProcessingNode;
/// struct FinalNode;
///
/// // Node implementations would be defined here...
///
/// let workflow = flow! {
///     storage: MemoryStorage,
///     start: "process",
///     nodes: {
///         "process": ProcessingNode,
///         "final": FinalNode,
///     },
///     routes: {
///         "process" - "next" => "final",
///     },
///     terminals: {
///         "final" - "complete",
///     }
/// };
/// ```
#[macro_export]
macro_rules! flow {
    // Structured syntax with explicit start, nodes, routes, and terminal routes
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
        },
        terminals: {
            $(
                $term_from:literal - $term_action:literal
            ),* $(,)?
        } $(,)?
    ) => {
        {
            let mut builder = $crate::FlowBuilder::<$storage>::new()
                .start_node($start);

            $(
                builder = builder.node($id, $backend);
            )*

            $(
                builder = builder.route($from, $action, $to);
            )*

            $(
                builder = builder.terminal_route($term_from, $term_action);
            )*

            builder.build()
        }
    };

    // Structured syntax with only routes (no terminal routes - for backward compatibility)
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
            let mut builder = $crate::FlowBuilder::<$storage>::new()
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

/// Declarative async workflow construction with explicit terminal routes and type inference.
///
/// This macro provides a clean, declarative syntax for building async CosmoFlow workflows.
/// It automatically handles node registration, routing setup, and explicit terminal route
/// configuration with the new type-safe termination system.
///
/// **Note**: This macro creates async flows. For sync flows, use `flow!` macro.
/// This macro is only available when the "async" feature is enabled.
///
/// # Syntax
///
/// Same as `flow!` macro but creates async flows instead of sync flows.
///
/// # Example
///
/// ```rust,ignore
/// #[cfg(feature = "async")]
/// let workflow = async_flow! {
///     storage: MemoryStorage,
///     start: "start",
///     nodes: {
///         "start": AsyncStartNode,
///         "end": AsyncEndNode,
///     },
///     routes: {
///         "start" - "next" => "end",
///     },
///     terminals: {
///         "end" - "complete",
///     }
/// };
/// ```
#[cfg(feature = "async")]
#[macro_export]
macro_rules! async_flow {
    // Structured syntax with explicit terminal routes
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
        },
        terminals: {
            $(
                $term_from:literal - $term_action:literal
            ),* $(,)?
        } $(,)?
    ) => {
        {
            let mut builder = $crate::flow::r#async::FlowBuilder::<$storage>::new()
                .start_node($start);

            $(
                builder = builder.node($id, $backend);
            )*

            $(
                builder = builder.route($from, $action, $to);
            )*

            $(
                builder = builder.terminal_route($term_from, $term_action);
            )*

            builder.build()
        }
    };

    // Structured syntax with only routes (no terminal routes - for backward compatibility)
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
            let mut builder = $crate::flow::r#async::FlowBuilder::<$storage>::new()
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
    use crate::shared_store::SharedStore;
    use crate::shared_store::backends::MemoryStorage;

    // Import the correct Node trait based on features
    #[cfg(not(feature = "async"))]
    use crate::node::Node;
    #[cfg(feature = "async")]
    use crate::node::r#async::Node;

    #[cfg(not(feature = "async"))]
    use crate::flow::{FlowBackend, FlowBuilder};

    #[cfg(feature = "async")]
    use crate::flow::r#async::{FlowBackend, FlowBuilder};

    // Test node implementations - sync version
    #[cfg(not(feature = "async"))]
    mod sync_nodes {
        use super::*;

        pub struct TestStartNode;

        impl<S: SharedStore> Node<S> for TestStartNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            fn prep(
                &mut self,
                _: &S,
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn exec(
                &mut self,
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn post(
                &mut self,
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple("next"))
            }
        }

        pub struct TestProcessNode;

        impl<S: SharedStore> Node<S> for TestProcessNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            fn prep(
                &mut self,
                _: &S,
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn exec(
                &mut self,
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn post(
                &mut self,
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple("next"))
            }
        }

        pub struct TestEndNode;

        impl<S: SharedStore> Node<S> for TestEndNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            fn prep(
                &mut self,
                _: &S,
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn exec(
                &mut self,
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn post(
                &mut self,
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple("complete"))
            }
        }

        pub struct TestCustomNode {
            pub action: String,
        }

        impl TestCustomNode {
            pub fn new(action: impl Into<String>) -> Self {
                Self {
                    action: action.into(),
                }
            }
        }

        impl<S: SharedStore> Node<S> for TestCustomNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            fn prep(
                &mut self,
                _: &S,
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn exec(
                &mut self,
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<(), Self::Error> {
                Ok(())
            }

            fn post(
                &mut self,
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple(&self.action))
            }
        }
    }

    // Test node implementations - async version
    #[cfg(feature = "async")]
    mod async_nodes {
        use super::*;
        use async_trait::async_trait;

        pub struct TestStartNode;

        #[async_trait]
        impl<S: SharedStore + Send + Sync> Node<S> for TestStartNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            async fn prep(
                &mut self,
                _: &S,
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
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple("next"))
            }
        }

        pub struct TestProcessNode;

        #[async_trait]
        impl<S: SharedStore + Send + Sync> Node<S> for TestProcessNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            async fn prep(
                &mut self,
                _: &S,
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
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple("next"))
            }
        }

        pub struct TestEndNode;

        #[async_trait]
        impl<S: SharedStore + Send + Sync> Node<S> for TestEndNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            async fn prep(
                &mut self,
                _: &S,
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
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple("complete"))
            }
        }

        pub struct TestCustomNode {
            pub action: String,
        }

        impl TestCustomNode {
            pub fn new(action: impl Into<String>) -> Self {
                Self {
                    action: action.into(),
                }
            }
        }

        #[async_trait]
        impl<S: SharedStore + Send + Sync> Node<S> for TestCustomNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = crate::node::NodeError;

            async fn prep(
                &mut self,
                _: &S,
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
                _: &mut S,
                _: (),
                _: (),
                _: &crate::node::ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(Action::simple(&self.action))
            }
        }
    }

    // Use appropriate node implementations based on feature
    #[cfg(feature = "async")]
    use async_nodes::*;
    #[cfg(not(feature = "async"))]
    use sync_nodes::*;

    // Test: Structured syntax with explicit terminal routes
    #[test]
    fn test_flow_macro_with_terminal_routes() {
        let _workflow = flow! {
            storage: MemoryStorage,
            start: "entry",
            nodes: {
                "entry": TestCustomNode::new("default"),
                "process": TestCustomNode::new("success"),
                "error": TestEndNode,
                "success": TestEndNode,
            },
            routes: {
                "entry" - "default" => "process",
                "process" - "success" => "success",
                "entry" - "error" => "error",
            },
            terminals: {
                "success" - "complete",
                "error" - "failed",
            }
        };
        // Test new terminal routes syntax
    }

    // Test: Legacy syntax without terminal routes (backward compatibility)
    #[test]
    fn test_flow_macro_legacy_syntax() {
        let _workflow = flow! {
            storage: MemoryStorage,
            start: "entry",
            nodes: {
                "entry": TestCustomNode::new("default"),
                "process": TestCustomNode::new("success"),
                "error": TestEndNode,
                "success": TestEndNode,
            },
            routes: {
                "entry" - "default" => "process",
                "process" - "success" => "success",
                "entry" - "error" => "error",
            }
        };
        // Test legacy syntax for backward compatibility
    }

    // Sync execution tests
    #[cfg(not(feature = "async"))]
    mod sync_tests {
        use super::*;

        #[test]
        fn test_flow_macro_with_terminal_routes_execution() {
            let mut workflow = flow! {
                storage: MemoryStorage,
                start: "entry",
                nodes: {
                    "entry": TestCustomNode::new("default"),
                    "process": TestCustomNode::new("continue"),
                    "end": TestEndNode,
                },
                routes: {
                    "entry" - "default" => "process",
                    "process" - "continue" => "end",
                },
                terminals: {
                    "end" - "complete",
                }
            };

            let mut store = MemoryStorage::new();
            let result = workflow.execute(&mut store);

            assert!(result.is_ok(), "Flow macro execution should succeed");
            let execution_result = result.unwrap();
            assert!(execution_result.success);
            assert_eq!(execution_result.steps_executed, 3);
            assert_eq!(
                execution_result.execution_path,
                vec!["entry", "process", "end"]
            );
        }

        #[test]
        fn test_flow_macro_legacy_execution() {
            let mut workflow = FlowBuilder::new()
                .start_node("entry")
                .node("entry", TestCustomNode::new("default"))
                .node("process", TestCustomNode::new("continue"))
                .node("end", TestEndNode)
                .route("entry", "default", "process")
                .route("process", "continue", "end")
                .terminal_route("end", "complete")
                .build();

            let mut store = MemoryStorage::new();
            let result = workflow.execute(&mut store);

            assert!(result.is_ok(), "Legacy flow execution should succeed");
            let execution_result = result.unwrap();
            assert!(execution_result.success);
            assert_eq!(execution_result.steps_executed, 3);
            assert_eq!(
                execution_result.execution_path,
                vec!["entry", "process", "end"]
            );
        }

        #[test]
        fn test_flow_macro_multiple_terminal_routes() {
            let mut success_workflow = flow! {
                storage: MemoryStorage,
                start: "check",
                nodes: {
                    "check": TestCustomNode::new("success"),
                    "success_handler": TestEndNode,
                    "error_handler": TestEndNode,
                },
                routes: {
                    "check" - "success" => "success_handler",
                    "check" - "error" => "error_handler",
                },
                terminals: {
                    "success_handler" - "complete",
                    "error_handler" - "failed",
                }
            };

            let mut store = MemoryStorage::new();
            let result = success_workflow.execute(&mut store);

            assert!(
                result.is_ok(),
                "Multiple terminal routes workflow should succeed"
            );
            let execution_result = result.unwrap();
            assert!(execution_result.success);
            assert_eq!(
                execution_result.execution_path,
                vec!["check", "success_handler"]
            );
        }
    }

    // Async execution tests
    #[cfg(feature = "async")]
    mod async_tests {
        use super::*;

        #[tokio::test]
        async fn test_flow_macro_with_terminal_routes_execution() {
            let mut workflow = flow! {
                storage: MemoryStorage,
                start: "entry",
                nodes: {
                    "entry": TestCustomNode::new("default"),
                    "process": TestCustomNode::new("continue"),
                    "end": TestEndNode,
                },
                routes: {
                    "entry" - "default" => "process",
                    "process" - "continue" => "end",
                },
                terminals: {
                    "end" - "complete",
                }
            };

            let mut store = MemoryStorage::new();
            let result = workflow.execute(&mut store).await;

            assert!(result.is_ok(), "Flow macro execution should succeed");
            let execution_result = result.unwrap();
            assert!(execution_result.success);
            assert_eq!(execution_result.steps_executed, 3);
            assert_eq!(
                execution_result.execution_path,
                vec!["entry", "process", "end"]
            );
        }

        #[tokio::test]
        async fn test_flow_macro_legacy_execution() {
            let mut workflow = FlowBuilder::new()
                .start_node("entry")
                .node("entry", TestCustomNode::new("default"))
                .node("process", TestCustomNode::new("continue"))
                .node("end", TestEndNode)
                .route("entry", "default", "process")
                .route("process", "continue", "end")
                .terminal_route("end", "complete")
                .build();

            let mut store = MemoryStorage::new();
            let result = workflow.execute(&mut store).await;

            assert!(result.is_ok(), "Legacy flow execution should succeed");
            let execution_result = result.unwrap();
            assert!(execution_result.success);
            assert_eq!(execution_result.steps_executed, 3);
            assert_eq!(
                execution_result.execution_path,
                vec!["entry", "process", "end"]
            );
        }

        #[tokio::test]
        async fn test_flow_macro_multiple_terminal_routes() {
            let mut success_workflow = flow! {
                storage: MemoryStorage,
                start: "check",
                nodes: {
                    "check": TestCustomNode::new("success"),
                    "success_handler": TestEndNode,
                    "error_handler": TestEndNode,
                },
                routes: {
                    "check" - "success" => "success_handler",
                    "check" - "error" => "error_handler",
                },
                terminals: {
                    "success_handler" - "complete",
                    "error_handler" - "failed",
                }
            };

            let mut store = MemoryStorage::new();
            let result = success_workflow.execute(&mut store).await;

            assert!(
                result.is_ok(),
                "Multiple terminal routes workflow should succeed"
            );
            let execution_result = result.unwrap();
            assert!(execution_result.success);
            assert_eq!(
                execution_result.execution_path,
                vec!["check", "success_handler"]
            );
        }
    }
}
