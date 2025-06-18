use crate::Action;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_simple_action_creation() {
    let simple = Action::simple("test");
    assert_eq!(simple.name(), "test");
    assert!(simple.is_simple());
    assert!(!simple.has_params());
}

#[test]
fn test_parameterized_action_creation() {
    let mut params = HashMap::new();
    params.insert("key".to_string(), json!("value"));
    params.insert("count".to_string(), json!(42));

    let parameterized = Action::with_params("test", params);
    assert_eq!(parameterized.name(), "test");
    assert!(!parameterized.is_simple());
    assert!(parameterized.has_params());

    let retrieved_params = parameterized.params().unwrap();
    assert_eq!(retrieved_params.get("key").unwrap(), &json!("value"));
    assert_eq!(retrieved_params.get("count").unwrap(), &json!(42));
}

#[test]
fn test_action_display() {
    // Test simple action display
    let simple = Action::simple("test");
    assert_eq!(format!("{}", simple), "test");

    // Test parameterized action display
    let mut params = HashMap::new();
    params.insert("key".to_string(), json!("value"));
    params.insert("count".to_string(), json!(42));
    let parameterized = Action::with_params("test", params);
    let display = format!("{}", parameterized);
    assert!(display.contains("test("));
    assert!(display.contains("key="));
    assert!(display.contains("count="));
}

#[test]
fn test_action_from_string() {
    let action1: Action = "test".into();
    assert_eq!(action1, Action::simple("test"));

    let action2: Action = String::from("test").into();
    assert_eq!(action2, Action::simple("test"));
}

#[test]
fn test_action_to_string() {
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
    let mut params = HashMap::new();
    params.insert("key".to_string(), json!("value"));
    let parameterized = Action::with_params("test", params);
    let serialized = serde_json::to_string(&parameterized).unwrap();
    let deserialized: Action = serde_json::from_str(&serialized).unwrap();
    assert_eq!(parameterized, deserialized);
}

#[test]
fn test_empty_params() {
    let action = Action::with_params("test", HashMap::new());
    assert!(action.has_params());
    assert!(action.params().unwrap().is_empty());
}

#[test]
fn test_action_clone() {
    let original = Action::simple("test");
    let cloned = original.clone();
    assert_eq!(original, cloned);

    let mut params = HashMap::new();
    params.insert("key".to_string(), json!("value"));
    let original_param = Action::with_params("test", params);
    let cloned_param = original_param.clone();
    assert_eq!(original_param, cloned_param);
}
