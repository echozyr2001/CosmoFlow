use thiserror::Error;

/// Simple error type for Node operations
#[derive(Debug, Error)]
pub enum NodeError {
    /// An error that occurs during the preparation phase of a node.
    #[error("Preparation error: {0}")]
    PreparationError(String),
    /// An error that occurs during node execution.
    #[error("Execution error: {0}")]
    ExecutionError(String),
    /// An error that occurs during the post-processing phase of a node.
    #[error("Post-processing error: {0}")]
    PostProcessingError(String),
    /// An error that occurs in the storage backend.
    #[error("Storage error: {0}")]
    StorageError(String),
    /// An error that occurs during input validation.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<String> for NodeError {
    fn from(s: String) -> Self {
        NodeError::ExecutionError(s)
    }
}

impl From<&str> for NodeError {
    fn from(s: &str) -> Self {
        NodeError::ExecutionError(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_node_error_display() {
        let exec_error = NodeError::ExecutionError("execution failed".to_string());
        assert_eq!(exec_error.to_string(), "Execution error: execution failed");

        let storage_error = NodeError::StorageError("storage failed".to_string());
        assert_eq!(storage_error.to_string(), "Storage error: storage failed");

        let validation_error = NodeError::ValidationError("validation failed".to_string());
        assert_eq!(
            validation_error.to_string(),
            "Validation error: validation failed"
        );

        let prep_error = NodeError::PreparationError("prep failed".to_string());
        assert_eq!(prep_error.to_string(), "Preparation error: prep failed");

        let post_error = NodeError::PostProcessingError("post failed".to_string());
        assert_eq!(post_error.to_string(), "Post-processing error: post failed");
    }

    #[test]
    fn test_node_error_debug() {
        let error = NodeError::ExecutionError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ExecutionError"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_node_error_from_str() {
        let error: NodeError = "test error message".into();
        match error {
            NodeError::ExecutionError(msg) => assert_eq!(msg, "test error message"),
            _ => panic!("Expected ExecutionError variant"),
        }
    }

    #[test]
    fn test_node_error_from_string() {
        let error: NodeError = "test error message".to_string().into();
        match error {
            NodeError::ExecutionError(msg) => assert_eq!(msg, "test error message"),
            _ => panic!("Expected ExecutionError variant"),
        }
    }

    #[test]
    fn test_node_error_is_error_trait() {
        let error = NodeError::ExecutionError("test".to_string());

        // Test that it implements std::error::Error
        let _: &dyn std::error::Error = &error;

        // Test source (should be None for our simple errors)
        assert!(error.source().is_none());
    }

    #[test]
    fn test_node_error_equality() {
        let error1 = NodeError::ExecutionError("same message".to_string());
        let error2 = NodeError::ExecutionError("same message".to_string());
        let error3 = NodeError::ExecutionError("different message".to_string());
        let error4 = NodeError::StorageError("same message".to_string());

        // Test Debug implementation allows comparison in tests
        assert_eq!(format!("{:?}", error1), format!("{:?}", error2));
        assert_ne!(format!("{:?}", error1), format!("{:?}", error3));
        assert_ne!(format!("{:?}", error1), format!("{:?}", error4));
    }

    #[test]
    fn test_node_error_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<NodeError>();
        assert_sync::<NodeError>();
    }
}
