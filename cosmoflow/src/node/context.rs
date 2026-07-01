use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Unique identifier for one node execution run.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExecutionId(String);

impl ExecutionId {
    /// Create a fresh execution identifier.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Return the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ExecutionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable identifier for a node within a flow or standalone execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    /// Create a node identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Return the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for NodeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Minimal context passed through one node execution.
#[derive(Debug, Clone)]
pub struct NodeContext {
    /// Unique identifier for this node execution.
    pub execution_id: ExecutionId,
    /// Identifier of the node being executed.
    pub node_id: NodeId,
    /// Caller-provided metadata for diagnostics or custom execution state.
    pub metadata: HashMap<String, Value>,
}

impl NodeContext {
    /// Create a context for a node with empty metadata.
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        Self {
            execution_id: ExecutionId::new(),
            node_id: node_id.into(),
            metadata: HashMap::new(),
        }
    }

    /// Insert metadata and return the updated context.
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_context_uses_empty_metadata() {
        let context = NodeContext::new("start");

        assert_eq!(context.node_id.as_str(), "start");
        assert!(context.metadata.is_empty());
        assert!(!context.execution_id.as_str().is_empty());
    }

    #[test]
    fn context_builders_update_explicit_fields() {
        let context =
            NodeContext::new(NodeId::new("worker")).with_metadata("source", json!("test"));

        assert_eq!(context.node_id.as_str(), "worker");
        assert_eq!(context.metadata.get("source"), Some(&json!("test")));
    }
}
