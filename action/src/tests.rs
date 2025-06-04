use crate::{Action, ActionCondition, ComparisonOperator};
use std::collections::HashMap;
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_action_creation() {
        let simple = Action::simple("test");
        assert_eq!(simple.name(), "test");
        assert!(simple.is_simple());

        let mut params = HashMap::new();
        params.insert("key".to_string(), json!("value"));
        let parameterized = Action::with_params("test", params);
        assert!(parameterized.has_params());
    }

    #[test]
    fn test_action_condition_evaluation() {
        let mut context = HashMap::new();
        context.insert("key1".to_string(), json!("value1"));
        context.insert("number".to_string(), json!(42));

        let cond1 = ActionCondition::key_exists("key1");
        assert!(cond1.evaluate(&context));

        let cond2 = ActionCondition::key_exists("nonexistent");
        assert!(!cond2.evaluate(&context));

        let cond3 = ActionCondition::key_equals("key1", json!("value1"));
        assert!(cond3.evaluate(&context));

        let cond4 =
            ActionCondition::numeric_compare("number", ComparisonOperator::GreaterThan, 40.0);
        assert!(cond4.evaluate(&context));

        let and_cond = ActionCondition::and(vec![cond1, cond3]);
        assert!(and_cond.evaluate(&context));
    }

    #[test]
    fn test_expression_evaluation() {
        let mut context = HashMap::new();
        context.insert("status".to_string(), json!("active"));
        context.insert("count".to_string(), json!(5));
        context.insert("enabled".to_string(), json!(true));
        context.insert("name".to_string(), json!("test"));

        // Test boolean literals
        let true_expr = ActionCondition::expression("true");
        assert!(true_expr.evaluate(&context));

        let false_expr = ActionCondition::expression("false");
        assert!(!false_expr.evaluate(&context));

        // Test variable substitution
        let var_expr = ActionCondition::expression("${enabled}");
        assert!(var_expr.evaluate(&context));

        let var_string_expr = ActionCondition::expression("${name}");
        assert!(var_string_expr.evaluate(&context)); // Non-empty string is truthy

        // Test string comparison
        let string_eq_expr = ActionCondition::expression("${status} == active");
        assert!(string_eq_expr.evaluate(&context));

        let string_ne_expr = ActionCondition::expression("${status} != inactive");
        assert!(string_ne_expr.evaluate(&context));

        // Test numeric comparisons
        let gt_expr = ActionCondition::expression("${count} > 3");
        assert!(gt_expr.evaluate(&context));

        let gte_expr = ActionCondition::expression("${count} >= 5");
        assert!(gte_expr.evaluate(&context));

        let lt_expr = ActionCondition::expression("${count} < 10");
        assert!(lt_expr.evaluate(&context));

        let lte_expr = ActionCondition::expression("${count} <= 5");
        assert!(lte_expr.evaluate(&context));

        let eq_expr = ActionCondition::expression("${count} == 5");
        assert!(eq_expr.evaluate(&context));

        // Test with quoted values
        let quoted_expr = ActionCondition::expression("${status} == \"active\"");
        assert!(quoted_expr.evaluate(&context));

        // Test non-existent variable
        let missing_expr = ActionCondition::expression("${missing}");
        assert!(!missing_expr.evaluate(&context));
    }

    #[test]
    fn test_action_flattening() {
        let action1 = Action::simple("action1");
        let action2 = Action::simple("action2");
        let multiple = Action::multiple(vec![action1, action2]);

        let flattened = multiple.flatten();
        assert_eq!(flattened.len(), 2);
    }

    #[test]
    fn test_action_display() {
        let action = Action::simple("test");
        assert_eq!(format!("{}", action), "test");

        let condition = ActionCondition::key_exists("key");
        assert_eq!(format!("{}", condition), "exists(key)");
    }

    #[test]
    fn test_prioritized_actions() {
        let action1 = Action::with_priority(Action::simple("low"), 1);
        let action2 = Action::with_priority(Action::simple("high"), 10);
        let multiple = Action::multiple(vec![action1, action2]);

        let prioritized = multiple.prioritized_actions();
        assert_eq!(prioritized.len(), 2);
        // Higher priority should come first
        assert_eq!(prioritized[0].1, 10);
        assert_eq!(prioritized[1].1, 1);
    }
}
