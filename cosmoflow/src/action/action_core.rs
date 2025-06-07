use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

use super::condition::ActionCondition;

/// Represents an action that controls flow between nodes in CosmoFlow workflows.
/// Actions can be simple labels, parameterized, conditional, or composite.
///
/// # Examples
///
/// ## Creating Simple Actions
/// ```
/// use cosmoflow::action::Action;
///
/// let action = Action::simple("next_node");
/// assert_eq!(action.name(), "next_node");
/// ```
///
/// ## Creating Parameterized Actions
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Simple string-based action (backward compatible)
    Simple(String),

    /// Action with parameters
    Parameterized {
        /// The name of the action.
        name: String,
        /// The parameters for the action.
        #[serde(default)]
        params: HashMap<String, Value>,
    },

    /// Conditional action that evaluates based on context
    Conditional {
        /// The condition to evaluate.
        condition: ActionCondition,
        /// The action to take if the condition is true.
        if_true: Box<Action>,
        /// The action to take if the condition is false.
        if_false: Box<Action>,
    },

    /// Multiple actions that can be taken (for parallel execution or choices)
    Multiple(Vec<Action>),

    /// Action with priority (higher numbers = higher priority)
    Prioritized {
        /// The action to prioritize.
        action: Box<Action>,
        /// The priority of the action.
        #[serde(default)]
        priority: i32,
    },

    /// Action with metadata
    WithMetadata {
        /// The action to add metadata to.
        action: Box<Action>,
        /// The metadata to add to the action.
        #[serde(default)]
        metadata: HashMap<String, Value>,
    },
}

impl Action {
    /// Create a simple action from a string
    ///
    /// Simple actions are the most common type, consisting of just a name/label
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

    /// Create a conditional action that chooses between two options
    ///
    /// Conditional actions evaluate a condition at runtime and return one of
    /// two possible actions based on the result. This enables dynamic routing
    /// based on workflow state.
    ///
    /// # Arguments
    ///
    /// * `condition` - Condition to evaluate
    /// * `if_true` - Action to return if condition is true
    /// * `if_false` - Action to return if condition is false
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::{Action, ActionCondition};
    /// use serde_json::json;
    ///
    /// let condition = ActionCondition::key_equals("status", json!("ready"));
    /// let action = Action::conditional(
    ///     condition,
    ///     Action::simple("proceed"),
    ///     Action::simple("wait")
    /// );
    /// ```
    pub fn conditional(condition: ActionCondition, if_true: Action, if_false: Action) -> Self {
        Action::Conditional {
            condition,
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }
    }

