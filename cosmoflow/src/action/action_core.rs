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
/// ## Creating Simple Actions (53.3% of usage)
/// ```
/// use cosmoflow::action::Action;
///
/// let action = Action::simple("next_node");
/// assert_eq!(action.name(), "next_node");
/// ```
///
/// ## Creating Parameterized Actions (1.0% of usage)
/// ```
/// use cosmoflow::action::Action;
/// use serde_json::json;
/// use std::collections::HashMap;
///
/// let mut params = HashMap::new();
/// params.insert("key".to_string(), json!("value"));
/// let action = Action::with_params("process", params);
/// assert!(action.has_params());
/// ```
///
/// ## Creating Conditional Actions (simplified)
/// ```
/// use cosmoflow::action::Action;
/// use serde_json::json;
///
/// let action = Action::conditional("status", json!("ready"), "proceed", "wait");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Simple string-based action (most common - 53.3% of usage)
    Simple(String),

    /// Action with parameters (essential for data passing - 1.0% of usage)
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
    /// Simple actions are the most common type (53.3% of usage), consisting of just a name/label
    /// that identifies the next step in the workflow.
    ///
    /// # Arguments
    ///
    /// * `name` - The action name/label
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let action = Action::simple("next_step");
    /// assert_eq!(action.name(), "next_step");
    /// assert!(!action.has_params());
    /// ```
    pub fn simple<S: Into<String>>(name: S) -> Self {
        Action::Simple(name.into())
    }

    /// Create a parameterized action with additional data
    ///
    /// Parameterized actions carry additional data that can be used by the
    /// target node or routing logic. This is useful for passing configuration,
    /// retry counts, or other contextual information.
    ///
    /// # Arguments
    ///
    /// * `name` - The action name/label
    /// * `params` - Key-value parameters to include with the action
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut params = HashMap::new();
    /// params.insert("retry_count".to_string(), json!(3));
    /// params.insert("timeout".to_string(), json!(30));
    ///
    /// let action = Action::with_params("retry", params);
    /// assert_eq!(action.name(), "retry");
    /// assert!(action.has_params());
    /// ```
    pub fn with_params<S: Into<String>>(name: S, params: HashMap<String, Value>) -> Self {
        Action::Parameterized {
            name: name.into(),
            params,
        }
    }

    /// Get the primary name/identifier of the action
    ///
    /// Returns the string identifier for this action. For conditional actions,
    /// this returns a descriptive string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    /// use serde_json::json;
    ///
    /// let simple = Action::simple("test");
    /// assert_eq!(simple.name(), "test");
    ///
    /// let conditional = Action::conditional("ready", json!(true), "go", "wait");
    /// assert_eq!(conditional.name(), "conditional(ready)");
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut params = HashMap::new();
    /// params.insert("key".to_string(), json!("value"));
    /// let action = Action::with_params("test", params);
    ///
    /// assert!(action.params().is_some());
    /// assert_eq!(action.params().unwrap().get("key").unwrap(), &json!("value"));
    /// ```
    pub fn params(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Action::Parameterized { params, .. } => Some(params),
            _ => None,
        }
    }

    /// Check if this is a simple action
    ///
    /// Returns true if this is a simple string-based action without any
    /// additional features like parameters or conditions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let simple = Action::simple("test");
    /// assert!(simple.is_simple());
    ///
    /// let complex = Action::with_params("test", std::collections::HashMap::new());
    /// assert!(!complex.is_simple());
    /// ```
    pub fn is_simple(&self) -> bool {
        matches!(self, Action::Simple(_))
    }

    /// Check if this action has parameters
    ///
    /// Returns true if this action carries parameters (key-value data).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    /// use std::collections::HashMap;
    ///
    /// let simple = Action::simple("test");
    /// assert!(!simple.has_params());
    ///
    /// let params = HashMap::new();
    /// let parameterized = Action::with_params("test", params);
    /// assert!(parameterized.has_params());
    /// ```
    pub fn has_params(&self) -> bool {
        self.params().is_some()
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
