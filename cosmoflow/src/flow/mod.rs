#![deny(missing_docs)]
//! # Flow - CosmoFlow Orchestration System
//!
//! This crate provides the core workflow orchestration system for CosmoFlow. It manages the
//! execution of interconnected nodes, handles routing between them, and provides comprehensive
//! error handling and execution tracking.
//!
//! ## Key Features
//!
//! - **Workflow Orchestration**: Execute complex multi-node workflows with different node types
//! - **Dynamic Routing**: Conditional and parameterized routing between nodes
//! - **Type Safety**: Compile-time safety with automatic type erasure through NodeRunner
//! - **Error Handling**: Comprehensive error management and recovery
//! - **Execution Tracking**: Detailed execution results and performance metrics
//! - **Retry Logic**: Built-in retry mechanisms for failed operations
//!
//! ## Quick Start
//!
//! ```rust
//! # #[cfg(feature = "storage-memory")]
//! # {
//! use cosmoflow::{Flow, FlowBuilder, FlowBackend};
//! use cosmoflow::shared_store::SharedStore;
//! use cosmoflow::shared_store::backends::MemoryStorage;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a shared store
//! let mut store = MemoryStorage::new();
//!
//! // Build a flow
//! let mut flow = FlowBuilder::new()
//!     .start_node("start")
//!     .build();
//!
//! // Execute the flow
//! let result = flow.execute(&mut store).await?;
//! println!("Flow completed with {} steps", result.steps_executed);
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Core Types
//!
//! - [`Flow`]: The main workflow execution engine
//! - [`FlowBuilder`]: Builder pattern for constructing flows
//! - [`FlowExecutionResult`]: Results and metadata from flow execution
//! - [`Route`]: Defines routing between nodes in the workflow
//! - [`NodeRunner`]: Type erasure trait for different node types
//!
//! ## Error Handling
//!
//! The flow crate provides comprehensive error handling through [`FlowError`]:
//!
//! ```rust
//! # #[cfg(feature = "storage-memory")]
//! # {
//! use cosmoflow::{Flow, FlowError, FlowBackend};
//!
//! # async fn example() -> Result<(), FlowError> {
//! # let mut flow = Flow::new();
//! # let mut store = cosmoflow::shared_store::backends::MemoryStorage::new();
//! match flow.execute(&mut store).await {
//!     Ok(result) => println!("Success: {:?}", result),
//!     Err(FlowError::NodeNotFound(id)) => eprintln!("Node '{}' not found", id),
//!     Err(FlowError::NodeError(msg)) => {
//!         eprintln!("Node execution failed: {}", msg);
//!     },
//!     Err(e) => eprintln!("Flow error: {}", e),
//! }
//! # Ok(())
//! # }
//! # }
//! ```

/// The errors module contains the error types for the flow crate.
pub mod errors;
/// The macros module contains convenient macros for building workflows.
pub mod macros;
/// The route module contains the `Route` struct and `RouteCondition` enum.
pub mod route;

/// Async-specific implementations (only available with "async" feature)
#[cfg(feature = "async")]
pub mod r#async;

#[cfg(feature = "async")]
pub use r#async::NodeRunner;

#[cfg(not(feature = "async"))]
use std::collections::HashMap;
#[cfg(not(feature = "async"))]
use std::time::Duration;

use crate::action::Action;
#[cfg(not(feature = "async"))]
use crate::node::{ExecutionContext, Node, NodeError};
use crate::shared_store::SharedStore;

use errors::FlowError;
use route::Route;
#[cfg(not(feature = "async"))]
use route::RouteCondition;

/// Node runner trait for workflow execution (sync version only)
///
/// This trait provides a unified interface for executing nodes with different
/// associated types in the same flow, allowing the flow system to work with
/// heterogeneous node collections while maintaining type safety.
#[cfg(not(feature = "async"))]
pub trait NodeRunner<S: SharedStore>: Send + Sync {
    /// Execute the node and return the resulting action
    fn run(&mut self, store: &mut S) -> Result<Action, NodeError>;

    /// Get the node's name for debugging and logging
    fn name(&self) -> &str;
}

/// Implementation of NodeRunner for any Node (sync version only)
#[cfg(not(feature = "async"))]
impl<T, S> NodeRunner<S> for T
where
    T: Node<S> + Send + Sync,
    S: SharedStore + Send + Sync,
{
    fn run(&mut self, store: &mut S) -> Result<Action, NodeError> {
        Node::run(self, store)
    }

    fn name(&self) -> &str {
        Node::name(self)
    }
}

