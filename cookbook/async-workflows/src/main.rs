//! Async FlowBuilder example with action-name routing.

use cosmoflow::action::Action;
use cosmoflow::node::{Node, NodeContext};
use cosmoflow::shared_store::backends::MemoryStorage;
use cosmoflow::{FlowBuilder, NodeError};

struct DecisionNode;

#[async_trait::async_trait]
impl Node<MemoryStorage> for DecisionNode {
    type Prep = ();
    type Output = bool;
    type Error = NodeError;

    async fn prep(
        &mut self,
        _state: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        println!("Decision Node: preparation phase");
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        println!("Decision Node: execution phase");
        Ok(true)
    }

    async fn post(
        &mut self,
        _state: &mut MemoryStorage,
        _prep: Self::Prep,
        success: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        if success {
            println!("Decision Node: choosing success path");
            Ok(Action::new("default"))
        } else {
            println!("Decision Node: choosing error path");
            Ok(Action::new("error"))
        }
    }
}

struct LogNode {
    message: &'static str,
    action: &'static str,
}

impl LogNode {
    fn new(message: &'static str, action: &'static str) -> Self {
        Self { message, action }
    }
}

#[async_trait::async_trait]
impl Node<MemoryStorage> for LogNode {
    type Prep = ();
    type Output = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        _state: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        println!("{}", self.message);
        Ok(())
    }

    async fn post(
        &mut self,
        _state: &mut MemoryStorage,
        _prep: Self::Prep,
        _output: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::new(self.action))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = FlowBuilder::<MemoryStorage>::new()
        .node("decision", DecisionNode)
        .node(
            "success_path",
            LogNode::new("Success Node: processing success", "continue"),
        )
        .node(
            "error_path",
            LogNode::new("Error Node: processing error", "continue"),
        )
        .node(
            "final",
            LogNode::new("Final Node: workflow completed", "complete"),
        )
        .start("decision")
        .route("decision", "default", "success_path")
        .route("decision", "error", "error_path")
        .route("success_path", "continue", "final")
        .route("error_path", "continue", "final")
        .build()?;

    let mut store = MemoryStorage::new();
    let execution = workflow.run_recorded(&mut store).await?;

    println!("Workflow execution completed!");
    println!("Steps executed: {}", execution.steps);
    println!("Execution path: {:?}", execution.path);
    println!("Final action: {}", execution.final_action);

    Ok(())
}
