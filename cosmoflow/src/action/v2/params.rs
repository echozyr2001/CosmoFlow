use serde_json::Value;
use std::collections::HashMap;

/// Parameters carried by a v2 action.
pub type ActionParams = HashMap<String, Value>;
