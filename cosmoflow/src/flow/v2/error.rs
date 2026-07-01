use crate::action::v2::ActionName;
use crate::node::v2::{NodeError, NodeId};
use thiserror::Error;

/// Errors produced while building or running a flow.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FlowError {
    /// Flow contains no nodes.
    #[error("flow contains no nodes")]
    EmptyFlow,

    /// A node id was registered more than once.
    #[error("duplicate node: {0}")]
    DuplicateNode(NodeId),

    /// The configured start node does not exist.
    #[error("start node not found: {0}")]
    MissingStart(NodeId),

    /// A route source node does not exist.
    #[error("route source node not found: {0}")]
    MissingRouteSource(NodeId),

    /// A route target node does not exist.
    #[error("route target node not found: {0}")]
    MissingRouteTarget(NodeId),

    /// A node has more than one route for the same action.
    #[error("duplicate route from node '{from}' for action '{action}'")]
    DuplicateRoute {
        /// Source node id.
        from: NodeId,
        /// Action name matched by the route.
        action: ActionName,
    },

    /// A node cannot be reached from the configured start node.
    #[error("unreachable node: {0}")]
    UnreachableNode(NodeId),

    /// A node referenced during execution was not found.
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),

    /// A node failed while running.
    #[error("node execution error: {0}")]
    NodeError(NodeError),
}

impl From<NodeError> for FlowError {
    fn from(error: NodeError) -> Self {
        Self::NodeError(error)
    }
}
