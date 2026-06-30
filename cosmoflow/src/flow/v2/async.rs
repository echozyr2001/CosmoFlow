use super::{FlowAnalysis, FlowError, FlowExecution, Route};
use crate::action::v2::{Action, ActionName};
use crate::node::v2::{DynNode, Node, NodeId};
use crate::shared_store::SharedStore;
use std::collections::HashMap;
use std::fmt;

/// Builder for a v2 asynchronous flow.
pub struct FlowBuilder<S: SharedStore> {
    nodes: HashMap<NodeId, Box<dyn DynNode<S>>>,
    node_order: Vec<NodeId>,
    routes: Vec<Route>,
    start: Option<NodeId>,
    duplicate_nodes: Vec<NodeId>,
}

impl<S: SharedStore> Default for FlowBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: SharedStore> FlowBuilder<S> {
    /// Create an empty flow builder.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            node_order: Vec::new(),
            routes: Vec::new(),
            start: None,
            duplicate_nodes: Vec::new(),
        }
    }

    /// Add a node to the flow.
    pub fn node<N>(mut self, id: impl Into<NodeId>, node: N) -> Self
    where
        N: Node<S> + Send + Sync + 'static,
    {
        let id = id.into();
        if self.nodes.contains_key(&id) {
            self.duplicate_nodes.push(id);
            return self;
        }

        if self.start.is_none() {
            self.start = Some(id.clone());
        }
        self.node_order.push(id.clone());
        self.nodes.insert(id, Box::new(node));
        self
    }

    /// Set the start node.
    pub fn start(mut self, id: impl Into<NodeId>) -> Self {
        self.start = Some(id.into());
        self
    }

    /// Add a route from one node to another for an action name.
    pub fn route(
        mut self,
        from: impl Into<NodeId>,
        action: impl Into<ActionName>,
        to: impl Into<NodeId>,
    ) -> Self {
        self.routes.push(Route::new(from, action, to));
        self
    }

    /// Validate and build the flow.
    pub fn build(self) -> Result<Flow<S>, FlowError> {
        let analysis = super::analysis::validate_graph(
            &self.node_order,
            &self.routes,
            self.start.as_ref(),
            &self.duplicate_nodes,
        )?;

        Ok(Flow {
            nodes: self.nodes,
            node_order: self.node_order,
            routes: self.routes,
            start: self
                .start
                .expect("validated non-empty flow has a start node"),
            analysis,
        })
    }
}

/// A v2 asynchronous flow.
pub struct Flow<S: SharedStore> {
    nodes: HashMap<NodeId, Box<dyn DynNode<S>>>,
    node_order: Vec<NodeId>,
    routes: Vec<Route>,
    start: NodeId,
    analysis: FlowAnalysis,
}

impl<S: SharedStore> fmt::Debug for Flow<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flow")
            .field("node_order", &self.node_order)
            .field("routes", &self.routes)
            .field("start", &self.start)
            .field("analysis", &self.analysis)
            .finish()
    }
}

impl<S: SharedStore> Flow<S> {
    /// Return static graph analysis computed at build time.
    pub fn analysis(&self) -> &FlowAnalysis {
        &self.analysis
    }

    /// Return the configured start node.
    pub fn start(&self) -> &NodeId {
        &self.start
    }

    /// Return nodes in insertion order.
    pub fn node_order(&self) -> &[NodeId] {
        &self.node_order
    }

    /// Return routes in insertion order.
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Run the flow and return the final action.
    pub async fn run(&mut self, state: &mut S) -> Result<Action, FlowError> {
        Ok(self.run_recorded(state).await?.final_action)
    }

    /// Run the flow and return an execution summary.
    pub async fn run_recorded(&mut self, state: &mut S) -> Result<FlowExecution, FlowError> {
        let mut current_node_id = self.start.clone();
        let mut path = Vec::new();

        loop {
            let node = self
                .nodes
                .get_mut(&current_node_id)
                .ok_or_else(|| FlowError::NodeNotFound(current_node_id.clone()))?;
            let action = node
                .run_node(state, &current_node_id)
                .await
                .map_err(FlowError::from)?;
            path.push(current_node_id.clone());

            if let Some(next_node_id) = self.next_node(&current_node_id, &action) {
                current_node_id = next_node_id;
                continue;
            }

            return Ok(FlowExecution {
                final_action: action,
                last_node_id: current_node_id,
                steps: path.len(),
                path,
            });
        }
    }

