//! Complete Node constructors for CosmoFlow workflows
//!
//! This module provides high-level Node constructors that wrap the basic NodeBackend
//! implementations with complete execution capabilities including retry logic,
//! error handling, and execution context management.

use std::time::Duration;

use node::Node;
use serde_json::Value;
use shared_store::{MemoryStorage, StorageBackend};

use crate::basic::*;

/// Create a complete log node that can be executed directly with MemoryStorage
pub fn log_node<S: Into<String>>(message: S) -> Node<LogNodeBackend, MemoryStorage> {
    Node::new(log(message))
}

/// Create a complete log node with custom retry configuration
pub fn log_node_with_retries<S: Into<String>>(
    message: S,
    max_retries: usize,
    retry_delay: Duration,
) -> Node<LogNodeBackend, MemoryStorage> {
    Node::new(
        log(message)
            .with_retries(max_retries)
            .with_retry_delay(retry_delay),
    )
}

/// Create a complete set value node that can be executed directly with MemoryStorage
pub fn set_value_node<S: Into<String>>(
    key: S,
    value: Value,
) -> Node<SetValueNodeBackend, MemoryStorage> {
    Node::new(set_value(key, value))
}

/// Create a complete set value node with custom retry configuration
pub fn set_value_node_with_retries<S: Into<String>>(
    key: S,
    value: Value,
    max_retries: usize,
) -> Node<SetValueNodeBackend, MemoryStorage> {
    Node::new(set_value(key, value).with_retries(max_retries))
}

/// Create a complete delay node that can be executed directly with MemoryStorage
pub fn delay_node(duration: Duration) -> Node<DelayNodeBackend, MemoryStorage> {
    Node::new(delay(duration))
}

/// Create a complete delay node with custom retry configuration
pub fn delay_node_with_retries(
    duration: Duration,
    max_retries: usize,
) -> Node<DelayNodeBackend, MemoryStorage> {
    Node::new(delay(duration).with_retries(max_retries))
}

/// Create a complete get value node that can be executed directly with MemoryStorage
pub fn get_value_node<S1: Into<String>, S2: Into<String>>(
    key: S1,
    output_key: S2,
) -> Node<GetValueNodeBackend<impl Fn(Option<Value>) -> Value + Send + Sync>, MemoryStorage> {
    Node::new(get_value(key, output_key))
}

/// Create a complete get value node with custom transform function
pub fn get_value_node_with_transform<S1, S2, F>(
    key: S1,
    output_key: S2,
    transform: F,
) -> Node<GetValueNodeBackend<F>, MemoryStorage>
where
    S1: Into<String>,
    S2: Into<String>,
    F: Fn(Option<Value>) -> Value + Send + Sync,
{
    Node::new(GetValueNodeBackend::new(
        key,
        output_key,
        transform,
        action::Action::simple("continue"),
    ))
}

/// Create a complete conditional node that can be executed directly with MemoryStorage
pub fn conditional_node<F>(
    condition: F,
    if_true: action::Action,
    if_false: action::Action,
) -> Node<ConditionalNodeBackend<F, MemoryStorage>, MemoryStorage>
where
    F: Fn(&shared_store::SharedStore<MemoryStorage>) -> bool + Send + Sync,
{
    Node::new(ConditionalNodeBackend::new(condition, if_true, if_false))
}

/// Create a complete conditional node with custom retry configuration
pub fn conditional_node_with_retries<F>(
    condition: F,
    if_true: action::Action,
    if_false: action::Action,
    max_retries: usize,
) -> Node<ConditionalNodeBackend<F, MemoryStorage>, MemoryStorage>
where
    F: Fn(&shared_store::SharedStore<MemoryStorage>) -> bool + Send + Sync,
{
    Node::new(ConditionalNodeBackend::new(condition, if_true, if_false).with_retries(max_retries))
}

/// Generic node creation functions for custom storage backends
pub mod generic {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use shared_store::{MemoryStorage, SharedStore};
    use std::time::Duration;

    #[tokio::test]
    async fn test_log_node() {
        let mut node = log_node("Test message");
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let action = node.run(&mut store).await.unwrap();
        assert_eq!(action, action::Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_set_value_node() {
        let mut node = set_value_node("test_key", json!("test_value"));
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let action = node.run(&mut store).await.unwrap();
        assert_eq!(action, action::Action::simple("continue"));
        assert_eq!(store.get("test_key").unwrap(), Some(json!("test_value")));
    }

    #[tokio::test]
    async fn test_delay_node() {
        let mut node = delay_node(Duration::from_millis(1));
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let start = std::time::Instant::now();
        let action = node.run(&mut store).await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(1));
        assert_eq!(action, action::Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_get_value_node() {
        let mut store = SharedStore::with_storage(MemoryStorage::new());
        store
            .set("input_key".to_string(), json!("input_value"))
            .unwrap();

        let mut node = get_value_node("input_key", "output_key");
        let action = node.run(&mut store).await.unwrap();

        assert_eq!(action, action::Action::simple("continue"));
        assert_eq!(store.get("output_key").unwrap(), Some(json!("input_value")));
    }

    #[tokio::test]
    async fn test_get_value_node_with_transform() {
        let mut store = SharedStore::with_storage(MemoryStorage::new());
        store.set("input".to_string(), json!(42)).unwrap();

        let transform = |value: Option<Value>| match value {
            Some(v) => json!(format!("Number: {}", v)),
            None => json!("No value"),
        };

        let mut node = get_value_node_with_transform("input", "output", transform);
        let action = node.run(&mut store).await.unwrap();

        assert_eq!(action, action::Action::simple("continue"));
        assert_eq!(store.get("output").unwrap(), Some(json!("Number: 42")));
    }

    #[tokio::test]
    async fn test_conditional_node() {
        let mut store = SharedStore::with_storage(MemoryStorage::new());
        store.set("flag".to_string(), json!(true)).unwrap();

        let condition = |store: &SharedStore<MemoryStorage>| {
            store.get("flag").unwrap_or(Some(json!(false))) == Some(json!(true))
        };

        let mut node = conditional_node(
            condition,
            action::Action::simple("true_branch"),
            action::Action::simple("false_branch"),
        );

        let action = node.run(&mut store).await.unwrap();
        assert_eq!(action, action::Action::simple("true_branch"));
    }

    #[tokio::test]
    async fn test_generic_nodes() {
        let mut node = generic::log_node::<MemoryStorage>("Generic test");
        let mut store = SharedStore::with_storage(MemoryStorage::new());

        let action = node.run(&mut store).await.unwrap();
        assert_eq!(action, action::Action::simple("continue"));
    }
}