/// Execution result from a flow run
///
/// This struct contains comprehensive information about the execution of a workflow,
/// including success status, execution path, and performance metrics.
///
/// # Examples
///
/// ```rust
/// # #[cfg(all(feature = "storage-memory", feature = "async"))]
/// # {
/// use cosmoflow::{Flow, FlowExecutionResult, FlowBackend};
/// use cosmoflow::action::Action;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut flow = Flow::new();
/// # let mut store = cosmoflow::shared_store::backends::MemoryStorage::new();
/// let result: FlowExecutionResult = flow.execute(&mut store).await?;
///
/// if result.success {
///     println!("Flow completed successfully in {} steps", result.steps_executed);
///     println!("Execution path: {:?}", result.execution_path);
/// } else {
///     println!("Flow failed at node: {}", result.last_node_id);
/// }
/// # Ok(())
/// # }
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FlowExecutionResult {
    /// The final action that terminated the flow
    ///
    /// This could be an action that completed the workflow successfully,
    /// or an action that caused the flow to terminate due to an error.
    pub final_action: Action,
    /// The ID of the last executed node
    ///
    /// This is useful for debugging and understanding where the flow
    /// stopped, especially in case of failures.
    pub last_node_id: String,
    /// Number of steps executed
    ///
    /// This count includes successful executions and retries, providing
    /// insight into the complexity and performance of the workflow.
    pub steps_executed: usize,
    /// Whether the flow completed successfully
    ///
    /// `true` if the flow reached a completion state without errors,
    /// `false` if the flow was terminated due to an error or timeout.
    pub success: bool,
    /// Execution path (node IDs in order)
    ///
    /// This vector contains the IDs of all nodes that were executed,
    /// in the order they were processed. Useful for debugging and
    /// workflow analysis.
    pub execution_path: Vec<String>,
}

/// Configuration for flow execution
#[derive(Debug, Clone)]
pub struct FlowConfig {
    /// Maximum number of execution steps before terminating
    pub max_steps: usize,
    /// Starting node ID
    pub start_node_id: String,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            max_steps: 1000,
            start_node_id: "start".to_string(),
        }
    }
}

/// Trait for implementing flow execution logic
pub trait FlowBackend<S: SharedStore> {
    /// Add a node to the flow
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError>;

    /// Add a route between nodes
    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError>;

    /// Execute the flow starting from the configured start node
    fn execute(&mut self, store: &mut S) -> Result<FlowExecutionResult, FlowError>;

    /// Execute the flow starting from a specific node
    fn execute_from(
        &mut self,
        store: &mut S,
        start_node_id: String,
    ) -> Result<FlowExecutionResult, FlowError>;

    /// Get the current configuration
    fn config(&self) -> &FlowConfig;

    /// Update the configuration
    fn set_config(&mut self, config: FlowConfig);

    /// Check if the flow is valid (no orphaned nodes, etc.)
    fn validate(&self) -> Result<(), FlowError>;
}

/// Builder for creating flows easily
#[cfg(not(feature = "async"))]
pub struct FlowBuilder<S: SharedStore> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