    fn next_node(&self, current_node_id: &NodeId, action: &Action) -> Option<NodeId> {
        self.routes
            .iter()
            .find(|route| {
                route.from == *current_node_id && route.action.as_str() == action.as_str()
            })
            .map(|route| route.to.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SharedStore;
    use crate::node::v2::NodeContext;
    use crate::shared_store::backends::MemoryStorage;
    use async_trait::async_trait;
    use serde_json::json;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct TestError(&'static str);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl Error for TestError {}

    struct StaticNode {
        action: Action,
        fail: bool,
    }

    impl StaticNode {
        fn new(action: impl Into<Action>) -> Self {
            Self {
                action: action.into(),
                fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                action: Action::new("unused"),
                fail: true,
            }
        }
    }

    #[async_trait]
    impl Node<MemoryStorage> for StaticNode {
        type Prep = ();
        type Output = ();
        type Error = TestError;

        async fn prep(
            &mut self,
            _state: &MemoryStorage,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            if self.fail {
                return Err(TestError("node failed"));
            }
            Ok(())
        }

        async fn exec(
            &mut self,
            _prep: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(())
        }

        async fn post(
            &mut self,
            state: &mut MemoryStorage,
            _prep: Self::Prep,
            _output: Self::Output,
            context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state
                .set(format!("visited:{}", context.node_id.as_str()), true)
                .map_err(|_| TestError("storage failed"))?;
            Ok(self.action.clone())
        }
    }

    #[tokio::test]
    async fn single_node_flow_uses_default_start_and_naturally_terminates() {
        let mut flow = FlowBuilder::new()
            .node("start", StaticNode::new("done"))
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).await.unwrap();

        assert_eq!(flow.start().as_str(), "start");
        assert_eq!(execution.final_action, Action::new("done"));
        assert_eq!(execution.last_node_id.as_str(), "start");
        assert_eq!(execution.steps, 1);
        assert_eq!(execution.path, vec![NodeId::new("start")]);
    }

    #[tokio::test]
    async fn run_returns_final_action() {
        let mut flow = FlowBuilder::new()
            .node("start", StaticNode::new("done"))
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let action = flow.run(&mut state).await.unwrap();

        assert_eq!(action, Action::new("done"));
    }

    #[tokio::test]
    async fn explicit_start_overrides_default_start() {
        let mut flow = FlowBuilder::new()
            .node("first", StaticNode::new("done"))
            .node("second", StaticNode::new("next"))
            .start("second")
            .route("second", "next", "first")
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).await.unwrap();

        assert_eq!(flow.start().as_str(), "second");
        assert_eq!(
            execution.path,
            vec![NodeId::new("second"), NodeId::new("first")]
        );
        assert_eq!(execution.final_action, Action::new("done"));
    }

    #[tokio::test]
    async fn action_params_do_not_affect_routing() {
        let mut flow = FlowBuilder::new()
            .node(
                "first",
                StaticNode::new(Action::with_param("next", "payload", json!(1))),
            )
            .node("second", StaticNode::new("done"))
            .route("first", "next", "second")
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).await.unwrap();

        assert_eq!(
            execution.path,
            vec![NodeId::new("first"), NodeId::new("second")]
        );
    }

    #[tokio::test]
    async fn no_matching_route_naturally_terminates() {
        let mut flow = FlowBuilder::new()
            .node("first", StaticNode::new("done"))
            .node("second", StaticNode::new("unused"))
            .route("first", "other", "second")
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).await.unwrap();

        assert_eq!(execution.path, vec![NodeId::new("first")]);
        assert_eq!(execution.final_action, Action::new("done"));
    }

    #[test]
    fn build_fails_for_invalid_graphs() {
        assert_eq!(
            FlowBuilder::<MemoryStorage>::new().build().unwrap_err(),
            FlowError::EmptyFlow
        );

        assert_eq!(
            FlowBuilder::new()
                .node("node", StaticNode::new("done"))
                .node("node", StaticNode::new("done"))
                .build()
                .unwrap_err(),
            FlowError::DuplicateNode(NodeId::new("node"))
        );

        assert_eq!(
            FlowBuilder::new()
                .node("node", StaticNode::new("done"))
                .start("missing")
                .build()
                .unwrap_err(),
            FlowError::MissingStart(NodeId::new("missing"))
        );

        assert_eq!(
            FlowBuilder::new()
                .node("node", StaticNode::new("done"))
                .route("missing", "next", "node")
                .build()
                .unwrap_err(),
            FlowError::MissingRouteSource(NodeId::new("missing"))
        );

        assert_eq!(
            FlowBuilder::new()
                .node("node", StaticNode::new("next"))
                .route("node", "next", "missing")
                .build()
                .unwrap_err(),
            FlowError::MissingRouteTarget(NodeId::new("missing"))
        );

        assert_eq!(
            FlowBuilder::new()
                .node("first", StaticNode::new("next"))
                .node("second", StaticNode::new("done"))
                .route("first", "next", "second")
                .route("first", "next", "second")
                .build()
                .unwrap_err(),
            FlowError::DuplicateRoute {
                from: NodeId::new("first"),
                action: ActionName::new("next")
            }
        );

        assert_eq!(
            FlowBuilder::new()
                .node("first", StaticNode::new("done"))
                .node("second", StaticNode::new("done"))
                .build()
                .unwrap_err(),
            FlowError::UnreachableNode(NodeId::new("second"))
        );
    }

    #[test]
    fn graph_analysis_reports_cycle_and_dag() {
        let cycle = FlowBuilder::new()
            .node("first", StaticNode::new("next"))
            .node("second", StaticNode::new("back"))
            .route("first", "next", "second")
            .route("second", "back", "first")
            .build()
            .unwrap();

        assert!(!cycle.analysis().is_dag);
        assert_eq!(cycle.analysis().topological_order, None);

        let dag = FlowBuilder::new()
            .node("first", StaticNode::new("next"))
            .node("second", StaticNode::new("done"))
            .route("first", "next", "second")
            .build()
            .unwrap();

        assert!(dag.analysis().is_dag);
        assert_eq!(
            dag.analysis().topological_order,
            Some(vec![NodeId::new("first"), NodeId::new("second")])
        );
    }

    #[tokio::test]
    async fn node_error_is_wrapped_as_flow_error() {
        let mut flow = FlowBuilder::new()
            .node("start", StaticNode::failing())
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let error = flow.run(&mut state).await.unwrap_err();

        match error {
            FlowError::NodeError(node_error) => {
                assert_eq!(node_error.node_id.as_str(), "start");
                assert_eq!(node_error.message, "node failed");
            }
            other => panic!("expected node error, got {other:?}"),
        }
    }
}
