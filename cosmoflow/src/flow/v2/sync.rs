use super::{FlowAnalysis, FlowError, FlowExecution, Route};
use crate::action::v2::{Action, ActionName};
use crate::node::v2::{FlowInput, IntoNodeAdapter, NodeAdapter, NodeError, NodeId, NodePhase};
use std::collections::HashMap;
use std::fmt;

/// Builder for a v2 synchronous flow.
pub struct FlowBuilder<S> {
    nodes: HashMap<NodeId, Box<dyn NodeAdapter<S>>>,
    node_order: Vec<NodeId>,
    routes: Vec<Route>,
    start: Option<NodeId>,
    duplicate_nodes: Vec<NodeId>,
}

impl<S> Default for FlowBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> FlowBuilder<S> {
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
    pub fn node<N, K>(mut self, id: impl Into<NodeId>, node: N) -> Self
    where
        N: IntoNodeAdapter<S, K>,
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
        self.nodes.insert(id, node.into_node_adapter());
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

/// A v2 synchronous flow.
pub struct Flow<S> {
    nodes: HashMap<NodeId, Box<dyn NodeAdapter<S>>>,
    node_order: Vec<NodeId>,
    routes: Vec<Route>,
    start: NodeId,
    analysis: FlowAnalysis,
}

impl<S> fmt::Debug for Flow<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Flow")
            .field("node_order", &self.node_order)
            .field("routes", &self.routes)
            .field("start", &self.start)
            .field("analysis", &self.analysis)
            .finish()
    }
}

impl<S> Flow<S> {
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
    pub fn run(&mut self, state: &mut S) -> Result<Action, FlowError> {
        Ok(self.run_recorded(state)?.final_action)
    }

