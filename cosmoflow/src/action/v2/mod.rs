//! Action API.
//!
//! This module provides the minimal action model for state-machine transitions:
//! an action has a routing identity and optional parameters.

mod action;
mod name;
mod params;

pub use action::Action;
pub use name::ActionName;
pub use params::ActionParams;

#[cfg(test)]
mod tests {
    use super::{Action, ActionName, ActionParams};
    use serde_json::{Value, json};

    #[test]
    fn action_name_supports_creation_and_display() {
        let from_new = ActionName::new("next");
        let from_str = ActionName::from("next");
        let from_string = ActionName::from("next".to_string());

        assert_eq!(from_new.as_str(), "next");
        assert_eq!(from_new.to_string(), "next");
        assert_eq!(from_new, from_str);
        assert_eq!(from_new, from_string);
    }

    #[test]
    fn new_action_has_empty_params() {
        let action = Action::new("next");

        assert_eq!(action.name().as_str(), "next");
        assert_eq!(action.as_str(), "next");
        assert_eq!(action.params(), &ActionParams::new());
        assert_eq!(action.get_param("missing"), None);
        assert!(!action.has_param("missing"));
        assert_eq!(action.param_count(), 0);
        assert!(!action.has_params());
    }

    #[test]
    fn action_with_param_stores_one_value() {
        let action = Action::with_param("retry", "count", json!(3));

        assert_eq!(action.name().as_str(), "retry");
        assert_eq!(action.get_param("count"), Some(&json!(3)));
        assert!(action.has_param("count"));
        assert_eq!(action.param_count(), 1);
        assert!(action.has_params());
    }

    #[test]
    fn action_with_params_stores_all_values() {
        let mut params = ActionParams::new();
        params.insert("path".to_string(), json!("tools"));
        params.insert("attempt".to_string(), json!(2));

        let action = Action::with_params("dispatch", params.clone());

        assert_eq!(action.name().as_str(), "dispatch");
        assert_eq!(action.params(), &params);
        assert_eq!(action.get_param("path"), Some(&json!("tools")));
        assert_eq!(action.get_param("attempt"), Some(&json!(2)));
        assert_eq!(action.param_count(), 2);
    }

    #[test]
    fn params_always_return_a_map_reference() {
        let empty = Action::with_params("empty", ActionParams::new());

        assert_eq!(empty.params(), &ActionParams::new());
        assert_eq!(empty.param_count(), 0);
        assert!(!empty.has_params());
    }

    #[test]
    fn action_display_only_uses_name() {
        let action = Action::with_param("route", "detail", json!("ignored by display"));

        assert_eq!(action.to_string(), "route");
    }

    #[test]
    fn action_serialization_roundtrips_name_and_params() {
        let action = Action::with_param("route", "score", json!(42));

        let serialized = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, action);
        assert_eq!(deserialized.get_param("score"), Some(&json!(42)));
    }

    #[test]
    fn action_serialization_skips_empty_params() {
        let action = Action::new("route");

        let serialized: Value = serde_json::to_value(&action).unwrap();

        assert_eq!(serialized, json!({ "name": "route" }));
    }

    #[test]
    fn missing_params_deserialize_as_empty_map() {
        let action: Action = serde_json::from_value(json!({ "name": "route" })).unwrap();

        assert_eq!(action.name().as_str(), "route");
        assert_eq!(action.params(), &ActionParams::new());
        assert_eq!(action.param_count(), 0);
        assert!(!action.has_params());
    }
}
