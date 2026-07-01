use crate::action::ActionName;
use crate::node::NodeId;

/// Directed transition between two nodes for one action name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    /// Source node id.
    pub from: NodeId,
    /// Action name that triggers this route.
    pub action: ActionName,
    /// Target node id.
    pub to: NodeId,
}

impl Route {
    /// Create a route from one node to another.
    pub fn new(
        from: impl Into<NodeId>,
        action: impl Into<ActionName>,
        to: impl Into<NodeId>,
    ) -> Self {
        Self {
            from: from.into(),
            action: action.into(),
            to: to.into(),
        }
    }
}