    /// Run the flow and return an execution summary.
    pub fn run_recorded(&mut self, state: &mut S) -> Result<FlowExecution, FlowError> {
        let mut current_node_id = self.start.clone();
        let mut path = Vec::new();

        loop {
            let node = self
                .nodes
                .get_mut(&current_node_id)
                .ok_or_else(|| FlowError::NodeNotFound(current_node_id.clone()))?;
            let action = node.run(state, &current_node_id).map_err(FlowError::from)?;
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

impl<S> NodeAdapter<S> for Flow<S> {
    fn run(&mut self, state: &mut S, node_id: &NodeId) -> Result<Action, NodeError> {
        Flow::run(self, state)
            .map_err(|error| NodeError::new(NodePhase::Exec, node_id.clone(), error.to_string()))
    }
}

impl<S> IntoNodeAdapter<S, FlowInput> for Flow<S>
where
    S: 'static,
{
    fn into_node_adapter(self) -> Box<dyn NodeAdapter<S>> {
        Box::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SharedStore;
    use crate::node::v2::{Node, NodeContext};
    use crate::shared_store::backends::MemoryStorage;
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

    impl Node<MemoryStorage> for StaticNode {
        type Prep = ();
        type Output = ();
        type Error = TestError;

        fn prep(
            &mut self,
            _state: &MemoryStorage,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            if self.fail {
                return Err(TestError("node failed"));
            }
            Ok(())
        }

        fn exec(
            &mut self,
            _prep: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(())
        }

        fn post(
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

    #[derive(Default)]
    struct TypedState {
        visits: Vec<String>,
        value: Option<String>,
    }

    struct TypedNode {
        action: Action,
    }

    impl TypedNode {
        fn new(action: impl Into<Action>) -> Self {
            Self {
                action: action.into(),
            }
        }
    }

    impl Node<TypedState> for TypedNode {
        type Prep = ();
        type Output = ();
        type Error = TestError;

        fn prep(
            &mut self,
            _state: &TypedState,
            _context: &NodeContext,
        ) -> Result<Self::Prep, Self::Error> {
            Ok(())
        }

        fn exec(
            &mut self,
            _prep: &Self::Prep,
            _context: &NodeContext,
        ) -> Result<Self::Output, Self::Error> {
            Ok(())
        }

        fn post(
            &mut self,
            state: &mut TypedState,
            _prep: Self::Prep,
            _output: Self::Output,
            context: &NodeContext,
        ) -> Result<Action, Self::Error> {
            state.visits.push(context.node_id.as_str().to_string());
            state.value = Some(self.action.as_str().to_string());
            Ok(self.action.clone())
        }
    }

    #[test]
    fn single_node_flow_uses_default_start_and_naturally_terminates() {
        let mut flow = FlowBuilder::new()
            .node("start", StaticNode::new("done"))
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(flow.start().as_str(), "start");
        assert_eq!(execution.final_action, Action::new("done"));
        assert_eq!(execution.last_node_id.as_str(), "start");
        assert_eq!(execution.steps, 1);
        assert_eq!(execution.path, vec![NodeId::new("start")]);
    }

    #[test]
    fn flow_runs_with_plain_typed_state() {
        let mut flow = FlowBuilder::new()
            .node("first", TypedNode::new("next"))
            .node("second", TypedNode::new("done"))
            .route("first", "next", "second")
            .build()
            .unwrap();
        let mut state = TypedState::default();

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(execution.final_action, Action::new("done"));
        assert_eq!(
            execution.path,
            vec![NodeId::new("first"), NodeId::new("second")]
        );
        assert_eq!(state.visits, vec!["first", "second"]);
        assert_eq!(state.value, Some("done".to_string()));
    }

    #[test]
    fn nested_flow_runs_with_plain_typed_state() {
        let flow_a = FlowBuilder::new()
            .node("inner_start", TypedNode::new("next"))
            .node("inner_end", TypedNode::new("inner_done"))
            .route("inner_start", "next", "inner_end")
            .build()
            .unwrap();
        let mut flow_b = FlowBuilder::new().node("nested", flow_a).build().unwrap();
        let mut state = TypedState::default();

        let execution = flow_b.run_recorded(&mut state).unwrap();

        assert_eq!(execution.final_action, Action::new("inner_done"));
        assert_eq!(execution.path, vec![NodeId::new("nested")]);
        assert_eq!(state.visits, vec!["inner_start", "inner_end"]);
        assert_eq!(state.value, Some("inner_done".to_string()));
    }

    #[test]
    fn run_returns_final_action() {
        let mut flow = FlowBuilder::new()
            .node("start", StaticNode::new("done"))
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let action = flow.run(&mut state).unwrap();

        assert_eq!(action, Action::new("done"));
    }

    #[test]
    fn first_added_node_becomes_start() {
        let flow = FlowBuilder::new()
            .node("first", StaticNode::new("next"))
            .node("second", StaticNode::new("done"))
            .route("first", "next", "second")
            .build()
            .unwrap();

        assert_eq!(flow.start().as_str(), "first");
    }

    #[test]
    fn explicit_start_overrides_default_start() {
        let mut flow = FlowBuilder::new()
            .node("first", StaticNode::new("done"))
            .node("second", StaticNode::new("next"))
            .start("second")
            .route("second", "next", "first")
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(flow.start().as_str(), "second");
        assert_eq!(
            execution.path,
            vec![NodeId::new("second"), NodeId::new("first")]
        );
        assert_eq!(execution.final_action, Action::new("done"));
    }

    #[test]
    fn routes_by_action_name() {
        let mut flow = FlowBuilder::new()
            .node("first", StaticNode::new("next"))
            .node("second", StaticNode::new("done"))
            .route("first", "next", "second")
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(
            execution.path,
            vec![NodeId::new("first"), NodeId::new("second")]
        );
    }

    #[test]
    fn action_params_do_not_affect_routing() {
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

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(
            execution.path,
            vec![NodeId::new("first"), NodeId::new("second")]
        );
    }

    #[test]
    fn no_matching_route_naturally_terminates() {
        let mut flow = FlowBuilder::new()
            .node("first", StaticNode::new("done"))
            .node("second", StaticNode::new("unused"))
            .route("first", "other", "second")
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow.run_recorded(&mut state).unwrap();

        assert_eq!(execution.path, vec![NodeId::new("first")]);
        assert_eq!(execution.final_action, Action::new("done"));
    }

    #[test]
    fn build_fails_for_empty_flow() {
        let error = FlowBuilder::<MemoryStorage>::new().build().unwrap_err();

        assert_eq!(error, FlowError::EmptyFlow);
    }

    #[test]
    fn build_fails_for_duplicate_node() {
        let error = FlowBuilder::new()
            .node("node", StaticNode::new("done"))
            .node("node", StaticNode::new("done"))
            .build()
            .unwrap_err();

        assert_eq!(error, FlowError::DuplicateNode(NodeId::new("node")));
    }

    #[test]
    fn build_fails_for_missing_start() {
        let error = FlowBuilder::new()
            .node("node", StaticNode::new("done"))
            .start("missing")
            .build()
            .unwrap_err();

        assert_eq!(error, FlowError::MissingStart(NodeId::new("missing")));
    }

    #[test]
    fn build_fails_for_missing_route_source() {
        let error = FlowBuilder::new()
            .node("node", StaticNode::new("done"))
            .route("missing", "next", "node")
            .build()
            .unwrap_err();

        assert_eq!(error, FlowError::MissingRouteSource(NodeId::new("missing")));
    }

    #[test]
    fn build_fails_for_missing_route_target() {
        let error = FlowBuilder::new()
            .node("node", StaticNode::new("next"))
            .route("node", "next", "missing")
            .build()
            .unwrap_err();

        assert_eq!(error, FlowError::MissingRouteTarget(NodeId::new("missing")));
    }

    #[test]
    fn build_fails_for_duplicate_route() {
        let error = FlowBuilder::new()
            .node("first", StaticNode::new("next"))
            .node("second", StaticNode::new("done"))
            .route("first", "next", "second")
            .route("first", "next", "second")
            .build()
            .unwrap_err();

        assert_eq!(
            error,
            FlowError::DuplicateRoute {
                from: NodeId::new("first"),
                action: ActionName::new("next")
            }
        );
    }

    #[test]
    fn build_fails_for_unreachable_node() {
        let error = FlowBuilder::new()
            .node("first", StaticNode::new("done"))
            .node("second", StaticNode::new("done"))
            .build()
            .unwrap_err();

        assert_eq!(error, FlowError::UnreachableNode(NodeId::new("second")));
    }

    #[test]
    fn cycle_builds_and_analysis_reports_non_dag() {
        let flow = FlowBuilder::new()
            .node("first", StaticNode::new("next"))
            .node("second", StaticNode::new("back"))
            .route("first", "next", "second")
            .route("second", "back", "first")
            .build()
            .unwrap();

        assert!(!flow.analysis().is_dag);
        assert_eq!(flow.analysis().topological_order, None);
    }

    #[test]
    fn dag_analysis_returns_topological_order() {
        let flow = FlowBuilder::new()
            .node("first", StaticNode::new("next"))
            .node("second", StaticNode::new("done"))
            .route("first", "next", "second")
            .build()
            .unwrap();

        assert!(flow.analysis().is_dag);
        assert_eq!(
            flow.analysis().topological_order,
            Some(vec![NodeId::new("first"), NodeId::new("second")])
        );
    }

    #[test]
    fn node_error_is_wrapped_as_flow_error() {
        let mut flow = FlowBuilder::new()
            .node("start", StaticNode::failing())
            .build()
            .unwrap();
        let mut state = MemoryStorage::new();

        let error = flow.run(&mut state).unwrap_err();

        match error {
            FlowError::NodeError(node_error) => {
                assert_eq!(node_error.node_id.as_str(), "start");
                assert_eq!(node_error.message, "node failed");
            }
            other => panic!("expected node error, got {other:?}"),
        }
    }

    #[test]
    fn nested_flow_runs_as_one_parent_node_and_returns_final_action() {
        let flow_a = FlowBuilder::new()
            .node("inner_start", StaticNode::new("next"))
            .node("inner_end", StaticNode::new("inner_done"))
            .route("inner_start", "next", "inner_end")
            .build()
            .unwrap();
        let mut flow_b = FlowBuilder::new().node("nested", flow_a).build().unwrap();
        let mut state = MemoryStorage::new();

        let execution = flow_b.run_recorded(&mut state).unwrap();

        assert_eq!(execution.final_action, Action::new("inner_done"));
        assert_eq!(execution.path, vec![NodeId::new("nested")]);
        assert_eq!(
            state.get::<bool>("visited:inner_start").unwrap(),
            Some(true)
        );
        assert_eq!(state.get::<bool>("visited:inner_end").unwrap(), Some(true));
    }

    #[test]
    fn nested_flow_error_is_reported_as_parent_node_exec_error() {
        let flow_a = FlowBuilder::new()
            .node("inner", StaticNode::failing())
            .build()
            .unwrap();
        let mut flow_b = FlowBuilder::new().node("nested", flow_a).build().unwrap();
        let mut state = MemoryStorage::new();

        let error = flow_b.run(&mut state).unwrap_err();

        match error {
            FlowError::NodeError(node_error) => {
                assert_eq!(node_error.phase, NodePhase::Exec);
                assert_eq!(node_error.node_id.as_str(), "nested");
                assert!(
                    node_error.message.contains("node execution error"),
                    "message was: {}",
                    node_error.message
                );
            }
            other => panic!("expected parent node error, got {other:?}"),
        }
    }
}
