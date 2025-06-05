use std::collections::HashMap;
use std::time::Duration;

use action::Action;
use async_trait::async_trait;
use node::{ExecutionContext, Node, NodeBackend, NodeError};
use shared_store::{SharedStore, StorageBackend};

use crate::{
    errors::FlowError,
    route::{Route, RouteCondition},
};

pub mod errors;
pub mod route;

/// Execution result from a flow run
#[derive(Debug, Clone)]
pub struct FlowExecutionResult {
    /// The final action that terminated the flow
    pub final_action: Action,
    /// The ID of the last executed node
    pub last_node_id: String,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Whether the flow completed successfully
    pub success: bool,
    /// Execution path (node IDs in order)
    pub execution_path: Vec<String>,
}

/// Configuration for flow execution
#[derive(Debug, Clone)]
pub struct FlowConfig {
    /// Maximum number of execution steps before terminating
    pub max_steps: usize,
    /// Whether to detect and prevent cycles
    pub detect_cycles: bool,
    /// Starting node ID
    pub start_node_id: String,
    /// Actions that terminate the flow
    pub terminal_actions: Vec<String>,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            max_steps: 1000,
            detect_cycles: true,
            start_node_id: "start".to_string(),
            terminal_actions: vec![
                "end".to_string(),
                "complete".to_string(),
                "finish".to_string(),
            ],
        }
    }
}

/// Type-erased node runner for dynamic dispatch
#[async_trait]
pub trait NodeRunner<S: StorageBackend>: Send + Sync {
    async fn run(&mut self, store: &mut SharedStore<S>) -> Result<Action, NodeError>;
}

/// Implementation of NodeRunner for any Node
#[async_trait]
impl<B, S> NodeRunner<S> for Node<B, S>
where
    B: NodeBackend<S> + Send + Sync,
    S: StorageBackend + Send + Sync,
    B::Error: Send + Sync + 'static,
{
    async fn run(&mut self, store: &mut SharedStore<S>) -> Result<Action, NodeError> {
        // Call the node's run method directly instead of recursive call
        Node::run(self, store)
            .await
            .map_err(|err| NodeError::ExecutionError(err.to_string()))
    }
}

/// Trait for implementing flow execution logic
#[async_trait]
pub trait FlowBackend<S: StorageBackend> {
    /// Add a node to the flow
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError>;

    /// Add a route between nodes
    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError>;

    /// Execute the flow starting from the configured start node
    async fn execute(
        &mut self,
        store: &mut SharedStore<S>,
    ) -> Result<FlowExecutionResult, FlowError>;

    /// Execute the flow starting from a specific node
    async fn execute_from(
        &mut self,
        store: &mut SharedStore<S>,
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
pub struct FlowBuilder<S: StorageBackend> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

impl<S: StorageBackend + 'static> Default for FlowBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: StorageBackend + 'static> FlowBuilder<S> {
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

    /// Add a terminal action
    pub fn terminal_action(mut self, action: impl Into<String>) -> Self {
        self.config.terminal_actions.push(action.into());
        self
    }

    /// Add a node to the flow
    pub fn node<B>(mut self, id: impl Into<String>, node: Node<B, S>) -> Self
    where
        B: NodeBackend<S> + Send + Sync + 'static,
        B::Error: Send + Sync + 'static,
    {
        self.nodes.insert(id.into(), Box::new(node));
        self
    }

    /// Add a simple route (action -> target node)
    pub fn route(
        mut self,
        from: impl Into<String>,
        action: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        let from_id = from.into();
        let route = Route {
            action: action.into(),
            target_node_id: to.into(),
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
        let route = Route {
            action: action.into(),
            target_node_id: to.into(),
            condition: Some(condition),
        };

        self.routes.entry(from_id).or_default().push(route);
        self
    }

    /// Build the flow
    pub fn build(self) -> Flow<S> {
        Flow {
            nodes: self.nodes,
            routes: self.routes,
            config: self.config,
        }
    }
}

/// Basic implementation of the FlowBackend trait
pub struct Flow<S: StorageBackend> {
    nodes: HashMap<String, Box<dyn NodeRunner<S>>>,
    routes: HashMap<String, Vec<Route>>,
    config: FlowConfig,
}

impl<S: StorageBackend> Flow<S> {
    /// Create a new basic flow
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config: FlowConfig::default(),
        }
    }

    /// Create a new basic flow with custom configuration
    pub fn with_config(config: FlowConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            routes: HashMap::new(),
            config,
        }
    }

