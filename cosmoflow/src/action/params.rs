use serde_json::Value;
use std::collections::HashMap;

/// Parameters carried by an action.
pub type ActionParams = HashMap<String, Value>;