    /// Create a multiple action containing several sub-actions
    ///
    /// Multiple actions can represent parallel execution opportunities or
    /// provide multiple routing options for complex workflows.
    ///
    /// # Arguments
    ///
    /// * `actions` - Vector of actions to include
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let actions = vec![
    ///     Action::simple("save_data"),
    ///     Action::simple("send_notification"),
    ///     Action::simple("update_status"),
    /// ];
    /// let action = Action::multiple(actions);
    /// ```
    pub fn multiple(actions: Vec<Action>) -> Self {
        Action::Multiple(actions)
    }

    /// Create a prioritized action with execution priority
    ///
    /// Prioritized actions include a priority value that can be used by
    /// workflow engines to determine execution order when multiple actions
    /// are available.
    ///
    /// # Arguments
    ///
    /// * `action` - The underlying action
    /// * `priority` - Priority value (higher numbers = higher priority)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let high_priority = Action::with_priority(Action::simple("critical_task"), 100);
    /// let low_priority = Action::with_priority(Action::simple("cleanup"), 1);
    /// ```
    pub fn with_priority(action: Action, priority: i32) -> Self {
        Action::Prioritized {
            action: Box::new(action),
            priority,
        }
    }

    /// Add metadata to an action
    ///
    /// Metadata actions attach additional key-value information to actions
    /// without affecting their core behavior. This is useful for:
    /// - Adding debugging information
    /// - Storing execution metrics
    /// - Carrying configuration data
    /// - Adding audit trails
    ///
    /// # Arguments
    ///
    /// * `action` - The underlying action to wrap
    /// * `metadata` - Key-value metadata to attach
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut metadata = HashMap::new();
    /// metadata.insert("created_by".to_string(), json!("user123"));
    /// metadata.insert("timestamp".to_string(), json!("2024-01-01T12:00:00Z"));
    ///
    /// let action = Action::with_metadata(
    ///     Action::simple("process_data"),
    ///     metadata
    /// );
    /// ```
    pub fn with_metadata(action: Action, metadata: HashMap<String, Value>) -> Self {
        Action::WithMetadata {
            action: Box::new(action),
            metadata,
        }
    }

    /// Get the primary name/identifier of the action
    ///
    /// Returns the string identifier for this action. For nested actions,
    /// this returns the name of the innermost action.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let simple = Action::simple("test");
    /// assert_eq!(simple.name(), "test");
    ///
    /// let prioritized = Action::with_priority(Action::simple("urgent"), 10);
    /// assert_eq!(prioritized.name(), "urgent");
    /// ```
    pub fn name(&self) -> String {
        match self {
            Action::Simple(name) => name.clone(),
            Action::Parameterized { name, .. } => name.clone(),
            Action::Conditional { if_true, .. } => if_true.name(),
            Action::Multiple(actions) => {
                if let Some(first) = actions.first() {
                    first.name()
                } else {
                    "empty".to_string()
                }
            }
            Action::Prioritized { action, .. } => action.name(),
            Action::WithMetadata { action, .. } => action.name(),
        }
    }

    /// Get parameters if this is a parameterized action
    ///
    /// Returns a reference to the parameters map if this action carries parameters,
    /// or None if it doesn't. This method unwraps nested actions to find parameters.
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
            Action::WithMetadata { action, .. } => action.params(),
            Action::Prioritized { action, .. } => action.params(),
            _ => None,
        }
    }

    /// Get priority if this action has one
    ///
    /// Returns the priority value for prioritized actions, or None if the action
    /// doesn't have an explicit priority. Higher numbers indicate higher priority.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let simple = Action::simple("test");
    /// assert!(simple.priority().is_none());
    ///
    /// let prioritized = Action::with_priority(Action::simple("urgent"), 100);
    /// assert_eq!(prioritized.priority(), Some(100));
    /// ```
    pub fn priority(&self) -> Option<i32> {
        match self {
            Action::Prioritized { priority, .. } => Some(*priority),
            Action::WithMetadata { action, .. } => action.priority(),
            _ => None,
        }
    }

    /// Get metadata if this action has any
    ///
    /// Returns a reference to the metadata map if this action carries metadata,
    /// or None if it doesn't. Metadata is additional key-value information
    /// attached to actions for debugging, auditing, or configuration purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut metadata = HashMap::new();
    /// metadata.insert("source".to_string(), json!("test"));
    /// let action = Action::with_metadata(Action::simple("test"), metadata);
    ///
    /// assert!(action.metadata().is_some());
    /// assert_eq!(action.metadata().unwrap().get("source").unwrap(), &json!("test"));
    /// ```
    pub fn metadata(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Action::WithMetadata { metadata, .. } => Some(metadata),
            _ => None,
        }
    }

    /// Check if this is a simple action
    ///
    /// Returns true if this is a simple string-based action without any
    /// additional features like parameters, conditions, or metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let simple = Action::simple("test");
    /// assert!(simple.is_simple());
    ///
    /// let complex = Action::with_priority(Action::simple("test"), 10);
    /// assert!(!complex.is_simple());
    /// ```
    pub fn is_simple(&self) -> bool {
        matches!(self, Action::Simple(_))
    }

    /// Check if this action has parameters
    ///
    /// Returns true if this action carries parameters (key-value data),
    /// either directly or nested within wrapper actions.
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

    /// Check if this action is conditional
    ///
    /// Returns true if this action contains conditional logic that evaluates
    /// different actions based on runtime conditions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::{Action, ActionCondition};
    /// use serde_json::json;
    ///
    /// let simple = Action::simple("test");
    /// assert!(!simple.is_conditional());
    ///
    /// let condition = ActionCondition::key_equals("ready", json!(true));
    /// let conditional = Action::conditional(
    ///     condition,
    ///     Action::simple("proceed"),
    ///     Action::simple("wait")
    /// );
    /// assert!(conditional.is_conditional());
    /// ```
    pub fn is_conditional(&self) -> bool {
        matches!(self, Action::Conditional { .. })
    }

    /// Check if this action represents multiple actions
    ///
    /// Returns true if this action contains multiple sub-actions that can
    /// be executed in parallel or represent multiple routing choices.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::Action;
    ///
    /// let simple = Action::simple("test");
    /// assert!(!simple.is_multiple());
    ///
    /// let multiple = Action::multiple(vec![
    ///     Action::simple("step1"),
    ///     Action::simple("step2"),
    /// ]);
    /// assert!(multiple.is_multiple());
    /// ```
    pub fn is_multiple(&self) -> bool {
        matches!(self, Action::Multiple(_))
    }

    /// Flatten nested actions to a single level for easier processing
    ///
    /// This method recursively flattens nested action structures into a flat vector.
    /// It's particularly useful for:
    /// - Processing multiple actions in sequence
    /// - Analyzing all actions in a workflow without dealing with nested structures
    /// - Converting hierarchical action trees to linear representations
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmoflow::action::Action;
    ///
    /// let action1 = Action::simple("step1");
    /// let action2 = Action::simple("step2");
    /// let multiple = Action::multiple(vec![action1, action2]);
    ///
    /// let flattened = multiple.flatten();
    /// assert_eq!(flattened.len(), 2);
    /// ```
    pub fn flatten(&self) -> Vec<&Action> {
        match self {
            Action::Multiple(actions) => actions.iter().flat_map(|a| a.flatten()).collect(),
            Action::WithMetadata { action, .. } => action.flatten(),
            Action::Prioritized { action, .. } => action.flatten(),
            _ => vec![self],
        }
    }

    /// Get all actions sorted by priority (highest first)
    ///
    /// This method extracts all actions along with their priorities and returns them
    /// sorted in descending order of priority (highest priority first). It's useful for:
    /// - Determining execution order when multiple actions are available
    /// - Implementing priority-based scheduling systems
    /// - Analyzing action hierarchies based on importance
    ///
    /// Actions without explicit priority are assigned a default priority of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmoflow::action::Action;
    ///
    /// let low_priority = Action::with_priority(Action::simple("background"), 1);
    /// let high_priority = Action::with_priority(Action::simple("critical"), 10);
    /// let multiple = Action::multiple(vec![low_priority, high_priority]);
    ///
    /// let prioritized = multiple.prioritized_actions();
    /// assert_eq!(prioritized[0].1, 10); // High priority first
    /// assert_eq!(prioritized[1].1, 1);  // Low priority second
    /// ```
    pub fn prioritized_actions(&self) -> Vec<(&Action, i32)> {
        let mut actions = Vec::new();
        self.collect_with_priority(&mut actions);
        actions.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by priority desc
        actions
    }

    /// Helper method to collect actions with their priorities
    ///
    /// This is a private recursive helper that traverses the action tree and collects
    /// each action along with its associated priority. It handles:
    /// - Extracting priorities from Prioritized actions
    /// - Recursively processing Multiple and nested actions  
    /// - Assigning default priority (0) to actions without explicit priority
    /// - Maintaining proper priority inheritance in nested structures
    fn collect_with_priority<'a>(&'a self, collector: &mut Vec<(&'a Action, i32)>) {
        match self {
            Action::Prioritized { action, priority } => {
                action.collect_with_priority(collector);
                if let Some(last) = collector.last_mut() {
                    last.1 = *priority;
                }
            }
            Action::Multiple(actions) => {
                for action in actions {
                    action.collect_with_priority(collector);
                }
            }
            Action::WithMetadata { action, .. } => {
                action.collect_with_priority(collector);
            }
            _ => {
                collector.push((self, 0));
            }
        }
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
            Action::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                write!(f, "if {condition} then {if_true} else {if_false}")
            }
            Action::Multiple(actions) => {
                write!(
                    f,
                    "[{}]",
                    actions
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Action::Prioritized { action, priority } => {
                write!(f, "{action}@{priority}")
            }
            Action::WithMetadata { action, .. } => {
                write!(f, "{action}")
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