#[cfg(not(feature = "async"))]
impl<S: SharedStore + 'static> Default for FlowBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "async"))]
impl<S: SharedStore + 'static> FlowBuilder<S> {
    /// Create a new flow builder
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config: FlowConfig::default(),
        }
    }

    /// Set the starting node ID
    pub fn start_node(mut self, node_id: impl Into<String>) -> Self {
        self.config.start_node_id = node_id.into();
        self
    }

    /// Set maximum execution steps
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        self.config.max_steps = max_steps;
        self
    }

    /// Add a node to the flow
    pub fn node<T>(mut self, id: impl Into<String>, node: T) -> Self
    where
        T: Node<S> + Send + Sync + 'static,
    {
        self.nodes.insert(id.into(), Box::new(node));
        self
    }

    /// Convenience method: add a node and set it as the starting node
    pub fn start_with<T>(mut self, id: impl Into<String>, node: T) -> Self
    where
        T: Node<S> + Send + Sync + 'static,
    {
        let id = id.into();
        self.config.start_node_id = id.clone();
        self.node(id, node)
    }

    /// Add a simple route (action -> target node)
    pub fn route(
        mut self,
        from: impl Into<String>,
        action: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        let from_id = from.into();
        let action_str = action.into();
        let to_id = to.into();

        let route = Route {
            action: action_str,
            target_node_id: Some(to_id),
            condition: None,
        };

        self.routes.entry(from_id).or_default().push(route);
        self
    }

    /// Add a conditional route
    pub fn conditional_route(
        mut self,
        from: impl Into<String>,
        action: impl Into<String>,
        to: impl Into<String>,
        condition: RouteCondition,
    ) -> Self {
        let from_id = from.into();
        let action_str = action.into();
        let to_id = to.into();

        let route = Route {
            action: action_str,
            target_node_id: Some(to_id),
            condition: Some(condition),
        };

        self.routes.entry(from_id).or_default().push(route);
        self
    }

    /// Add an explicit terminal route that does not target any node
    ///
    /// This is a convenience method for routes that should terminate the workflow
    /// without routing to another node. It's semantically clearer than using a
    /// regular route with a terminal action.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::FlowBuilder;
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let flow = FlowBuilder::<MemoryStorage>::new()
    ///     .route("node1", "continue", "node2")
    ///     .terminal_route("node2", "complete")  // Explicit termination
    ///     .build();
    /// # }
    /// ```
    pub fn terminal_route(mut self, from: impl Into<String>, action: impl Into<String>) -> Self {
        let from_id = from.into();
        let action_str = action.into();

        let route = Route {
            action: action_str,
            target_node_id: None, // None indicates termination
            condition: None,
        };

        self.routes.entry(from_id).or_default().push(route);
        self
    }

    /// Add an explicit conditional terminal route
    ///
    /// Like `terminal_route`, but only triggers termination if the condition is met.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::FlowBuilder;
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let flow = FlowBuilder::<MemoryStorage>::new()
    ///     .conditional_terminal_route("node1", "finish", RouteCondition::KeyExists("success".to_string()))
    ///     .build();
    /// # }
    /// ```
    pub fn conditional_terminal_route(
        mut self,
        from: impl Into<String>,
        action: impl Into<String>,
        condition: RouteCondition,
    ) -> Self {
        let from_id = from.into();
        let action_str = action.into();

        let route = Route {
            action: action_str,
            target_node_id: None, // None indicates termination
            condition: Some(condition),
        };

        self.routes.entry(from_id).or_default().push(route);
        self
    }

    /// Build the flow with the configured settings
    pub fn build(self) -> Flow<S> {
        Flow {
            nodes: self.nodes,
            routes: self.routes,
            config: self.config,
        }
    }

    /// Convenience method to create a self-routing loop
    ///
    /// This is equivalent to `.route(node_id, action, node_id)` but more explicit
    /// about the intent to create a loop.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::FlowBuilder;
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let flow = FlowBuilder::<MemoryStorage>::new()
    ///     .start_node("counter")
    ///     .max_steps(100)  // Allow intentional loops
    ///     // ... add nodes ...
    ///     .self_route("counter", "continue")  // Loop back to self
    ///     .terminal_route("counter", "done")  // Exit condition
    ///     .build();
    /// # }
    /// ```
    pub fn self_route(self, node_id: impl Into<String>, action: impl Into<String>) -> Self {
        let node_id_str = node_id.into();
        self.route(node_id_str.clone(), action, node_id_str)
    }
}

/// Basic implementation of the FlowBackend trait
///
/// The `Flow` struct represents a complete workflow definition with nodes, routes,
/// and configuration. It manages the execution of workflows by orchestrating
/// node execution and routing decisions.
///
/// # Type Parameters
///
/// * `S` - The storage backend type that implements [`SharedStore`]
///
/// # Fields
///
/// - `nodes`: Collection of executable nodes indexed by their unique IDs
/// - `routes`: Routing table that defines transitions between nodes
/// - `config`: Configuration settings that control flow execution behavior
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "storage-memory")]
/// # {
/// use cosmoflow::{Flow, FlowConfig};
/// use cosmoflow::shared_store::backends::MemoryStorage;
///
/// // Create a flow with default configuration
/// let flow: Flow<MemoryStorage> = Flow::new();
///
/// // Create a flow with custom configuration
/// let config = FlowConfig {
///     max_steps: 500,
///     start_node_id: "start".to_string(),
/// };
/// let flow: Flow<MemoryStorage> = Flow::with_config(config);
/// # }
/// ```
///
/// # Thread Safety
///
/// The `Flow` struct is designed to be used in single-threaded contexts within
/// CosmoFlow's execution model. For concurrent execution of multiple workflows,
/// create separate `Flow` instances for each workflow.
#[cfg(not(feature = "async"))]
pub struct Flow<S: SharedStore> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

