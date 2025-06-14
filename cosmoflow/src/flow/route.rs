use crate::shared_store::SharedStore;
use serde_json::Value;

/// Represents a route from one node to another based on an action
#[derive(Debug, Clone)]
pub struct Route {
    /// The action that triggers this route
    pub action: String,
    /// The target node ID, or None for terminal routes
    pub target_node_id: Option<String>,
    /// Optional condition that must be met for this route to be taken
    pub condition: Option<RouteCondition>,
}

/// Conditions for route evaluation
#[derive(Debug)]
pub enum RouteCondition {
    /// Always true
    Always,
    /// Check if a key exists in the shared store
    KeyExists(String),
    /// Check if a key equals a specific value
    KeyEquals(String, Value),
}

impl Clone for RouteCondition {
    fn clone(&self) -> Self {
        match self {
            RouteCondition::Always => RouteCondition::Always,
            RouteCondition::KeyExists(key) => RouteCondition::KeyExists(key.clone()),
            RouteCondition::KeyEquals(key, value) => {
                RouteCondition::KeyEquals(key.clone(), value.clone())
            }
        }
    }
}

impl RouteCondition {
    /// Evaluate the condition against the shared store
    pub fn evaluate<S: SharedStore>(&self, store: &S) -> bool {
        match self {
            RouteCondition::Always => true,
            RouteCondition::KeyExists(key) => store.contains_key(key).unwrap_or(false),
            RouteCondition::KeyEquals(key, expected_value) => {
                if let Ok(Some(actual_value)) = store.get::<Value>(key) {
                    &actual_value == expected_value
                } else {
                    false
                }
            }
        }
    }
}
