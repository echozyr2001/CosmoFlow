use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// Represents a condition for conditional actions
///
/// # Examples
///
/// ## Basic Conditions
/// ```
/// use cosmoflow::action::ActionCondition;
/// use serde_json::json;
///
/// // Check if a key exists
/// let exists_cond = ActionCondition::key_exists("user_id");
///
/// // Check if a key has a specific value
/// let equals_cond = ActionCondition::key_equals("status", json!("active"));
/// ```
///
/// ## Logical Operations
/// ```
/// use cosmoflow::action::ActionCondition;
/// use serde_json::json;
///
/// let cond1 = ActionCondition::key_exists("user_id");
/// let cond2 = ActionCondition::key_equals("status", json!("active"));
///
/// // Combine conditions
/// let and_cond = ActionCondition::and(vec![cond1.clone(), cond2.clone()]);
/// let or_cond = ActionCondition::or(vec![cond1, cond2]);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionCondition {
    /// Always true
    Always,

    /// Always false
    Never,

    /// Check if a key exists in the shared store
    KeyExists(String),

    /// Check if a key has a specific value
    KeyEquals(String, Value),

    /// Compare a numeric value
    NumericCompare {
        /// The key to compare.
        key: String,
        /// The comparison operator.
        operator: ComparisonOperator,
        /// The value to compare against.
        value: f64,
    },

    /// Custom condition with a string expression
    Expression(String),

    /// Logical AND of multiple conditions
    And(Vec<ActionCondition>),

    /// Logical OR of multiple conditions
    Or(Vec<ActionCondition>),

    /// Logical NOT of a condition
    Not(Box<ActionCondition>),
}

/// Comparison operators for numeric conditions
///
/// These operators define how numeric values should be compared in
/// `NumericCompare` conditions. All comparisons are performed as
/// floating-point operations for consistency.
///
/// # Examples
///
/// ```rust
/// use cosmoflow::action::{ActionCondition, ComparisonOperator};
///
/// // Greater than comparison
/// let condition = ActionCondition::numeric_compare(
///     "score",
///     ComparisonOperator::GreaterThan,
///     90.0
/// );
///
/// // Equal comparison (with epsilon tolerance)
/// let equal_condition = ActionCondition::numeric_compare(
///     "temperature",
///     ComparisonOperator::Equal,
///     98.6
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Equality comparison (uses epsilon tolerance for floating-point)
    Equal,
    /// Inequality comparison
    NotEqual,
    /// Greater than comparison
    GreaterThan,
    /// Greater than or equal comparison
    GreaterThanOrEqual,
    /// Less than comparison
    LessThan,
    /// Less than or equal comparison
    LessThanOrEqual,
}

impl ActionCondition {
    /// Create a condition that checks if a key exists
    ///
    /// This condition evaluates to true if the specified key exists in the
    /// evaluation context, regardless of its value (including null).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for existence
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::ActionCondition;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let condition = ActionCondition::key_exists("user_id");
    ///
    /// let mut context = HashMap::new();
    /// context.insert("user_id".to_string(), json!(123));
    /// assert!(condition.evaluate(&context));
    ///
    /// context.clear();
    /// assert!(!condition.evaluate(&context));
    /// ```
    pub fn key_exists<S: Into<String>>(key: S) -> Self {
        ActionCondition::KeyExists(key.into())
    }

    /// Create a condition that checks if a key equals a value
    ///
    /// This condition evaluates to true if the specified key exists in the
    /// context and its value exactly matches the provided value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check
    /// * `value` - The expected value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::ActionCondition;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let condition = ActionCondition::key_equals("status", json!("active"));
    ///
    /// let mut context = HashMap::new();
    /// context.insert("status".to_string(), json!("active"));
    /// assert!(condition.evaluate(&context));
    ///
    /// context.insert("status".to_string(), json!("inactive"));
    /// assert!(!condition.evaluate(&context));
    /// ```
    pub fn key_equals<S: Into<String>>(key: S, value: Value) -> Self {
        ActionCondition::KeyEquals(key.into(), value)
    }

