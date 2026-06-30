use super::{ActionName, ActionParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

fn no_params(params: &ActionParams) -> bool {
    params.is_empty()
}

/// State transition signal returned by v2 nodes and flows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    name: ActionName,
    #[serde(default, skip_serializing_if = "no_params")]
    params: ActionParams,
}

impl Action {
    /// Create an action without parameters.
    pub fn new(name: impl Into<ActionName>) -> Self {
        Self {
            name: name.into(),
            params: ActionParams::new(),
        }
    }

    /// Create an action with a parameter map.
    pub fn with_params(name: impl Into<ActionName>, params: ActionParams) -> Self {
        Self {
            name: name.into(),
            params,
        }
    }

    /// Create an action with one parameter.
    pub fn with_param(name: impl Into<ActionName>, key: impl Into<String>, value: Value) -> Self {
        let mut params = ActionParams::new();
        params.insert(key.into(), value);
        Self::with_params(name, params)
    }

    /// Return the routing identity of this action.
    pub fn name(&self) -> &ActionName {
        &self.name
    }

    /// Return the routing identity as a string slice.
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }

    /// Return all action parameters.
    pub fn params(&self) -> &ActionParams {
        &self.params
    }

    /// Return one action parameter by key.
    pub fn get_param(&self, key: &str) -> Option<&Value> {
        self.params.get(key)
    }

    /// Return whether this action has a parameter with the given key.
    pub fn has_param(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Return the number of parameters carried by this action.
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    /// Return whether this action carries any parameters.
    pub fn has_params(&self) -> bool {
        !self.params.is_empty()
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

impl From<&str> for Action {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for Action {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}
