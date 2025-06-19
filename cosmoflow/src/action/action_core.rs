use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Represents an action that controls flow between nodes in CosmoFlow workflows.
///
/// This simplified version focuses on the most commonly used action patterns
/// while eliminating complex features that had minimal real-world adoption.
///
/// # Examples
///
/// ```
/// use cosmoflow::action::Action;
/// use serde_json::json;
/// use std::collections::HashMap;
///
/// // Simple action
/// let simple = Action::simple("next_node");
/// assert_eq!(simple.name(), "next_node");
/// assert!(simple.is_simple());
///
/// // Single parameter action (convenience method)
/// let single_param = Action::with_param("retry", "count", json!(3));
/// assert!(single_param.is_parameterized());
/// assert_eq!(single_param.get_param("count"), Some(&json!(3)));
///
/// // Multiple parameters action
/// let mut params = HashMap::new();
/// params.insert("timeout".to_string(), json!(30));
/// params.insert("retries".to_string(), json!(3));
/// let multi_param = Action::with_params("process", params);
/// assert_eq!(multi_param.param_count(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Simple string-based action
    Simple(String),

    /// Action with parameters
    Parameterized {
        /// The name of the action.
        name: String,
        /// The parameters for the action.
        #[serde(default)]
        params: HashMap<String, Value>,
    },
}

impl Action {
    /// Create a simple action from a string
    ///
    /// Simple actions are the most common type, consisting of just a name/label
    /// that identifies the next step in the workflow.
    pub fn simple<S: Into<String>>(name: S) -> Self {
        Action::Simple(name.into())
    }

    /// Create a parameterized action with additional data
    ///
    /// Parameterized actions carry additional data that can be used by the
    /// target node or routing logic.
    pub fn with_params<S: Into<String>>(name: S, params: HashMap<String, Value>) -> Self {
        Action::Parameterized {
            name: name.into(),
            params,
        }
    }

    /// Create a parameterized action with a single parameter (convenience method)
    ///
    /// This is a convenience method for the common case of creating an action
    /// with just one parameter, reducing boilerplate code.
    pub fn with_param<S: Into<String>, K: Into<String>>(name: S, key: K, value: Value) -> Self {
        let mut params = HashMap::new();
        params.insert(key.into(), value);
        Action::Parameterized {
            name: name.into(),
            params,
        }
    }

    /// Get the primary name/identifier of the action
    pub fn name(&self) -> String {
        match self {
            Action::Simple(name) => name.clone(),
            Action::Parameterized { name, .. } => name.clone(),
        }
    }

    /// Get parameters if this is a parameterized action
    ///
    /// Returns a reference to the parameters map if this action carries parameters,
    /// or None if it doesn't.
    pub fn params(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Action::Parameterized { params, .. } => Some(params),
            _ => None,
        }
    }

    /// Get a specific parameter value by key
    ///
    /// Returns a reference to the parameter value if it exists, or None if
    /// the action doesn't have parameters or the key doesn't exist.
    pub fn get_param(&self, key: &str) -> Option<&Value> {
        self.params()?.get(key)
    }

    /// Check if this is a simple action
    ///
    /// Returns true if this is a simple string-based action without any parameters.
    pub fn is_simple(&self) -> bool {
        matches!(self, Action::Simple(_))
    }

    /// Check if this is a parameterized action
    ///
    /// Returns true if this action carries parameters (key-value data).
    /// This is the inverse of `is_simple()`.
    pub fn is_parameterized(&self) -> bool {
        matches!(self, Action::Parameterized { .. })
    }

    /// Check if this action has a specific parameter
    ///
    /// Returns true if this action has parameters and contains the specified key.
    pub fn has_param(&self, key: &str) -> bool {
        self.params().is_some_and(|params| params.contains_key(key))
    }

    /// Get the number of parameters
    ///
    /// Returns the number of parameters this action carries, or 0 if it's a simple action.
    pub fn param_count(&self) -> usize {
        self.params().map_or(0, |params| params.len())
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Simple(name) => write!(f, "{name}"),
            Action::Parameterized { name, params } => {
                write!(
                    f,
                    "{}({})",
                    name,
                    params
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

impl Default for Action {
    fn default() -> Self {
        Action::Simple("default".to_string())
    }
}

impl From<String> for Action {
    fn from(s: String) -> Self {
        Action::Simple(s)
    }
}

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        Action::Simple(s.to_string())
    }
}

impl From<Action> for String {
    fn from(action: Action) -> Self {
        action.name()
    }
}
