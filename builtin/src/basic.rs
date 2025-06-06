use std::time::Duration;

use action::Action;
use async_trait::async_trait;
use node::{ExecutionContext, NodeBackend, NodeError};
use serde_json::Value;
use shared_store::SharedStore;
use storage::StorageBackend;

/// A simple node that logs messages and passes through
pub struct LogNodeBackend {
    message: String,
    action: Action,
    max_retries: usize,
    retry_delay: Duration,
}

impl LogNodeBackend {
    /// Create a new log node
    pub fn new<S: Into<String>>(message: S, action: Action) -> Self {
        Self {
            message: message.into(),
            action,
            max_retries: 1,
            retry_delay: Duration::ZERO,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> NodeBackend<S> for LogNodeBackend {
    type PrepResult = String;
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(format!(
            "Execution {}: {}",
            context.execution_id, &self.message
        ))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("{}", prep_result);
        Ok(prep_result)
    }

    async fn post(
        &mut self,
        _store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "LogNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }

    fn retry_delay(&self) -> Duration {
        self.retry_delay
    }
}

/// A node that sets a value in the shared store
pub struct SetValueNodeBackend {
    key: String,
    value: Value,
    action: Action,
    max_retries: usize,
}

impl SetValueNodeBackend {
    /// Create a new set value node
    pub fn new<S: Into<String>>(key: S, value: Value, action: Action) -> Self {
        Self {
            key: key.into(),
            value,
            action,
            max_retries: 1,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> NodeBackend<S> for SetValueNodeBackend {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Ok(())
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        store
            .set(self.key.clone(), self.value.clone())
            .map_err(|e| NodeError::StorageError(e.to_string()))?;
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "SetValueNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// A node that gets a value from the shared store and optionally transforms it
pub struct GetValueNodeBackend<F>
where
    F: Fn(Option<Value>) -> Value + Send + Sync,
{
    key: String,
    output_key: String,
    transform: F,
    action: Action,
    max_retries: usize,
}

impl<F> GetValueNodeBackend<F>
where
    F: Fn(Option<Value>) -> Value + Send + Sync,
{
    /// Create a new get value node
    pub fn new<S1: Into<String>, S2: Into<String>>(
        key: S1,
        output_key: S2,
        transform: F,
        action: Action,
    ) -> Self {
        Self {
            key: key.into(),
            output_key: output_key.into(),
            transform,
            action,
            max_retries: 1,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S, F> NodeBackend<S> for GetValueNodeBackend<F>
where
    S: StorageBackend + Send + Sync,
    F: Fn(Option<Value>) -> Value + Send + Sync,
{
    type PrepResult = Option<Value>;
    type ExecResult = Value;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        store
            .get(&self.key)
            .map_err(|e| NodeError::StorageError(e.to_string()))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Ok((self.transform)(prep_result))
    }

    async fn post(
        &mut self,
        store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        store
            .set(self.output_key.clone(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "GetValueNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// A conditional node that chooses actions based on store content
pub struct ConditionalNodeBackend<F, S>
where
    F: Fn(&SharedStore<S>) -> bool + Send + Sync,
    S: StorageBackend,
{
    condition: F,
    if_true: Action,
    if_false: Action,
    max_retries: usize,
    _phantom: std::marker::PhantomData<S>,
}

impl<F, S> ConditionalNodeBackend<F, S>
where
    F: Fn(&SharedStore<S>) -> bool + Send + Sync,
    S: StorageBackend,
{
    /// Create a new conditional node
    pub fn new(condition: F, if_true: Action, if_false: Action) -> Self {
        Self {
            condition,
            if_true,
            if_false,
            max_retries: 1,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S, F> NodeBackend<S> for ConditionalNodeBackend<F, S>
where
    S: StorageBackend + Send + Sync,
    F: Fn(&SharedStore<S>) -> bool + Send + Sync,
{
    type PrepResult = bool;
    type ExecResult = bool;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok((self.condition)(store))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Ok(prep_result)
    }

    async fn post(
        &mut self,
        _store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        if exec_result {
            Ok(self.if_true.clone())
        } else {
            Ok(self.if_false.clone())
        }
    }

    fn name(&self) -> &str {
        "ConditionalNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// A delay node that waits for a specified duration
pub struct DelayNodeBackend {
    duration: Duration,
    action: Action,
    max_retries: usize,
}

impl DelayNodeBackend {
    /// Create a new delay node
    pub fn new(duration: Duration, action: Action) -> Self {
        Self {
            duration,
            action,
            max_retries: 1,
        }
    }

    /// Set maximum retries
    pub fn with_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

#[async_trait]
impl<S: StorageBackend + Send + Sync> NodeBackend<S> for DelayNodeBackend {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<S>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        tokio::time::sleep(self.duration).await;
        Ok(())
    }

    async fn post(
        &mut self,
        _store: &mut SharedStore<S>,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(self.action.clone())
    }

    fn name(&self) -> &str {
        "DelayNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }
}

/// Helper function to create a simple log node with default settings
pub fn log<S: Into<String>>(message: S) -> LogNodeBackend {
    LogNodeBackend::new(message, Action::simple("continue"))
}

/// Helper function to create a simple set value node
pub fn set_value<S: Into<String>>(key: S, value: Value) -> SetValueNodeBackend {
    SetValueNodeBackend::new(key, value, Action::simple("continue"))
}

/// Helper function to create a simple delay node
pub fn delay(duration: Duration) -> DelayNodeBackend {
    DelayNodeBackend::new(duration, Action::simple("continue"))
}

/// Helper function to create a get value node with identity transform
pub fn get_value<S1: Into<String>, S2: Into<String>>(
    key: S1,
    output_key: S2,
) -> GetValueNodeBackend<impl Fn(Option<Value>) -> Value + Send + Sync> {
    GetValueNodeBackend::new(
        key,
        output_key,
        |value| value.unwrap_or(Value::Null),
        Action::simple("continue"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use shared_store::SharedStore;
    use storage::MemoryStorage;

    #[tokio::test]
    async fn test_optimized_log_node() {
        let mut node: LogNodeBackend = log("test message");
        let store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        let context = ExecutionContext::new(1, Duration::ZERO);

        let prep_result =
            <LogNodeBackend as NodeBackend<MemoryStorage>>::prep(&mut node, &store, &context)
                .await
                .unwrap();
        assert!(prep_result.contains("test message"));

        let exec_result = <LogNodeBackend as NodeBackend<MemoryStorage>>::exec(
            &mut node,
            prep_result.clone(),
            &context,
        )
        .await
        .unwrap();
        assert_eq!(exec_result, prep_result);
    }

    #[tokio::test]
    async fn test_optimized_set_value_node() {
        let mut node: SetValueNodeBackend = set_value("test_key", json!("test_value"));
        let mut store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        let context = ExecutionContext::new(1, Duration::ZERO);

        <SetValueNodeBackend as NodeBackend<MemoryStorage>>::prep(&mut node, &store, &context)
            .await
            .unwrap();
        <SetValueNodeBackend as NodeBackend<MemoryStorage>>::exec(&mut node, (), &context)
            .await
            .unwrap();
        let action = <SetValueNodeBackend as NodeBackend<MemoryStorage>>::post(
            &mut node,
            &mut store,
            (),
            (),
            &context,
        )
        .await
        .unwrap();

        assert_eq!(action, Action::simple("continue"));
        assert_eq!(store.get("test_key").unwrap(), Some(json!("test_value")));
    }

    #[tokio::test]
    async fn test_optimized_delay_node() {
        let mut node: DelayNodeBackend = delay(Duration::from_millis(1));
        let mut store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        let context = ExecutionContext::new(1, Duration::ZERO);

        let start = std::time::Instant::now();
        <DelayNodeBackend as NodeBackend<MemoryStorage>>::prep(&mut node, &store, &context)
            .await
            .unwrap();
        <DelayNodeBackend as NodeBackend<MemoryStorage>>::exec(&mut node, (), &context)
            .await
            .unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(1));
        let action = <DelayNodeBackend as NodeBackend<MemoryStorage>>::post(
            &mut node,
            &mut store,
            (),
            (),
            &context,
        )
        .await
        .unwrap();
        assert_eq!(action, Action::simple("continue"));
    }

    #[tokio::test]
    async fn test_optimized_get_value_node() {
        let mut store: SharedStore<MemoryStorage> = SharedStore::with_storage(MemoryStorage::new());
        store
            .set("input_key".to_string(), json!("input_value"))
            .unwrap();

        let mut node = get_value("input_key", "output_key");
        let context = ExecutionContext::new(1, Duration::ZERO);

        let prep_result = <GetValueNodeBackend<_> as NodeBackend<MemoryStorage>>::prep(
            &mut node, &store, &context,
        )
        .await
        .unwrap();
        assert_eq!(prep_result, Some(json!("input_value")));

        let exec_result = <GetValueNodeBackend<_> as NodeBackend<MemoryStorage>>::exec(
            &mut node,
            prep_result,
            &context,
        )
        .await
        .unwrap();
        assert_eq!(exec_result, json!("input_value"));

        let action = <GetValueNodeBackend<_> as NodeBackend<MemoryStorage>>::post(
            &mut node,
            &mut store,
            Some(json!("input_value")),
            exec_result,
            &context,
        )
        .await
        .unwrap();
        assert_eq!(action, Action::simple("continue"));
        assert_eq!(store.get("output_key").unwrap(), Some(json!("input_value")));
    }
}
