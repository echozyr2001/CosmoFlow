use crate::node::NodeError;
use thiserror::Error;

/// Errors that can occur during flow execution
#[derive(Error, Debug, Clone)]
pub enum FlowError {
    /// Node with the given ID was not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// No route found for the given action from the current node
    #[error("No route found from node '{0}' for action '{1}'")]
    NoRouteFound(String, String),

    /// Cycle detected in the flow
    #[error("Cycle detected in flow: {}", .0.join(" -> "))]
    CycleDetected(Vec<String>),

    /// Maximum execution steps exceeded
    #[error("Maximum execution steps exceeded: {0}")]
    MaxStepsExceeded(usize),

    /// Node execution error
    #[error("Node execution error: {0}")]
    NodeError(String),

    /// Invalid flow configuration
    #[error("Invalid flow configuration: {0}")]
    InvalidConfiguration(String),
}

impl From<NodeError> for FlowError {
    fn from(err: NodeError) -> Self {
        FlowError::NodeError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::FlowError;

    #[test]
    fn test_error_display_formats() {
        // Test NodeNotFound error
        let error = FlowError::NodeNotFound("test_node".to_string());
        assert_eq!(error.to_string(), "Node not found: test_node");

        // Test NoRouteFound error
        let error = FlowError::NoRouteFound("node1".to_string(), "action1".to_string());
        assert_eq!(
            error.to_string(),
            "No route found from node 'node1' for action 'action1'"
        );

        // Test CycleDetected error
        let error = FlowError::CycleDetected(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "A".to_string(),
        ]);
        assert_eq!(
            error.to_string(),
            "Cycle detected in flow: A -> B -> C -> A"
        );

        // Test MaxStepsExceeded error
        let error = FlowError::MaxStepsExceeded(100);
        assert_eq!(error.to_string(), "Maximum execution steps exceeded: 100");

        // Test NodeError
        let error = FlowError::NodeError("Something went wrong".to_string());
        assert_eq!(
            error.to_string(),
            "Node execution error: Something went wrong"
        );

        // Test InvalidConfiguration error
        let error = FlowError::InvalidConfiguration("Missing required field".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid flow configuration: Missing required field"
        );
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = FlowError::NodeNotFound("test".to_string());

        // Test that Error trait is implemented
        let _: &dyn std::error::Error = &error;

        // Test Debug trait
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NodeNotFound"));
    }
}