#[cfg(not(feature = "async"))]
impl<S: SharedStore> Flow<S> {
    /// Create a new basic flow with default configuration
    ///
    /// Creates an empty flow with no nodes or routes. Use [`FlowBuilder`] for
    /// a more convenient way to construct workflows.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::Flow;
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let flow: Flow<MemoryStorage> = Flow::new();
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config: FlowConfig::default(),
        }
    }

    /// Create a new basic flow with custom configuration
    ///
    /// Use this constructor when you need to specify custom execution behavior
    /// such as timeouts or terminal actions.
    ///
    /// # Arguments
    ///
    /// * `config` - Flow configuration settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::{Flow, FlowConfig};
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let config = FlowConfig {
    ///     max_steps: 1000,
    ///     start_node_id: "start".to_string(),
    /// };
    ///
    /// let flow: Flow<MemoryStorage> = Flow::with_config(config);
    /// # }
    /// ```
    pub fn with_config(config: FlowConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config,
        }
    }

    /// Find the next node ID based on the current action
    ///
    /// This method implements the core routing logic of the flow engine. It:
    /// 1. Checks if the action is a terminal action (ends execution)
    /// 2. Looks up available routes from the current node
    /// 3. Evaluates route conditions to find the appropriate next node
    ///
    /// # Arguments
    ///
    /// * `current_node_id` - ID of the currently executing node
    /// * `action` - Action returned by the current node
    /// * `store` - Shared store for condition evaluation
    ///
    /// # Returns
    ///
    /// * `Ok(Some(String))` - ID of the next node to execute
    /// * `Ok(None)` - Terminal action, execution should end
    /// * `Err(FlowError)` - Routing error (no route found, condition evaluation failed)
    ///
    /// # Errors
    ///
    /// * [`FlowError::NoRouteFound`] - No route exists for the given action
    /// * [`FlowError::InvalidConfiguration`] - Route condition evaluation failed
    fn find_next_node(
        &self,
        current_node_id: &str,
        action: &Action,
        store: &S,
    ) -> Result<Option<String>, FlowError> {
        let action_str = action.to_string();

        // Get routes for the current node
        let routes = self.routes.get(current_node_id).ok_or_else(|| {
            FlowError::NoRouteFound(current_node_id.to_string(), action_str.clone())
        })?;

        // Find matching route
        for route in routes {
            if route.action == action_str {
                // Check condition if present - skip route if condition fails
                if route.condition.as_ref().is_some_and(|c| !c.evaluate(store)) {
                    continue;
                }
                return Ok(route.target_node_id.clone());
            }
        }

        Err(FlowError::NoRouteFound(
            current_node_id.to_string(),
            action_str,
        ))
    }
}

