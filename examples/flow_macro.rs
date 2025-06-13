//! Flow Macro Example - Declarative Workflow Construction with Custom Action Routing
//!
//! This example demonstrates the powerful `flow!` macro for building CosmoFlow workflows
//! with declarative syntax and custom action routing. The workflow showcases:
//!
//! ## Workflow Behavior
//! - **Decision Node**: Evaluates conditions and chooses between success/error paths
//! - **Success Path**: Handles positive outcomes and continues to final processing
//! - **Error Path**: Handles negative outcomes and also continues to final processing
//! - **Final Node**: Convergence point where both paths complete the workflow
//! - **Custom Actions**: Uses specific action names like "default", "error", "continue"
//!
//! ## Advanced Features Demonstrated
//! - **Declarative Syntax**: Clean, readable workflow definition using the `flow!` macro
//! - **Custom Action Routing**: Specific action names for conditional workflow paths
//! - **Decision-Based Routing**: Nodes that choose different execution paths based on logic
//! - **Path Convergence**: Multiple execution paths that merge at a common endpoint
//! - **Custom Storage Backend**: Complete implementation with JSON serialization
//! - **Structured Workflow**: Explicit node and route definitions for complex flows
//!
//! ## Macro Syntax Features
//! - **Node Definition**: `"id" : NodeType` syntax for clean node registration
//! - **Action Routing**: `"from" - "action" => "to"` syntax for explicit action handling
//! - **Type Safety**: Compile-time storage type checking and validation
//! - **Flexible Routing**: Support for multiple actions from a single node
//!
//! ## Execution Flow
//! 1. Decision node evaluates business logic and selects execution path
//! 2. Success path processes positive outcomes
//! 3. Error path processes negative outcomes (alternative branch)
//! 4. Both paths converge at the final node for completion
//! 5. Workflow demonstrates conditional routing with custom action names
//!
//! This example is perfect for understanding advanced macro usage and conditional workflows.
//!
//! To run this example:
//! ```bash
//! cd examples && cargo run --bin flow_macro --features basic
//! ```

use std::collections::HashMap;

use cosmoflow::StorageBackend;
use cosmoflow::flow::FlowBackend;
use cosmoflow::flow::macros::flow;
use cosmoflow::shared_store::SharedStore;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A simple in-memory storage implementation for the workflow
///
/// This storage backend provides JSON-based serialization and maintains
/// all data in memory using a HashMap. It implements the complete
/// StorageBackend trait required by CosmoFlow.
#[derive(Debug, Clone)]
pub struct SimpleStorage {
    /// Internal data store using JSON values for flexible data types
    data: HashMap<String, serde_json::Value>,
}

impl SimpleStorage {
    /// Creates a new empty storage instance
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

/// Error types for the simple storage implementation
#[derive(Debug, thiserror::Error)]
pub enum SimpleStorageError {
    /// Error during JSON serialization
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// Error during JSON deserialization
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Decision node that evaluates conditions and chooses execution paths
///
/// This node demonstrates conditional routing by analyzing business logic
/// and returning different actions based on the evaluation results.
struct DecisionNode;

#[async_trait::async_trait]
impl<S: cosmoflow::storage::StorageBackend + Send + Sync> cosmoflow::node::Node<S>
    for DecisionNode
{
    type PrepResult = ();
    type ExecResult = bool;
    type Error = cosmoflow::node::NodeError;

    async fn prep(
        &mut self,
        _: &cosmoflow::shared_store::SharedStore<S>,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Decision Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<bool, Self::Error> {
        println!("Decision Node: Execution phase - simulating decision logic");
        // Simulate a decision process
        let success = true; // In real applications, this would contain actual decision logic
        Ok(success)
    }

    async fn post(
        &mut self,
        _: &mut cosmoflow::shared_store::SharedStore<S>,
        _: (),
        result: bool,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        if result {
            println!("Decision Node: Choosing success path");
            Ok(cosmoflow::action::Action::simple("default"))
        } else {
            println!("Decision Node: Choosing error path");
            Ok(cosmoflow::action::Action::simple("error"))
        }
    }
}

/// Success path node that handles positive outcomes
///
/// This node processes successful scenarios and continues the workflow
/// toward the final convergence point.
struct SuccessNode;

#[async_trait::async_trait]
impl<S: cosmoflow::storage::StorageBackend + Send + Sync> cosmoflow::node::Node<S> for SuccessNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = cosmoflow::node::NodeError;

    async fn prep(
        &mut self,
        _: &cosmoflow::shared_store::SharedStore<S>,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Success Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Success Node: Processing successful scenario");
        Ok(())
    }

    async fn post(
        &mut self,
        _: &mut cosmoflow::shared_store::SharedStore<S>,
        _: (),
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        println!("Success Node: Continuing to final node");
        Ok(cosmoflow::action::Action::simple("continue"))
    }
}

/// Error path node that handles negative outcomes
///
/// This node processes error scenarios and provides an alternative
/// execution path that also leads to the final convergence point.
struct ErrorNode;

#[async_trait::async_trait]
impl<S: cosmoflow::storage::StorageBackend + Send + Sync> cosmoflow::node::Node<S> for ErrorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = cosmoflow::node::NodeError;

    async fn prep(
        &mut self,
        _: &cosmoflow::shared_store::SharedStore<S>,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Error Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Error Node: Processing error scenario");
        Ok(())
    }

    async fn post(
        &mut self,
        _: &mut cosmoflow::shared_store::SharedStore<S>,
        _: (),
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        println!("Error Node: Continuing to final node");
        Ok(cosmoflow::action::Action::simple("continue"))
    }
}

/// Final convergence node where all execution paths complete
///
/// This node serves as the endpoint for both success and error paths,
/// demonstrating how different workflow branches can converge.
struct FinalNode;

#[async_trait::async_trait]
impl<S: cosmoflow::storage::StorageBackend + Send + Sync> cosmoflow::node::Node<S> for FinalNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = cosmoflow::node::NodeError;

    async fn prep(
        &mut self,
        _: &cosmoflow::shared_store::SharedStore<S>,
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Final Node: Preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<(), Self::Error> {
        println!("Final Node: Workflow ending");
        Ok(())
    }

    async fn post(
        &mut self,
        _: &mut cosmoflow::shared_store::SharedStore<S>,
        _: (),
        _: (),
        _: &cosmoflow::node::ExecutionContext,
    ) -> Result<cosmoflow::action::Action, Self::Error> {
        println!("Final Node: Workflow completed");
        Ok(cosmoflow::action::Action::simple("complete"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a workflow using custom action routing with the flow! macro
    let mut workflow = flow! {
        storage: SimpleStorage,
        start: "decision",
        nodes: {
            "decision" : DecisionNode,
            "success_path" : SuccessNode,
            "error_path" : ErrorNode,
            "final" : FinalNode,
        },
        routes: {
            "decision" - "default" => "success_path",
            "decision" - "error" => "error_path",
            "success_path" - "continue" => "final",
            "error_path" - "continue" => "final",
        }
    };

    let mut store = SharedStore::with_storage(SimpleStorage::new());
    let result = workflow.execute(&mut store).await?;

    println!("Workflow execution completed!");
    println!("Steps executed: {}", result.steps_executed);
    println!("Execution path: {:?}", result.execution_path);
    println!("Success: {}", result.success);

    Ok(())
}
