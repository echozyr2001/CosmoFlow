use std::collections::HashMap;

use async_trait::async_trait;
use cosmoflow::action::Action;
use cosmoflow::node::{ExecutionContext, Node, NodeError};
use cosmoflow::prelude::*;
use cosmoflow::shared_store::SharedStore;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SimpleStorage {
    /// Internal data store using JSON values for flexibility
    data: HashMap<String, serde_json::Value>,
}

impl SimpleStorage {
    /// Creates a new empty storage instance.
    ///
    /// This initializes the internal HashMap that will store all data
    /// as JSON values, allowing for flexible type storage and retrieval.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Default for SimpleStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for SimpleStorage {
    type Error = SimpleStorageError;

    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.get(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value.clone())
                    .map_err(|e| SimpleStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn set<T: Serialize>(&mut self, key: String, value: T) -> Result<(), Self::Error> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| SimpleStorageError::SerializationError(e.to_string()))?;
        self.data.insert(key, json_value);
        Ok(())
    }

    fn remove<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Self::Error> {
        match self.data.remove(key) {
            Some(value) => {
                let deserialized = serde_json::from_value(value)
                    .map_err(|e| SimpleStorageError::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    fn contains_key(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.data.contains_key(key))
    }

    fn keys(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.data.clear();
        Ok(())
    }

    fn len(&self) -> Result<usize, Self::Error> {
        Ok(self.data.len())
    }
}

/// Error types for SimpleStorage operations.
///
/// This enum covers the two main categories of errors that can occur
/// when working with JSON serialization/deserialization in storage operations.
#[derive(Debug, thiserror::Error)]
pub enum SimpleStorageError {
    /// Error occurred during serialization to JSON
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// Error occurred during deserialization from JSON
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}
struct SimpleNode {
    message: String,
}

impl SimpleNode {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[async_trait]
impl Node<SimpleStorage> for SimpleNode {
    type PrepResult = Value;
    type ExecResult = Value;
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &SharedStore<SimpleStorage>,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(Value::String(self.message.clone()))
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
        store: &mut SharedStore<SimpleStorage>,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        store
            .set("result".to_string(), exec_result)
            .map_err(|e| NodeError::StorageError(e.to_string()))?;
        Ok(Action::simple("complete"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建流程 - 现在非常简单！
    let mut flow = FlowBuilder::new()
        .start_with("hello", SimpleNode::new("Hello, World!"))
        .build();

    let mut store = SharedStore::with_storage(SimpleStorage::new());

    let result = flow.execute(&mut store).await?;
    println!("Flow completed: {result:?}");

    let stored_result: Option<Value> = store.get("result")?;
    println!("Stored result: {stored_result:?}");

    Ok(())
}
