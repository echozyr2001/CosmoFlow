use crate::Action;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_simple_action_creation() {
    let simple = Action::simple("test");
    assert_eq!(simple.name(), "test");
    assert!(simple.is_simple());
    assert!(!simple.is_parameterized());
    assert_eq!(simple.param_count(), 0);
}

#[test]
fn test_parameterized_action_creation() {
    let mut params = HashMap::new();
    params.insert("key".to_string(), json!("value"));
    params.insert("count".to_string(), json!(42));

    let parameterized = Action::with_params("test", params);
    assert_eq!(parameterized.name(), "test");
    assert!(!parameterized.is_simple());
    assert!(parameterized.is_parameterized());
    assert_eq!(parameterized.param_count(), 2);

    let retrieved_params = parameterized.params().unwrap();
    assert_eq!(retrieved_params.get("key").unwrap(), &json!("value"));
    assert_eq!(retrieved_params.get("count").unwrap(), &json!(42));
}

#[test]
fn test_single_param_convenience_method() {
    let action = Action::with_param("retry", "count", json!(3));
    assert_eq!(action.name(), "retry");
    assert!(action.is_parameterized());
    assert!(!action.is_simple());
    assert_eq!(action.param_count(), 1);

    // Test parameter access
    assert_eq!(action.get_param("count"), Some(&json!(3)));
    assert_eq!(action.get_param("missing"), None);
    assert!(action.has_param("count"));
    assert!(!action.has_param("missing"));
}

#[test]
fn test_parameter_access_methods() {
    // Test with parameterized action
    let action = Action::with_param("test", "key", json!("value"));
    assert_eq!(action.get_param("key"), Some(&json!("value")));
    assert_eq!(action.get_param("missing"), None);
    assert!(action.has_param("key"));
    assert!(!action.has_param("missing"));

    // Test with simple action
    let simple = Action::simple("test");
    assert_eq!(simple.get_param("any"), None);
    assert!(!simple.has_param("any"));
}

#[test]
fn test_action_display() {
    // Test simple action display
    let simple = Action::simple("test");
    assert_eq!(format!("{}", simple), "test");

    // Test parameterized action display
    let action = Action::with_param("test", "key", json!("value"));
    let display = format!("{}", action);
    assert!(display.contains("test("));
    assert!(display.contains("key="));
}

#[test]
fn test_action_conversions() {
    // Test From<&str>
    let action1: Action = "test".into();
    assert_eq!(action1, Action::simple("test"));

    // Test From<String>
    let action2: Action = String::from("test").into();
    assert_eq!(action2, Action::simple("test"));

    // Test Into<String>
    let action = Action::simple("test");
    let name: String = action.into();
    assert_eq!(name, "test");
}

#[test]
fn test_action_default() {
    let action = Action::default();
    assert_eq!(action, Action::simple("default"));
}

#[test]
fn test_action_serialization() {
    // Test simple action serialization
    let simple = Action::simple("test");
    let serialized = serde_json::to_string(&simple).unwrap();
    let deserialized: Action = serde_json::from_str(&serialized).unwrap();
    assert_eq!(simple, deserialized);

    // Test parameterized action serialization
    let action = Action::with_param("test", "key", json!("value"));
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: Action = serde_json::from_str(&serialized).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn test_empty_params() {
    let action = Action::with_params("test", HashMap::new());
    assert!(action.is_parameterized());
    assert!(action.params().unwrap().is_empty());
    assert_eq!(action.param_count(), 0);
}

#[test]
fn test_action_clone() {
    let original = Action::simple("test");
    let cloned = original.clone();
    assert_eq!(original, cloned);

    let original_param = Action::with_param("test", "key", json!("value"));
    let cloned_param = original_param.clone();
    assert_eq!(original_param, cloned_param);
}

#[test]
fn test_naming_consistency() {
    // Test that is_* methods are consistent
    let simple = Action::simple("test");
    assert!(simple.is_simple());
    assert!(!simple.is_parameterized());

    let parameterized = Action::with_param("test", "key", json!("value"));
    assert!(!parameterized.is_simple());
    assert!(parameterized.is_parameterized());
    assert!(parameterized.has_param("key"));
    assert!(!parameterized.has_param("missing"));
}

#[test]
fn test_param_count_accuracy() {
    let simple = Action::simple("test");
    assert_eq!(simple.param_count(), 0);

    let single = Action::with_param("test", "key", json!("value"));
    assert_eq!(single.param_count(), 1);

    let mut params = HashMap::new();
    params.insert("a".to_string(), json!(1));
    params.insert("b".to_string(), json!(2));
    params.insert("c".to_string(), json!(3));
    let multiple = Action::with_params("test", params);
    assert_eq!(multiple.param_count(), 3);

    let empty_params = Action::with_params("test", HashMap::new());
    assert_eq!(empty_params.param_count(), 0);
}
