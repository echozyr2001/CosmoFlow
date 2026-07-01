use super::context::NodeId;
use super::phase::NodePhase;
use std::fmt;

/// Error produced by the node executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeError {
    /// Execution phase where the error occurred.
    pub phase: NodePhase,
    /// Identifier of the node that failed.
    pub node_id: NodeId,
    /// Human-readable error message.
    pub message: String,
}

impl NodeError {
    /// Create a phase-aware node error.
    pub fn new(phase: NodePhase, node_id: impl Into<NodeId>, message: impl Into<String>) -> Self {
        Self {
            phase,
            node_id: node_id.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "node '{}' {} error: {}",
            self.node_id, self.phase, self.message
        )
    }
}

impl std::error::Error for NodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_node_phase_and_message() {
        let error = NodeError::new(NodePhase::Exec, "worker", "failed");

        assert_eq!(error.to_string(), "node 'worker' exec error: failed");
    }
}