    /// Create a numeric comparison condition
    ///
    /// This condition extracts a numeric value from the context and compares
    /// it against a provided value using the specified operator. Non-numeric
    /// values are treated as comparison failures.
    ///
    /// # Arguments
    ///
    /// * `key` - The key containing the numeric value to compare
    /// * `operator` - The comparison operator to use
    /// * `value` - The value to compare against
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::{ActionCondition, ComparisonOperator};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let condition = ActionCondition::numeric_compare(
    ///     "score",
    ///     ComparisonOperator::GreaterThan,
    ///     85.0
    /// );
    ///
    /// let mut context = HashMap::new();
    /// context.insert("score".to_string(), json!(92.5));
    /// assert!(condition.evaluate(&context));
    ///
    /// context.insert("score".to_string(), json!(75.0));
    /// assert!(!condition.evaluate(&context));
    /// ```
    pub fn numeric_compare<S: Into<String>>(
        key: S,
        operator: ComparisonOperator,
        value: f64,
    ) -> Self {
        ActionCondition::NumericCompare {
            key: key.into(),
            operator,
            value,
        }
    }

    /// Create an expression-based condition
    ///
    /// Expression conditions support simple string-based logic with variable
    /// substitution and basic comparisons. This provides flexibility for
    /// complex conditional logic that doesn't fit other condition types.
    ///
    /// Supported patterns:
    /// - Variable substitution: `"${variable_name}"`
    /// - Comparisons: `"${var} == value"`, `"${var} > 10"`
    /// - Boolean literals: `"true"`, `"false"`
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression string to evaluate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::ActionCondition;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// // Variable substitution
    /// let bool_cond = ActionCondition::expression("${is_ready}");
    ///
    /// // Comparison expression
    /// let comp_cond = ActionCondition::expression("${count} > 5");
    ///
    /// let mut context = HashMap::new();
    /// context.insert("is_ready".to_string(), json!(true));
    /// context.insert("count".to_string(), json!(10));
    ///
    /// assert!(bool_cond.evaluate(&context));
    /// assert!(comp_cond.evaluate(&context));
    /// ```
    pub fn expression<S: Into<String>>(expr: S) -> Self {
        ActionCondition::Expression(expr.into())
    }

    /// Create an AND condition
    ///
    /// AND conditions evaluate to true only if ALL of the contained conditions
    /// evaluate to true. Empty condition lists evaluate to true.
    ///
    /// # Arguments
    ///
    /// * `conditions` - Vector of conditions that must all be true
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::ActionCondition;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let cond1 = ActionCondition::key_exists("user_id");
    /// let cond2 = ActionCondition::key_equals("status", json!("active"));
    /// let and_cond = ActionCondition::and(vec![cond1, cond2]);
    ///
    /// let mut context = HashMap::new();
    /// context.insert("user_id".to_string(), json!(123));
    /// context.insert("status".to_string(), json!("active"));
    /// assert!(and_cond.evaluate(&context));
    ///
    /// context.remove("user_id");
    /// assert!(!and_cond.evaluate(&context)); // Missing user_id
    /// ```
    pub fn and(conditions: Vec<ActionCondition>) -> Self {
        ActionCondition::And(conditions)
    }

    /// Create an OR condition
    ///
    /// OR conditions evaluate to true if ANY of the contained conditions
    /// evaluate to true. Empty condition lists evaluate to false.
    ///
    /// # Arguments
    ///
    /// * `conditions` - Vector of conditions where at least one must be true
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::ActionCondition;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let cond1 = ActionCondition::key_equals("role", json!("admin"));
    /// let cond2 = ActionCondition::key_equals("role", json!("moderator"));
    /// let or_cond = ActionCondition::or(vec![cond1, cond2]);
    ///
    /// let mut context = HashMap::new();
    /// context.insert("role".to_string(), json!("admin"));
    /// assert!(or_cond.evaluate(&context));
    ///
    /// context.insert("role".to_string(), json!("user"));
    /// assert!(!or_cond.evaluate(&context)); // Neither admin nor moderator
    /// ```
    pub fn or(conditions: Vec<ActionCondition>) -> Self {
        ActionCondition::Or(conditions)
    }

    /// Create a NOT condition from another condition
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmoflow::action::ActionCondition;
    ///
    /// let condition = ActionCondition::key_equals("status", serde_json::json!("ready"));
    ///
    /// // Using static method
    /// let negated1 = ActionCondition::negate(condition.clone());
    /// ```
    pub fn negate(condition: ActionCondition) -> Self {
        ActionCondition::Not(Box::new(condition))
    }

    /// Evaluate a condition against a context
    ///
    /// This method evaluates the condition using the provided context map.
    /// The context typically contains data from the workflow's shared store
    /// or node execution results.
    ///
    /// # Arguments
    ///
    /// * `context` - Map containing key-value data for condition evaluation
    ///
    /// # Returns
    ///
    /// Boolean result of the condition evaluation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cosmoflow::action::ActionCondition;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let condition = ActionCondition::key_equals("status", json!("ready"));
    ///
    /// let mut context = HashMap::new();
    /// context.insert("status".to_string(), json!("ready"));
    /// assert!(condition.evaluate(&context));
    ///
    /// context.insert("status".to_string(), json!("waiting"));
    /// assert!(!condition.evaluate(&context));
    /// ```
    pub fn evaluate(&self, context: &HashMap<String, Value>) -> bool {
        match self {
            ActionCondition::Always => true,
            ActionCondition::Never => false,
            ActionCondition::KeyExists(key) => context.contains_key(key),
            ActionCondition::KeyEquals(key, value) => context.get(key) == Some(value),
            ActionCondition::NumericCompare {
                key,
                operator,
                value,
            } => {
                if let Some(context_value) = context.get(key) {
                    if let Some(num) = context_value.as_f64() {
                        match operator {
                            ComparisonOperator::Equal => (num - value).abs() < f64::EPSILON,
                            ComparisonOperator::NotEqual => (num - value).abs() >= f64::EPSILON,
                            ComparisonOperator::GreaterThan => num > *value,
                            ComparisonOperator::GreaterThanOrEqual => num >= *value,
                            ComparisonOperator::LessThan => num < *value,
                            ComparisonOperator::LessThanOrEqual => num <= *value,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            ActionCondition::Expression(expr) => {
                // Basic expression evaluation for simple variable substitution and comparisons
                // Supports patterns like: "${variable}", "${var1} == value", "${var1} > 10", etc.
                self.evaluate_expression(expr, context)
            }
            ActionCondition::And(conditions) => conditions.iter().all(|c| c.evaluate(context)),
            ActionCondition::Or(conditions) => conditions.iter().any(|c| c.evaluate(context)),
            ActionCondition::Not(condition) => !condition.evaluate(context),
        }
    }

    /// Evaluate a simple expression with variable substitution and basic comparisons
    ///
    /// This method provides basic expression evaluation supporting:
    /// - Variable substitution: "${variable_name}"
    /// - Simple comparisons: "${var} == value", "${var} > 10", etc.
    /// - Boolean values: "true", "false"
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmoflow::action::ActionCondition;
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// let mut context = HashMap::new();
    /// context.insert("status".to_string(), json!("active"));
    ///
    /// let condition = ActionCondition::expression("${status} == active");
    /// assert!(condition.evaluate(&context));
    /// ```
    fn evaluate_expression(&self, expr: &str, context: &HashMap<String, Value>) -> bool {
        let expr = expr.trim();

        // Handle boolean literals
        if expr == "true" {
            return true;
        }
        if expr == "false" {
            return false;
        }

        // Handle simple variable substitution: ${variable}
        if expr.starts_with("${") && expr.ends_with("}") {
            let var_name = &expr[2..expr.len() - 1];
            if let Some(value) = context.get(var_name) {
                return match value {
                    Value::Bool(b) => *b,
                    Value::String(s) => !s.is_empty(),
                    Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    Value::Null => false,
                    _ => true,
                };
            }
            return false;
        }

        // Handle simple comparisons: ${var} operator value
        if let Some(eq_pos) = expr.find(" == ") {
            let left = expr[..eq_pos].trim();
            let right = expr[eq_pos + 4..].trim();
            return self.compare_values(left, right, context, |a, b| a == b);
        }

        if let Some(ne_pos) = expr.find(" != ") {
            let left = expr[..ne_pos].trim();
            let right = expr[ne_pos + 4..].trim();
            return self.compare_values(left, right, context, |a, b| a != b);
        }

        if let Some(gte_pos) = expr.find(" >= ") {
            let left = expr[..gte_pos].trim();
            let right = expr[gte_pos + 4..].trim();
            return self.compare_numeric(left, right, context, |a, b| a >= b);
        }

        if let Some(lte_pos) = expr.find(" <= ") {
            let left = expr[..lte_pos].trim();
            let right = expr[lte_pos + 4..].trim();
            return self.compare_numeric(left, right, context, |a, b| a <= b);
        }

        if let Some(gt_pos) = expr.find(" > ") {
            let left = expr[..gt_pos].trim();
            let right = expr[gt_pos + 3..].trim();
            return self.compare_numeric(left, right, context, |a, b| a > b);
        }

        if let Some(lt_pos) = expr.find(" < ") {
            let left = expr[..lt_pos].trim();
            let right = expr[lt_pos + 3..].trim();
            return self.compare_numeric(left, right, context, |a, b| a < b);
        }

        // Default: treat as variable name and check for existence/truthiness
        if let Some(value) = context.get(expr) {
            match value {
                Value::Bool(b) => *b,
                Value::String(s) => !s.is_empty(),
                Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                Value::Null => false,
                _ => true,
            }
        } else {
            false
        }
    }

    /// Helper method to compare values (handles variable substitution)
    fn compare_values<F>(
        &self,
        left: &str,
        right: &str,
        context: &HashMap<String, Value>,
        op: F,
    ) -> bool
    where
        F: Fn(&Value, &Value) -> bool,
    {
        let left_val = self.resolve_value(left, context);
        let right_val = self.resolve_value(right, context);
        op(&left_val, &right_val)
    }

    /// Helper method to compare numeric values
    fn compare_numeric<F>(
        &self,
        left: &str,
        right: &str,
        context: &HashMap<String, Value>,
        op: F,
    ) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let left_num = self.resolve_numeric(left, context);
        let right_num = self.resolve_numeric(right, context);

        match (left_num, right_num) {
            (Some(a), Some(b)) => op(a, b),
            _ => false,
        }
    }

    /// Resolve a string to a Value (handles variable substitution)
    fn resolve_value(&self, s: &str, context: &HashMap<String, Value>) -> Value {
        if s.starts_with("${") && s.ends_with("}") {
            let var_name = &s[2..s.len() - 1];
            context.get(var_name).cloned().unwrap_or(Value::Null)
        } else if s.starts_with('"') && s.ends_with('"') {
            Value::String(s[1..s.len() - 1].to_string())
        } else if s == "true" {
            Value::Bool(true)
        } else if s == "false" {
            Value::Bool(false)
        } else if let Ok(num) = s.parse::<i64>() {
            Value::Number(serde_json::Number::from(num))
        } else if let Ok(num) = s.parse::<f64>() {
            Value::Number(serde_json::Number::from_f64(num).unwrap_or(serde_json::Number::from(0)))
        } else {
            Value::String(s.to_string())
        }
    }

    /// Resolve a string to a numeric value
    fn resolve_numeric(&self, s: &str, context: &HashMap<String, Value>) -> Option<f64> {
        let value = self.resolve_value(s, context);
        value.as_f64()
    }
}

impl fmt::Display for ActionCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionCondition::Always => write!(f, "true"),
            ActionCondition::Never => write!(f, "false"),
            ActionCondition::KeyExists(key) => write!(f, "exists({key})"),
            ActionCondition::KeyEquals(key, value) => write!(f, "{key} == {value}"),
            ActionCondition::NumericCompare {
                key,
                operator,
                value,
            } => {
                let op_str = match operator {
                    ComparisonOperator::Equal => "==",
                    ComparisonOperator::NotEqual => "!=",
                    ComparisonOperator::GreaterThan => ">",
                    ComparisonOperator::GreaterThanOrEqual => ">=",
                    ComparisonOperator::LessThan => "<",
                    ComparisonOperator::LessThanOrEqual => "<=",
                };
                write!(f, "{key} {op_str} {value}")
            }
            ActionCondition::Expression(expr) => write!(f, "({expr})"),
            ActionCondition::And(conditions) => {
                write!(
                    f,
                    "({})",
                    conditions
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(" && ")
                )
            }
            ActionCondition::Or(conditions) => {
                write!(
                    f,
                    "({})",
                    conditions
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(" || ")
                )
            }
            ActionCondition::Not(condition) => write!(f, "!({condition})"),
        }
    }
}
