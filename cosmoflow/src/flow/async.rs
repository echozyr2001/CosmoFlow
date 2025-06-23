//! Async-specific implementations for CosmoFlow
//!
//! This module contains all the async-specific trait implementations and functionality
//! for the CosmoFlow workflow engine. These are only available when the "async" feature
//! is enabled.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::action::Action;
use crate::node::{ExecutionContext, NodeError, r#async::Node};
use crate::shared_store::SharedStore;

use super::errors::FlowError;
use super::route::{Route, RouteCondition};
use super::{FlowConfig, FlowExecutionResult};

/// Node runner trait for workflow execution (async version)
///
/// This trait provides a unified interface for executing nodes with different
/// associated types in the same flow, allowing the flow system to work with
/// heterogeneous node collections while maintaining type safety.
#[async_trait]
pub trait NodeRunner<S: SharedStore>: Send + Sync {
    /// Execute the node and return the resulting action
    async fn run(&mut self, store: &mut S) -> Result<Action, NodeError>;

    /// Get the node's name for debugging and logging
    fn name(&self) -> &str;
}

/// Implementation of NodeRunner for any Node (async version)
#[async_trait]
impl<T, S> NodeRunner<S> for T
where
    T: Node<S> + Send + Sync,
    S: SharedStore + Send + Sync,
{
    async fn run(&mut self, store: &mut S) -> Result<Action, NodeError> {
        Node::run(self, store).await
    }

    fn name(&self) -> &str {
        Node::name(self)
    }
}

/// Trait for implementing flow execution logic (async version)
#[async_trait]
pub trait FlowBackend<S: SharedStore> {
    /// Add a node to the flow
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError>;

    /// Add a route between nodes
    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError>;

    /// Execute the flow starting from the configured start node
    async fn execute(&mut self, store: &mut S) -> Result<FlowExecutionResult, FlowError>;

    /// Execute the flow starting from a specific node
    async fn execute_from(
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

/// The main flow structure (async version)
pub struct Flow<S: SharedStore> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

impl<S: SharedStore + 'static> Flow<S> {
    /// Create a new empty flow
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config: FlowConfig::default(),
        }
    }

    /// Create a new flow with custom configuration
    pub fn with_config(config: FlowConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config,
        }
    }

    /// Add a node to the flow
    pub fn add_node<N>(&mut self, id: impl Into<String>, node: N) -> Result<(), FlowError>
    where
        N: Node<S> + Send + Sync + 'static,
    {
        let id = id.into();
        if self.nodes.contains_key(&id) {
            return Err(FlowError::InvalidConfiguration(format!(
                "Duplicate node: {id}"
            )));
        }
        self.nodes.insert(id, Box::new(node));
        Ok(())
    }

    /// Add a route from one node to another
    pub fn add_route(
        &mut self,
        from_node_id: impl Into<String>,
        action: impl Into<String>,
        to_node_id: impl Into<String>,
    ) -> Result<(), FlowError> {
        let from_node_id = from_node_id.into();
        let route = Route {
            action: action.into(),
            target_node_id: Some(to_node_id.into()),
            condition: Some(RouteCondition::Always),
        };

        // Check for terminal action warning
        if route.target_node_id.is_none() {
            eprintln!(
                "Warning: Adding route with terminal action '{}' from node '{}'. \
                 Terminal actions typically end workflows and may not route to other nodes.",
                route.action, from_node_id
            );
        }

        self.routes.entry(from_node_id).or_default().push(route);
        Ok(())
    }

    // Internal helper methods
    async fn internal_execute(
        &mut self,
        store: &mut S,
        start_node_id: String,
    ) -> Result<FlowExecutionResult, FlowError> {
        let _start_time = Instant::now();
        let mut current_node_id = start_node_id;
        let mut execution_path = Vec::new();
        let mut steps_executed = 0;

        loop {
            // Check max steps
            if steps_executed >= self.config.max_steps {
                return Err(FlowError::MaxStepsExceeded(self.config.max_steps));
            }

            // Get the current node
            let node = self
                .nodes
                .get_mut(&current_node_id)
                .ok_or_else(|| FlowError::NodeNotFound(current_node_id.clone()))?;

            // Execute the node
            let _context = ExecutionContext::new(execution_path.len(), Duration::from_secs(30));
            let action = node.run(store).await?;

            execution_path.push(current_node_id.clone());
            steps_executed += 1;

            // Check for terminal actions (routes with no target)
            if (self.find_next_node(&current_node_id, &action, store)?).is_none() {
                return Ok(FlowExecutionResult {
                    final_action: action,
                    last_node_id: current_node_id,
                    steps_executed,
                    success: true,
                    execution_path,
                });
            }

            // Find the next node
            let routes = self.routes.get(&current_node_id).ok_or_else(|| {
                FlowError::NoRouteFound(current_node_id.clone(), action.name().to_string())
            })?;

            let next_node_id = routes
                .iter()
                .find(|route| {
                    route.action == action.name()
                        && route.condition.as_ref().is_none_or(|c| c.evaluate(store))
                })
                .and_then(|route| route.target_node_id.as_ref())
                .ok_or_else(|| {
                    FlowError::NoRouteFound(current_node_id, action.name().to_string())
                })?;

            current_node_id = next_node_id.clone();
        }
    }

    fn internal_validate(&self) -> Result<(), FlowError> {
        // Check if start node exists
        if !self.nodes.contains_key(&self.config.start_node_id) {
            return Err(FlowError::InvalidConfiguration(format!(
                "Start node '{}' not found in flow",
                self.config.start_node_id
            )));
        }

        // Additional validation can be added here
        Ok(())
    }

    /// Find the next node to execute based on the current node and action
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