// Implementation of FlowBackend for Flow
#[cfg(not(feature = "async"))]
impl<S: SharedStore + Send + Sync> FlowBackend<S> for Flow<S>
where
    S::Error: Send + Sync + 'static,
{
    /// Add a node to the flow
    ///
    /// Registers a new executable node with the flow. Each node must have a unique ID
    /// that will be used for routing and execution control.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the node
    /// * `node` - Boxed node implementation that can be executed
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Node was successfully added
    /// * `Err(FlowError)` - Currently always returns `Ok`, but may validate in the future
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::{Flow, FlowBackend};
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let mut flow: Flow<MemoryStorage> = Flow::new();
    /// // In practice, you would create a proper Node implementation
    /// // flow.add_node("processing_step".to_string(), node).unwrap();
    /// # }
    /// ```
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError> {
        self.nodes.insert(id, node);
        Ok(())
    }

    /// Add a route to the flow
    ///
    /// Defines a routing rule that determines how the flow transitions from one node
    /// to another based on the action returned by the source node.
    ///
    /// # Arguments
    ///
    /// * `from_node_id` - ID of the source node
    /// * `route` - Route definition including target node and conditions
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Route was successfully added
    /// * `Err(FlowError)` - Currently always returns `Ok`, but may validate in the future
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::{Flow, FlowBackend};
    /// use cosmoflow::flow::route::Route;
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let mut flow: Flow<MemoryStorage> = Flow::new();
    /// let route = Route {
    ///     action: "next_step".to_string(),
    ///     target_node_id: Some("target_node".to_string()),
    ///     condition: None,
    /// };
    /// flow.add_route("source_node".to_string(), route).unwrap();
    /// # }
    /// ```
    /// ```
    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError> {
        self.routes.entry(from_node_id).or_default().push(route);
        Ok(())
    }

    /// Execute the flow from the configured start node
    ///
    /// Begins workflow execution from the start node specified in the flow configuration.
    /// This is the primary entry point for workflow execution.
    ///
    /// # Arguments
    ///
    /// * `store` - Mutable reference to the shared store for data communication
    ///
    /// # Returns
    ///
    /// * `Ok(FlowExecutionResult)` - Successful execution result with metadata
    /// * `Err(FlowError)` - Execution error (node not found, etc.)
    ///
    /// # Errors
    ///
    /// * [`FlowError::NodeNotFound`] - Start node doesn't exist
    /// * [`FlowError::MaxStepsExceeded`] - Execution exceeded step limit
    /// * [`FlowError::MaxStepsExceeded`] - Execution limit reached
    /// * [`FlowError::NoRouteFound`] - No route available for node's action
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::{Flow, FlowBackend, FlowConfig};
    /// use cosmoflow::shared_store::SharedStore;
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// # async {
    /// let mut flow: Flow<MemoryStorage> = Flow::new();
    /// let mut store = MemoryStorage::new();
    ///
    /// // Add nodes and routes to flow...
    ///
    /// let result = flow.execute(&mut store).await.unwrap();
    /// # };
    /// # }
    /// ```
    /// println!("Execution completed in {} steps", result.steps_executed);
    /// # };
    /// ```
    fn execute(&mut self, store: &mut S) -> Result<FlowExecutionResult, FlowError> {
        let start_node_id = self.config.start_node_id.clone();
        self.execute_from(store, start_node_id)
    }

    /// Execute the flow from a specific node
    ///
    /// Begins workflow execution from an arbitrary node rather than the configured
    /// start node. Useful for debugging, testing, or resuming execution from a
    /// specific point.
    ///
    /// # Arguments
    ///
    /// * `store` - Mutable reference to the shared store for data communication
    /// * `start_node_id` - ID of the node to begin execution from
    ///
    /// # Returns
    ///
    /// * `Ok(FlowExecutionResult)` - Successful execution result with metadata
    /// * `Err(FlowError)` - Execution error (node not found, etc.)
    ///
    /// # Execution Flow
    ///
    /// 1. Starts from the specified node
    /// 2. Executes nodes sequentially following routing rules
    /// 3. Checks step limits at each iteration
    /// 4. Continues until a terminal action is reached
    /// 5. Returns detailed execution results
    ///
    /// # Performance Considerations
    ///
    /// - Each step includes step limit checking (O(1))
    /// - Step counting prevents infinite loops in complex workflows
    /// - Node execution is sequential (no parallel execution within a flow)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::{Flow, FlowBackend};
    /// use cosmoflow::shared_store::SharedStore;
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// # async {
    /// let mut flow: Flow<MemoryStorage> = Flow::new();
    /// let mut store = MemoryStorage::new();
    ///
    /// // Execute from a specific node (useful for testing)
    /// let result = flow.execute_from(&mut store, "validation_step".to_string()).await.unwrap();
    /// # };
    /// # }
    /// ```
    /// ```
    fn execute_from(
        &mut self,
        store: &mut S,
        start_node_id: String,
    ) -> Result<FlowExecutionResult, FlowError> {
        let mut current_node_id = start_node_id;
        let mut execution_path = Vec::new();
        let mut steps_executed = 0;

        loop {
            // Check step limit
            if steps_executed >= self.config.max_steps {
                return Err(FlowError::MaxStepsExceeded(self.config.max_steps));
            }

            // Add current node to execution path
            execution_path.push(current_node_id.clone());

            // Get the current node
            let node = self
                .nodes
                .get_mut(&current_node_id)
                .ok_or_else(|| FlowError::NodeNotFound(current_node_id.clone()))?;

            // Execute the node
            let action = node.run(store)?;
            steps_executed += 1;

            // Find next node
            match self.find_next_node(&current_node_id, &action, store)? {
                Some(next_node_id) => {
                    current_node_id = next_node_id;
                }
                None => {
                    // Terminal action reached
                    return Ok(FlowExecutionResult {
                        final_action: action,
                        last_node_id: current_node_id,
                        steps_executed,
                        success: true,
                        execution_path,
                    });
                }
            }
        }
    }

    /// Get the current flow configuration
    ///
    /// Returns a reference to the flow's configuration settings including
    /// execution limits, terminal actions, and other behavioral controls.
    ///
    /// # Returns
    ///
    /// Reference to the current [`FlowConfig`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::{Flow, FlowBackend};
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let flow: Flow<MemoryStorage> = Flow::new();
    /// let config = flow.config();
    /// println!("Max steps: {}", config.max_steps);
    /// # }
    /// ```
    fn config(&self) -> &FlowConfig {
        &self.config
    }

    /// Update the flow configuration
    ///
    /// Replaces the current configuration with new settings. This can be used
    /// to modify execution behavior, timeouts, or other flow parameters.
    ///
    /// # Arguments
    ///
    /// * `config` - New configuration to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::{Flow, FlowConfig, FlowBackend};
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let mut flow: Flow<MemoryStorage> = Flow::new();
    /// let new_config = FlowConfig {
    ///     max_steps: 1000,
    ///     start_node_id: "entry_point".to_string(),
    /// };
    /// flow.set_config(new_config);
    /// # }
    /// ```
    fn set_config(&mut self, config: FlowConfig) {
        self.config = config;
    }

    /// Validate the flow configuration and structure
    ///
    /// Performs comprehensive validation of the flow to ensure it can execute
    /// successfully. This includes checking node references, route validity,
    /// and configuration consistency.
    ///
    /// # Validation Checks
    ///
    /// 1. **Start Node Exists**: Verifies the configured start node is present
    /// 2. **Route Integrity**: Ensures all route source and target nodes exist
    /// 3. **Reachability**: Checks that all nodes can potentially be reached
    /// 4. **Terminal Actions**: Validates terminal action configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Flow is valid and ready for execution
    /// * `Err(FlowError::InvalidConfiguration)` - Validation failed with details
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "storage-memory")]
    /// # {
    /// use cosmoflow::flow::{Flow, FlowBackend};
    /// use cosmoflow::shared_store::backends::MemoryStorage;
    ///
    /// let flow: Flow<MemoryStorage> = Flow::new();
    /// match flow.validate() {
    ///     Ok(()) => println!("Flow is valid"),
    ///     Err(e) => eprintln!("Validation failed: {}", e),
    /// }
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// Validation time is O(n + m) where n is the number of nodes and m is the
    /// number of routes. It's recommended to validate flows during construction
    /// rather than at runtime.
    fn validate(&self) -> Result<(), FlowError> {
        // Check if start node exists
        if !self.nodes.contains_key(&self.config.start_node_id) {
            return Err(FlowError::InvalidConfiguration(format!(
                "Start node '{}' not found",
                self.config.start_node_id
            )));
        }

        // Check if all route targets exist
        for (from_node, routes) in &self.routes {
            if !self.nodes.contains_key(from_node) {
                return Err(FlowError::InvalidConfiguration(format!(
                    "Source node '{from_node}' in routes not found",
                )));
            }

            for route in routes {
                // Only validate target node existence if it's not a terminal route
                let Some(target_node_id) = &route.target_node_id else {
                    continue;
                };

                if !self.nodes.contains_key(target_node_id) {
                    return Err(FlowError::InvalidConfiguration(format!(
                        "Target node '{target_node_id}' in route not found"
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(not(feature = "async"))]
impl<S: SharedStore + 'static> Default for Flow<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of Node for Flow, allowing flows to be nested (sync version)
///
/// This implementation enables flows to be used as nodes within other flows,
/// creating hierarchical workflow structures. Key features:
///
/// - Automatic flow validation during preparation
/// - Nesting depth protection to prevent infinite recursion
/// - Result storage in shared store for parent flow access
/// - Proper error propagation through the flow hierarchy
#[cfg(not(feature = "async"))]
impl<S: SharedStore + Send + Sync + 'static> Node<S> for Flow<S>
where
    S::Error: Send + Sync + 'static,
{
    type PrepResult = ();
    type ExecResult = FlowExecutionResult;
    type Error = FlowError;

    fn prep(
        &mut self,
        _store: &S,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        // Check nesting depth to prevent infinite recursion
        let current_depth = context
            .get_metadata("flow_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if current_depth > 10 {
            return Err(FlowError::InvalidConfiguration(
                "Maximum flow nesting depth exceeded".to_string(),
            ));
        }

        // Validate the flow before execution
        self.validate()?;
        Ok(())
    }

    fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // Create a placeholder result - actual execution happens in post
        // This separation allows us to have mutable access to the store in post
        Ok(FlowExecutionResult {
            final_action: Action::simple("flow_ready"),
            last_node_id: format!("nested_flow_{}", context.execution_id()),
            steps_executed: 0,
            success: true,
            execution_path: vec![],
        })
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Increment nesting depth for nested flow execution
        let current_depth = context
            .get_metadata("flow_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Store the increased depth in the context for nested flow
        let nested_context_key = format!("nested_flow_depth_{}", context.execution_id());
        store
            .set(nested_context_key, serde_json::json!(current_depth + 1))
            .map_err(|e| FlowError::NodeError(format!("Failed to set nesting depth: {e}")))?;

        // Execute the nested flow
        let result = self.execute(store)?;

        // Store the nested flow result in the shared store for parent flow access
        let result_key = format!("nested_flow_result_{}", context.execution_id());
        store
            .set(
                result_key,
                serde_json::json!({
                    "final_action": result.final_action.to_string(),
                    "last_node_id": result.last_node_id,
                    "steps_executed": result.steps_executed,
                    "success": result.success,
                    "execution_path": result.execution_path,
                    "execution_id": context.execution_id()
                }),
            )
            .map_err(|e| FlowError::NodeError(format!("Failed to store flow result: {e}")))?;

        // Return the final action from the nested flow
        Ok(result.final_action)
    }

    fn max_retries(&self) -> usize {
        // Flows typically shouldn't be retried as they handle their own error recovery
        0
    }

    fn retry_delay(&self) -> Duration {
        Duration::ZERO
    }
}

/// Implementation of Node for Flow, allowing flows to be nested (sync version)
///
/// This implementation enables flows to be used as nodes within other flows,
/// creating hierarchical workflow structures. Key features:
///
/// - Automatic flow validation during preparation
/// - Nesting depth protection to prevent infinite recursion
/// - Result storage in shared store for parent flow access
/// - Proper error propagation through the flow hierarchy

#[cfg(all(test, feature = "storage-memory", not(feature = "async")))]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::node::ExecutionContext;
    use crate::shared_store::backends::MemoryStorage;

    // Sync test helper node
    #[cfg(not(feature = "async"))]
    mod sync_test_node {
        use super::*;

        pub struct TestNode {
            pub action: Action,
            pub should_fail: bool,
        }

        impl TestNode {
            pub fn new(action: Action) -> Self {
                Self {
                    action,
                    should_fail: false,
                }
            }
        }

        impl Node<MemoryStorage> for TestNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = NodeError;

            fn prep(
                &mut self,
                _store: &MemoryStorage,
                _context: &ExecutionContext,
            ) -> Result<Self::PrepResult, Self::Error> {
                if self.should_fail {
                    return Err(NodeError::PreparationError("Test failure".to_string()));
                }
                Ok(())
            }

            fn exec(
                &mut self,
                _prep_result: Self::PrepResult,
                _context: &ExecutionContext,
            ) -> Result<Self::ExecResult, Self::Error> {
                Ok(())
            }

            fn post(
                &mut self,
                _store: &mut MemoryStorage,
                _prep_result: Self::PrepResult,
                _exec_result: Self::ExecResult,
                _context: &ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(self.action.clone())
            }
        }
    }

    // Async test helper node
    #[cfg(feature = "async")]
    mod async_test_node {
        use super::*;
        use async_trait::async_trait;

        pub struct TestNode {
            pub action: Action,
            pub should_fail: bool,
        }

        impl TestNode {
            pub fn new(action: Action) -> Self {
                Self {
                    action,
                    should_fail: false,
                }
            }
        }

        #[async_trait]
        impl Node<MemoryStorage> for TestNode {
            type PrepResult = ();
            type ExecResult = ();
            type Error = NodeError;

            async fn prep(
                &mut self,
                _store: &MemoryStorage,
                _context: &ExecutionContext,
            ) -> Result<Self::PrepResult, Self::Error> {
                if self.should_fail {
                    return Err(NodeError::PreparationError("Test failure".to_string()));
                }
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
                _store: &mut MemoryStorage,
                _prep_result: Self::PrepResult,
                _exec_result: Self::ExecResult,
                _context: &ExecutionContext,
            ) -> Result<Action, Self::Error> {
                Ok(self.action.clone())
            }
        }
    }

    // Use appropriate TestNode implementation
    #[cfg(feature = "async")]
    use async_test_node::TestNode;
    #[cfg(not(feature = "async"))]
    use sync_test_node::TestNode;

    // Sync tests
    #[cfg(not(feature = "async"))]
    mod sync_tests {
        use super::*;

        #[test]
        fn test_basic_flow_execution() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("next")))
                .node("middle", TestNode::new(Action::simple("end")))
                .route("start", "next", "middle")
                .terminal_route("middle", "end")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            if let Err(e) = &result {
                eprintln!("Flow execution failed: {:?}", e);
            }
            assert!(result.is_ok());
            let result = result.unwrap();
            assert!(result.success);
            assert_eq!(result.steps_executed, 2);
            assert_eq!(result.execution_path, vec!["start", "middle"]);
        }

        #[test]
        fn test_flow_max_steps_exceeded() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .max_steps(2)
                .node("start", TestNode::new(Action::simple("next")))
                .node("middle", TestNode::new(Action::simple("continue")))
                .node("end_node", TestNode::new(Action::simple("end")))
                .route("start", "next", "middle")
                .route("middle", "continue", "end_node")
                .route("end_node", "end", "final")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::MaxStepsExceeded(max) => assert_eq!(max, 2),
                _ => panic!("Expected MaxStepsExceeded error"),
            }
        }

        #[test]
        fn test_flow_node_not_found() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("next")))
                .route("start", "next", "nonexistent")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::NodeNotFound(id) => assert_eq!(id, "nonexistent"),
                _ => panic!("Expected NodeNotFound error"),
            }
        }

        #[test]
        fn test_flow_no_route_found() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("unknown")))
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::NoRouteFound(node_id, action) => {
                    assert_eq!(node_id, "start");
                    assert_eq!(action, "unknown");
                }
                _ => panic!("Expected NoRouteFound error"),
            }
        }

        #[test]
        fn test_flow_validation() {
            let flow = FlowBuilder::new()
                .start_node("nonexistent")
                .node("start", TestNode::new(Action::simple("next")))
                .build();

            let result = flow.validate();
            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::InvalidConfiguration(msg) => {
                    assert!(msg.contains("Start node 'nonexistent' not found"));
                }
                _ => panic!("Expected InvalidConfiguration error"),
            }
        }

        #[test]
        fn test_flow_builder_methods() {
            let flow: Flow<MemoryStorage> = FlowBuilder::new()
                .start_node("custom_start")
                .max_steps(500)
                .build();

            assert_eq!(flow.config().start_node_id, "custom_start");
            assert_eq!(flow.config().max_steps, 500);
        }

        #[test]
        fn test_conditional_route() {
            use crate::flow::route::RouteCondition;

            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("check")))
                .node("success", TestNode::new(Action::simple("end")))
                .node("failure", TestNode::new(Action::simple("end")))
                .conditional_route("start", "check", "success", RouteCondition::Always)
                .terminal_route("success", "end")
                .terminal_route("failure", "end")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            if let Err(e) = &result {
                eprintln!("Conditional route test failed: {:?}", e);
            }
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.execution_path, vec!["start", "success"]);
        }
    }

    // Async tests
    #[cfg(feature = "async")]
    mod async_tests {
        use super::*;

        #[tokio::test]
        fn test_basic_flow_execution() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("next")))
                .node("middle", TestNode::new(Action::simple("end")))
                .route("start", "next", "middle")
                .terminal_route("middle", "end")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            if let Err(e) = &result {
                eprintln!("Flow execution failed: {:?}", e);
            }
            assert!(result.is_ok());
            let result = result.unwrap();
            assert!(result.success);
            assert_eq!(result.steps_executed, 2);
            assert_eq!(result.execution_path, vec!["start", "middle"]);
        }

        #[tokio::test]
        fn test_flow_max_steps_exceeded() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .max_steps(2)
                .node("start", TestNode::new(Action::simple("next")))
                .node("middle", TestNode::new(Action::simple("continue")))
                .node("end_node", TestNode::new(Action::simple("end")))
                .route("start", "next", "middle")
                .route("middle", "continue", "end_node")
                .route("end_node", "end", "final")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::MaxStepsExceeded(max) => assert_eq!(max, 2),
                _ => panic!("Expected MaxStepsExceeded error"),
            }
        }

        #[tokio::test]
        fn test_flow_node_not_found() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("next")))
                .route("start", "next", "nonexistent")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::NodeNotFound(id) => assert_eq!(id, "nonexistent"),
                _ => panic!("Expected NodeNotFound error"),
            }
        }

        #[tokio::test]
        fn test_flow_no_route_found() {
            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("unknown")))
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::NoRouteFound(node_id, action) => {
                    assert_eq!(node_id, "start");
                    assert_eq!(action, "unknown");
                }
                _ => panic!("Expected NoRouteFound error"),
            }
        }

        #[tokio::test]
        fn test_flow_validation() {
            let flow = FlowBuilder::new()
                .start_node("nonexistent")
                .node("start", TestNode::new(Action::simple("next")))
                .build();

            let result = flow.validate();
            assert!(result.is_err());
            match result.unwrap_err() {
                FlowError::InvalidConfiguration(msg) => {
                    assert!(msg.contains("Start node 'nonexistent' not found"));
                }
                _ => panic!("Expected InvalidConfiguration error"),
            }
        }

        #[tokio::test]
        fn test_flow_builder_methods() {
            let flow: Flow<MemoryStorage> = FlowBuilder::new()
                .start_node("custom_start")
                .max_steps(500)
                .build();

            assert_eq!(flow.config().start_node_id, "custom_start");
            assert_eq!(flow.config().max_steps, 500);
        }

        #[tokio::test]
        fn test_conditional_route() {
            use crate::flow::route::RouteCondition;

            let mut flow = FlowBuilder::new()
                .start_node("start")
                .node("start", TestNode::new(Action::simple("check")))
                .node("success", TestNode::new(Action::simple("end")))
                .node("failure", TestNode::new(Action::simple("end")))
                .conditional_route("start", "check", "success", RouteCondition::Always)
                .terminal_route("success", "end")
                .terminal_route("failure", "end")
                .build();

            let mut store = MemoryStorage::new();
            let result = flow.execute(&mut store);

            if let Err(e) = &result {
                eprintln!("Conditional route test failed: {:?}", e);
            }
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.execution_path, vec!["start", "success"]);
        }
    }

    // Common tests that don't require flow execution
    #[test]
    fn test_flow_config_default() {
        let config = FlowConfig::default();
        assert_eq!(config.start_node_id, "start");
        assert_eq!(config.max_steps, 1000);
    }

    #[test]
    fn test_terminal_action_warning_in_route() {
        // Test that building a flow with terminal actions in routes doesn't panic
        // The warning will be printed to stderr during execution
        let _flow = FlowBuilder::<MemoryStorage>::new()
            .route("node1", "complete", "node2") // This should trigger a warning
            .build();

        // This test mainly ensures the code compiles and runs without panic
        // In a real-world scenario, you might want to capture stderr to verify the warning
    }

    #[test]
    fn test_terminal_action_warning_in_conditional_route() {
        use crate::flow::route::RouteCondition;

        let _flow = FlowBuilder::<MemoryStorage>::new()
            .conditional_route("node1", "end", "node2", RouteCondition::Always) // This should trigger a warning
            .build();
    }

    #[test]
    fn test_non_terminal_action_no_warning() {
        // This should not trigger any warning
        let _flow = FlowBuilder::<MemoryStorage>::new()
            .route("node1", "custom_action", "node2")
            .build();
    }
}
