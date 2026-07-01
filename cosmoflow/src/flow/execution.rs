use crate::action::Action;
use crate::node::NodeId;

/// Summary of one successful flow execution.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowExecution {
    /// Final action returned by the last executed node.
    pub final_action: Action,
    /// Last executed node id.
    pub last_node_id: NodeId,
    /// Number of executed nodes.
    pub steps: usize,
    /// Executed node ids in order.
    pub path: Vec<NodeId>,
}
