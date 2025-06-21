//! Test for chat application loop pattern - demonstrating simplified approach

use cosmoflow::prelude::*;
use async_trait::async_trait;

struct InputNode;

#[async_trait]
impl Node<MemoryStorage> for InputNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(&mut self, _store: &MemoryStorage, _context: &ExecutionContext) -> Result<Self::PrepResult, Self::Error> {
        Ok(())
    }

    async fn exec(&mut self, _prep_result: Self::PrepResult, _context: &ExecutionContext) -> Result<Self::ExecResult, Self::Error> {
        println!("📥 Getting user input...");
        Ok(())
    }

    async fn post(&mut self, store: &mut MemoryStorage, _prep_result: Self::PrepResult, _exec_result: Self::ExecResult, _context: &ExecutionContext) -> Result<Action, Self::Error> {
        // Simulate getting user input
        let message_count: i32 = match store.get("message_count") {
            Ok(Some(count)) => count,
            _ => 0,
        };
        store.set("current_message".to_string(), format!("Message {}", message_count + 1)).unwrap();
        store.set("message_count".to_string(), message_count + 1).unwrap();
        
        // Stop after 5 messages for demo
        if message_count >= 5 {
            Ok(Action::simple("quit"))
        } else {
            Ok(Action::simple("process"))
        }
    }

    fn name(&self) -> &str {
        "InputNode"
    }
}

struct ProcessNode;

#[async_trait]
impl Node<MemoryStorage> for ProcessNode {
    type PrepResult = String;
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(&mut self, store: &MemoryStorage, _context: &ExecutionContext) -> Result<Self::PrepResult, Self::Error> {
        let message = store.get::<String>("current_message").unwrap().unwrap_or("".to_string());
        Ok(message)
    }

    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) -> Result<Self::ExecResult, Self::Error> {
        println!("🔄 Processing: {}", prep_result);
        let response = format!("Processed: {}", prep_result);
        Ok(response)
    }

    async fn post(&mut self, store: &mut MemoryStorage, _prep_result: Self::PrepResult, exec_result: Self::ExecResult, _context: &ExecutionContext) -> Result<Action, Self::Error> {
        store.set("response".to_string(), exec_result).unwrap();
        Ok(Action::simple("output"))
    }

    fn name(&self) -> &str {
        "ProcessNode"
    }
}

struct OutputNode;

#[async_trait]
impl Node<MemoryStorage> for OutputNode {
    type PrepResult = String;
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(&mut self, store: &MemoryStorage, _context: &ExecutionContext) -> Result<Self::PrepResult, Self::Error> {
        let response = store.get::<String>("response").unwrap().unwrap_or("".to_string());
        Ok(response)
    }

    async fn exec(&mut self, prep_result: Self::PrepResult, _context: &ExecutionContext) -> Result<Self::ExecResult, Self::Error> {
        println!("📤 Output: {}", prep_result);
        Ok(())
    }

    async fn post(&mut self, _store: &mut MemoryStorage, _prep_result: Self::PrepResult, _exec_result: Self::ExecResult, _context: &ExecutionContext) -> Result<Action, Self::Error> {
        Ok(Action::simple("input"))  // Back to input for next message
    }

    fn name(&self) -> &str {
        "OutputNode"
    }
}

struct QuitNode;

#[async_trait]
impl Node<MemoryStorage> for QuitNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(&mut self, _store: &MemoryStorage, _context: &ExecutionContext) -> Result<Self::PrepResult, Self::Error> {
        Ok(())
    }

    async fn exec(&mut self, _prep_result: Self::PrepResult, _context: &ExecutionContext) -> Result<Self::ExecResult, Self::Error> {
        println!("👋 Chat session ended");
        Ok(())
    }

    async fn post(&mut self, _store: &mut MemoryStorage, _prep_result: Self::PrepResult, _exec_result: Self::ExecResult, _context: &ExecutionContext) -> Result<Action, Self::Error> {
        Ok(Action::simple("complete"))
    }

    fn name(&self) -> &str {
        "QuitNode"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💬 Chat Application Loop Pattern - Simplified Approach");
    println!("======================================================\n");

    let mut store = MemoryStorage::new();

    println!("✨ Building Chat Flow with Natural Loop Support");
    println!("----------------------------------------------");
    
    let mut chat_flow = FlowBuilder::new()
        .start_node("input")
        .max_steps(20)  // Simple protection against infinite loops
        .node("input", InputNode)
        .node("process", ProcessNode)
        .node("output", OutputNode)
        .node("quit", QuitNode)
        .route("input", "process", "process")
        .route("process", "output", "output")
        .route("output", "input", "input")  // Natural chat loop - no workarounds needed!
        .route("input", "quit", "quit")
        .terminal_route("quit", "complete")
        .build();

    println!("Flow configuration:");
    println!("  max_steps: {}", chat_flow.config().max_steps);
    println!("  start_node_id: {}", chat_flow.config().start_node_id);
    println!();

    println!("🚀 Executing Chat Flow...");
    println!("-------------------------");

    match chat_flow.execute(&mut store).await {
        Ok(result) => {
            println!("✅ Chat flow executed successfully!");
            println!("   Steps executed: {}", result.steps_executed);
            println!("   Messages processed: {}", store.get::<i32>("message_count").unwrap().unwrap_or(0));
            println!("   Execution path: {:?}", result.execution_path);
        }
        Err(e) => {
            println!("❌ Chat flow failed: {}", e);
        }
    }

    println!("\n🎯 Key Benefits");
    println!("===============");
    println!("• ✅ Natural loop patterns work out of the box");
    println!("• ✅ No need to disable cycle detection");
    println!("• ✅ Simple max_steps protection against infinite loops");
    println!("• ✅ Clean, readable flow definitions");
    println!("• ✅ Perfect for chat, game loops, and iterative workflows");

    Ok(())
}