#[async_trait]
impl<S: SharedStore + Send + Sync + 'static> FlowBackend<S> for Flow<S> {
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError> {
        if self.nodes.contains_key(&id) {
            return Err(FlowError::InvalidConfiguration(format!(
                "Duplicate node: {id}"
            )));
        }
        self.nodes.insert(id, node);
        Ok(())
    }

    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError> {
        self.routes.entry(from_node_id).or_default().push(route);
        Ok(())
    }

    async fn execute(&mut self, store: &mut S) -> Result<FlowExecutionResult, FlowError> {
        self.validate()?;
        self.internal_execute(store, self.config.start_node_id.clone())
            .await
    }

    async fn execute_from(
        &mut self,
        store: &mut S,
        start_node_id: String,
    ) -> Result<FlowExecutionResult, FlowError> {
        self.internal_execute(store, start_node_id).await
    }

    fn config(&self) -> &FlowConfig {
        &self.config
    }

    fn set_config(&mut self, config: FlowConfig) {
        self.config = config;
    }

    fn validate(&self) -> Result<(), FlowError> {
        self.internal_validate()
    }
}

impl<S: SharedStore + 'static> Default for Flow<S> {
    fn default() -> Self {
        Self::new()
    }
}

// Flow as Node implementation (async version)
#[async_trait]
impl<S: SharedStore + Send + Sync + 'static> Node<S> for Flow<S> {
    type PrepResult = ();
    type ExecResult = FlowExecutionResult;
    type Error = NodeError;

    async fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), NodeError> {
        self.validate()
            .map_err(|e| NodeError::PreparationError(e.to_string()))?;
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: (),
        _context: &ExecutionContext,
    ) -> Result<FlowExecutionResult, NodeError> {
        // For now, we can't execute a flow as a node without a store
        // This would need to be implemented differently
        Err(NodeError::ExecutionError(
            "Flow as Node execution not yet implemented".to_string(),
        ))
    }

    async fn post(
        &mut self,
        _store: &mut S,
        _prep_result: (),
        exec_result: FlowExecutionResult,
        _context: &ExecutionContext,
    ) -> Result<Action, NodeError> {
        // Return the final action from the flow execution
        Ok(exec_result.final_action)
    }

    fn name(&self) -> &'static str {
        "Flow"
    }
}

/// Builder for creating async flows easily
pub struct FlowBuilder<S: SharedStore> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

impl<S: SharedStore + Send + Sync + 'static> Default for FlowBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: SharedStore + Send + Sync + 'static> FlowBuilder<S> {
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
    pub fn self_route(self, node_id: impl Into<String>, action: impl Into<String>) -> Self {
        let node_id_str = node_id.into();
        self.route(node_id_str.clone(), action, node_id_str)
    }
}