    /// Find the next node ID based on the current action
    fn find_next_node(
        &self,
        current_node_id: &str,
        action: &Action,
        store: &SharedStore<S>,
    ) -> Result<Option<String>, FlowError> {
        let action_str = action.to_string();

        // Check if this is a terminal action
        if self.config.terminal_actions.contains(&action_str) {
            return Ok(None);
        }

        // Get routes for the current node
        let routes = self.routes.get(current_node_id).ok_or_else(|| {
            FlowError::NoRouteFound(current_node_id.to_string(), action_str.clone())
        })?;

        // Find matching route
        for route in routes {
            if route.action == action_str {
                // Check condition if present
                if let Some(condition) = &route.condition {
                    if !condition.evaluate(store) {
                        continue;
                    }
                }
                return Ok(Some(route.target_node_id.clone()));
            }
        }

        Err(FlowError::NoRouteFound(
            current_node_id.to_string(),
            action_str,
        ))
    }

    /// Check for cycles in the execution path
    fn check_cycle(&self, path: &[String], next_node_id: &str) -> Result<(), FlowError> {
        if !self.config.detect_cycles {
            return Ok(());
        }

        // Use more efficient search for cycle detection
        if let Some(cycle_start_index) = path.iter().position(|id| id == next_node_id) {
            let mut cycle_path = path[cycle_start_index..].to_vec();
            cycle_path.push(next_node_id.to_string());
            return Err(FlowError::CycleDetected(cycle_path));
        }

        Ok(())
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> FlowBackend<S> for Flow<S>
where
    S::Error: Send + Sync + 'static,
{
    fn add_node(&mut self, id: String, node: Box<dyn NodeRunner<S>>) -> Result<(), FlowError> {
        self.nodes.insert(id, node);
        Ok(())
    }

    fn add_route(&mut self, from_node_id: String, route: Route) -> Result<(), FlowError> {
        self.routes.entry(from_node_id).or_default().push(route);
        Ok(())
    }

    async fn execute(
        &mut self,
        store: &mut SharedStore<S>,
    ) -> Result<FlowExecutionResult, FlowError> {
        let start_node_id = self.config.start_node_id.clone();
        self.execute_from(store, start_node_id).await
    }

    async fn execute_from(
        &mut self,
        store: &mut SharedStore<S>,
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

            // Check for cycles
            self.check_cycle(&execution_path, &current_node_id)?;

            // Add current node to execution path
            execution_path.push(current_node_id.clone());

            // Get the current node
            let node = self
                .nodes
                .get_mut(&current_node_id)
                .ok_or_else(|| FlowError::NodeNotFound(current_node_id.clone()))?;

            // Execute the node
            let action = node.run(store).await.map_err(FlowError::from)?;
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

    fn config(&self) -> &FlowConfig {
        &self.config
    }

    fn set_config(&mut self, config: FlowConfig) {
        self.config = config;
    }

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
                if !self.nodes.contains_key(&route.target_node_id) {
                    return Err(FlowError::InvalidConfiguration(format!(
                        "Target node '{}' in route not found",
                        route.target_node_id
                    )));
                }
            }
        }

        Ok(())
    }
}

