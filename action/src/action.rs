use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

use crate::condition::ActionCondition;

/// Represents an action that controls flow between nodes in CosmoFlow workflows.
/// Actions can be simple labels, parameterized, conditional, or composite.
///
/// # Examples
///
/// ## Creating Simple Actions
/// ```
/// use action::Action;
///
/// let action = Action::simple("next_node");
/// assert_eq!(action.name(), "next_node");
/// ```
///
/// ## Creating Parameterized Actions
/// ```
/// use action::Action;
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
        name: String,
        #[serde(default)]
        params: HashMap<String, Value>,
    },

    /// Conditional action that evaluates based on context
    Conditional {
        condition: ActionCondition,
        if_true: Box<Action>,
        if_false: Box<Action>,
    },

    /// Multiple actions that can be taken (for parallel execution or choices)
    Multiple(Vec<Action>),

    /// Action with priority (higher numbers = higher priority)
    Prioritized {
        action: Box<Action>,
        #[serde(default)]
        priority: i32,
    },

    /// Action with metadata
    WithMetadata {
        action: Box<Action>,
        #[serde(default)]
        metadata: HashMap<String, Value>,
    },
}

impl Action {
    /// Create a simple action from a string
    pub fn simple<S: Into<String>>(name: S) -> Self {
        Action::Simple(name.into())
    }

    /// Create a parameterized action
    pub fn with_params<S: Into<String>>(name: S, params: HashMap<String, Value>) -> Self {
        Action::Parameterized {
            name: name.into(),
            params,
        }
    }

    /// Create a conditional action
    pub fn conditional(condition: ActionCondition, if_true: Action, if_false: Action) -> Self {
        Action::Conditional {
            condition,
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }
    }

    /// Create a multiple action
    pub fn multiple(actions: Vec<Action>) -> Self {
        Action::Multiple(actions)
    }

    /// Create a prioritized action
    pub fn with_priority(action: Action, priority: i32) -> Self {
        Action::Prioritized {
            action: Box::new(action),
            priority,
        }
    }

    /// Add metadata to an action
    pub fn with_metadata(action: Action, metadata: HashMap<String, Value>) -> Self {
        Action::WithMetadata {
            action: Box::new(action),
            metadata,
        }
    }

    /// Get the primary name/identifier of the action
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
    pub fn params(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Action::Parameterized { params, .. } => Some(params),
            Action::WithMetadata { action, .. } => action.params(),
            Action::Prioritized { action, .. } => action.params(),
            _ => None,
        }
    }

    /// Get priority if this action has one
    pub fn priority(&self) -> Option<i32> {
        match self {
            Action::Prioritized { priority, .. } => Some(*priority),
            Action::WithMetadata { action, .. } => action.priority(),
            _ => None,
        }
    }

    /// Get metadata if this action has any
    pub fn metadata(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Action::WithMetadata { metadata, .. } => Some(metadata),
            _ => None,
        }
    }

    /// Check if this is a simple action
    pub fn is_simple(&self) -> bool {
        matches!(self, Action::Simple(_))
    }

    /// Check if this action has parameters
    pub fn has_params(&self) -> bool {
        self.params().is_some()
    }

    /// Check if this action is conditional
    pub fn is_conditional(&self) -> bool {
        matches!(self, Action::Conditional { .. })
    }

    /// Check if this action represents multiple actions
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
    /// use action::Action;
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
    /// use action::Action;
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
