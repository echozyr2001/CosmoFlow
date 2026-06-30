use serde::{Deserialize, Serialize};
use std::fmt;

/// Name used as the routing identity for a v2 action.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionName(String);

impl ActionName {
    /// Create an action name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Return the action name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ActionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for ActionName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for ActionName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}