impl<S: StorageBackend + 'static> Default for Flow<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of NodeBackend for Flow, allowing flows to be nested
///
/// This implementation enables flows to be used as nodes within other flows,
/// creating hierarchical workflow structures. Key features:
///
/// - Automatic flow validation during preparation
/// - Nesting depth protection to prevent infinite recursion
/// - Result storage in shared store for parent flow access
/// - Proper error propagation through the flow hierarchy
#[async_trait]
impl<S: StorageBackend + Send + Sync + 'static> NodeBackend<S> for Flow<S>
where
    S::Error: Send + Sync + 'static,
{
    type PrepResult = ();
    type ExecResult = FlowExecutionResult;
    type Error = FlowError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
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

    async fn exec(
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

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
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
        let result = self.execute(store).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use action::Action;
    use async_trait::async_trait;
    use node::{ExecutionContext, NodeBackend};
    use shared_store::{MemoryStorage, SharedStore};

    // Test helper node
    struct TestNode {
        action: Action,
        should_fail: bool,
    }

    impl TestNode {
        fn new(action: Action) -> Self {
            Self {
                action,
                should_fail: false,
            }
        }
    }

    #[async_trait]
    impl NodeBackend<MemoryStorage> for TestNode {
        type PrepResult = ();
        type ExecResult = ();
        type Error = NodeError;

        async fn prep(
            &mut self,
            _store: &SharedStore<MemoryStorage>,
            _context: &ExecutionContext,
        ) -> Result<Self::PrepResult, Self::Error> {
            if self.should_fail {
                return Err(NodeError::PrepError("Test failure".to_string()));
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
            _store: &mut SharedStore<MemoryStorage>,
            _prep_result: Self::PrepResult,
            _exec_result: Self::ExecResult,
            _context: &ExecutionContext,
        ) -> Result<Action, Self::Error> {
            Ok(self.action.clone())
        }
    }

    #[tokio::test]
    async fn test_basic_flow_execution() {
        let mut flow = FlowBuilder::new()
            .start_node("start")
            .node("start", Node::new(TestNode::new(Action::simple("next"))))
            .node("middle", Node::new(TestNode::new(Action::simple("end"))))
            .route("start", "next", "middle")
            .route("middle", "end", "end")
            .build();

        let mut store = SharedStore::with_storage(MemoryStorage::new());
        let result = flow.execute(&mut store).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.steps_executed, 2);
        assert_eq!(result.execution_path, vec!["start", "middle"]);
    }

    #[tokio::test]
    async fn test_flow_cycle_detection() {
        let mut flow = FlowBuilder::new()
            .start_node("start")
            .node("start", Node::new(TestNode::new(Action::simple("next"))))
            .node("middle", Node::new(TestNode::new(Action::simple("back"))))
            .route("start", "next", "middle")
            .route("middle", "back", "start") // Creates a cycle
            .build();

        let mut store = SharedStore::with_storage(MemoryStorage::new());
        let result = flow.execute(&mut store).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            FlowError::CycleDetected(path) => {
                assert!(path.contains(&"start".to_string()));
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[tokio::test]
    async fn test_flow_max_steps_exceeded() {
        let mut flow = FlowBuilder::new()
            .start_node("start")
            .max_steps(2)
            .node("start", Node::new(TestNode::new(Action::simple("next"))))
            .node(
                "middle",
                Node::new(TestNode::new(Action::simple("continue"))),
            )
            .node("end_node", Node::new(TestNode::new(Action::simple("end"))))
            .route("start", "next", "middle")
            .route("middle", "continue", "end_node")
            .route("end_node", "end", "final")
            .build();

        let mut store = SharedStore::with_storage(MemoryStorage::new());
        let result = flow.execute(&mut store).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            FlowError::MaxStepsExceeded(max) => assert_eq!(max, 2),
            _ => panic!("Expected MaxStepsExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_flow_node_not_found() {
        let mut flow = FlowBuilder::new()
            .start_node("start")
            .node("start", Node::new(TestNode::new(Action::simple("next"))))
            .route("start", "next", "nonexistent")
            .build();

        let mut store = SharedStore::with_storage(MemoryStorage::new());
        let result = flow.execute(&mut store).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            FlowError::NodeNotFound(id) => assert_eq!(id, "nonexistent"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_flow_no_route_found() {
        let mut flow = FlowBuilder::new()
            .start_node("start")
            .node("start", Node::new(TestNode::new(Action::simple("unknown"))))
            .build();

        let mut store = SharedStore::with_storage(MemoryStorage::new());
        let result = flow.execute(&mut store).await;

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
    async fn test_flow_validation() {
        let flow = FlowBuilder::new()
            .start_node("nonexistent")
            .node("start", Node::new(TestNode::new(Action::simple("next"))))
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
    async fn test_flow_builder_methods() {
        let flow: Flow<MemoryStorage> = FlowBuilder::new()
            .start_node("custom_start")
            .max_steps(500)
            .terminal_action("custom_end")
            .build();

        assert_eq!(flow.config().start_node_id, "custom_start");
        assert_eq!(flow.config().max_steps, 500);
        assert!(
            flow.config()
                .terminal_actions
                .contains(&"custom_end".to_string())
        );
    }

    #[test]
    fn test_flow_config_default() {
        let config = FlowConfig::default();
        assert_eq!(config.start_node_id, "start");
        assert_eq!(config.max_steps, 1000);
        assert!(config.detect_cycles);
        assert!(config.terminal_actions.contains(&"end".to_string()));
    }

    #[tokio::test]
    async fn test_conditional_route() {
        use crate::route::RouteCondition;

        let mut flow = FlowBuilder::new()
            .start_node("start")
            .node("start", Node::new(TestNode::new(Action::simple("check"))))
            .node("success", Node::new(TestNode::new(Action::simple("end"))))
            .node("failure", Node::new(TestNode::new(Action::simple("end"))))
            .conditional_route("start", "check", "success", RouteCondition::Always)
            .build();

        let mut store = SharedStore::with_storage(MemoryStorage::new());
        let result = flow.execute(&mut store).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.execution_path, vec!["start", "success"]);
    }
}
