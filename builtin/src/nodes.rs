//! Complete Node constructors for CosmoFlow workflows
//!
//! This module provides high-level Node constructors that wrap the basic NodeBackend
//! implementations with complete execution capabilities including retry logic,
//! error handling, and execution context management.

use std::time::Duration;

use node::Node;
use serde_json::Value;

use crate::basic::*;

/// Generic node creation functions for custom storage backends
pub mod generic {
    use storage::StorageBackend;

    use super::*;

    /// Create a log node with a custom storage backend
    pub fn log_node<S: StorageBackend>(message: impl Into<String>) -> Node<LogNodeBackend, S> {
        Node::new(log(message))
    }

    /// Create a set value node with a custom storage backend
    pub fn set_value_node<S: StorageBackend>(
        key: impl Into<String>,
        value: Value,
    ) -> Node<SetValueNodeBackend, S> {
        Node::new(set_value(key, value))
    }

    /// Create a delay node with a custom storage backend
    pub fn delay_node<S: StorageBackend>(duration: Duration) -> Node<DelayNodeBackend, S> {
        Node::new(delay(duration))
    }

    /// Create a get value node with a custom storage backend
    pub fn get_value_node<S: StorageBackend>(
        key: impl Into<String>,
        output_key: impl Into<String>,
    ) -> Node<GetValueNodeBackend<impl Fn(Option<Value>) -> Value + Send + Sync>, S> {
        Node::new(get_value(key, output_key))
    }

    /// Create a conditional node with a custom storage backend
    pub fn conditional_node<F, S: StorageBackend>(
        condition: F,
        if_true: action::Action,
        if_false: action::Action,
    ) -> Node<ConditionalNodeBackend<F, S>, S>
    where
        F: Fn(&shared_store::SharedStore<S>) -> bool + Send + Sync,
    {
        Node::new(ConditionalNodeBackend::new(condition, if_true, if_false))
    }
}
