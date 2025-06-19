//! # LLM Request Example
//!
//! This example demonstrates how to integrate LLM functionality into CosmoFlow
//! workflows using direct HTTP client patterns instead of heavy SDK dependencies.
//!
//! This approach shows how to implement LLM integration patterns that can be
//! copied into your own projects as needed, rather than depending on framework-provided
//! abstractions.
//!
//! ## Environment Variables
//!
//! Set these environment variables before running:
//! - `LLM_API_KEY`: Your API key (e.g., OpenAI API key)
//! - `LLM_BASE_URL`: Base URL for the API (e.g., "https://api.openai.com/v1")
//! - `LLM_MODEL`: Model name (e.g., "gpt-3.5-turbo", "gpt-4")

use async_trait::async_trait;
use cosmoflow::flow::errors::FlowError;
use cosmoflow::prelude::*;
use cosmoflow::shared_store::backends::MemoryStorage;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;

// ============================================================================
// LLM INTEGRATION UTILITIES
// ============================================================================
// These utilities show patterns you can copy into your own projects

/// A lightweight HTTP client wrapper for LLM API calls
///
/// This demonstrates a simple pattern for making HTTP requests to LLM APIs
/// without heavy SDK dependencies. Copy this pattern into your own code.
pub struct LlmClient {
    client: reqwest::Client,
    base_url: String,
    headers: HashMap<String, String>,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            headers: HashMap::new(),
        }
    }

    /// Add a header to all requests
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Make a POST request to the specified endpoint
    pub async fn post(&self, endpoint: &str, body: &Value) -> Result<Value, reqwest::Error> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        let mut request = self.client.post(&url).json(body);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        let json: Value = response.json().await?;
        Ok(json)
    }
}

/// Utility function to create a simple chat completion request
pub fn create_chat_request(model: &str, messages: Vec<Value>) -> Value {
    json!({
        "model": model,
        "messages": messages
    })
}

/// Utility function to create a user message
pub fn user_message(content: &str) -> Value {
    json!({
        "role": "user",
        "content": content
    })
}

/// Extract content from a chat completion response
pub fn extract_content(response: &Value) -> Option<String> {
    response
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()
        .map(|s| s.to_string())
}

/// Configuration for LLM API from environment variables
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl LlmConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let api_key =
            env::var("LLM_API_KEY").map_err(|_| "LLM_API_KEY environment variable not set")?;

        let base_url =
            env::var("LLM_BASE_URL").map_err(|_| "LLM_BASE_URL environment variable not set")?;

        let model = env::var("LLM_MODEL").map_err(|_| "LLM_MODEL environment variable not set")?;

        Ok(Self {
            api_key,
            base_url,
            model,
        })
    }
}

// ============================================================================
// WORKFLOW NODES
// ============================================================================
// These show how to implement workflow patterns without framework abstractions

/// Simple logging node that demonstrates direct implementation
/// instead of using framework-provided LogNode
struct SimpleLogNode {
    message: String,
    next_action: Action,
}

impl SimpleLogNode {
    fn new(message: &str, next_action: Action) -> Self {
        Self {
            message: message.to_string(),
            next_action,
        }
    }
}

#[async_trait]
impl<S: SharedStore + Send + Sync> Node<S> for SimpleLogNode {
    type PrepResult = String;
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &S,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok(self.message.clone())
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("🚀 {}", prep_result);
        Ok(())
    }

    async fn post(
        &mut self,
        _store: &mut S,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(self.next_action.clone())
    }

    fn name(&self) -> &str {
        "SimpleLogNode"
    }
}

/// Data setup node that demonstrates direct store operations
/// instead of using framework-provided SetValueNode
struct DataSetupNode {
    next_action: Action,
}

impl DataSetupNode {
    fn new(next_action: Action) -> Self {
        Self { next_action }
    }
}

#[async_trait]
impl<S: SharedStore + Send + Sync> Node<S> for DataSetupNode {
    type PrepResult = String;
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        _store: &S,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        Ok("Setting up workflow data".to_string())
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("📊 {}", prep_result);
        Ok(())
    }

    async fn post(
        &mut self,
        store: &mut S,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Direct store operations - no abstraction needed
        store
            .set("user_prompt".to_string(), "What is the meaning of life?")
            .unwrap();

        // Load LLM configuration from environment variables
        let config = LlmConfig::from_env()
            .map_err(|e| NodeError::ExecutionError(format!("Failed to load LLM config: {}", e)))?;

        store.set("llm_config".to_string(), config).unwrap();

        Ok(self.next_action.clone())
    }

    fn name(&self) -> &str {
        "DataSetupNode"
    }
}

/// Real LLM node that demonstrates HTTP-based LLM integration patterns
/// This makes actual HTTP calls to LLM APIs using environment configuration
struct LlmNode {
    prompt_key: String,
    response_key: String,
    next_action: Action,
}

impl LlmNode {
    fn new(prompt_key: &str, response_key: &str, next_action: Action) -> Self {
        Self {
            prompt_key: prompt_key.to_string(),
            response_key: response_key.to_string(),
            next_action,
        }
    }
}

#[async_trait]
impl<S: SharedStore + Send + Sync> Node<S> for LlmNode {
    type PrepResult = (String, LlmConfig);
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &S,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        let prompt: String = store.get(&self.prompt_key).unwrap().unwrap();
        let config: LlmConfig = store.get("llm_config").unwrap().unwrap();
        Ok((prompt, config))
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        let (prompt, config) = prep_result;

        println!("🤖 Processing prompt with {}: {}", config.model, prompt);

        // Create HTTP client with authorization header
        let client = LlmClient::new(&config.base_url)
            .with_header("Authorization", format!("Bearer {}", config.api_key))
            .with_header("Content-Type", "application/json");

        // Build request using utility functions
        let messages = vec![user_message(&prompt)];
        let request = create_chat_request(&config.model, messages);

        // Make API call
        let response = client
            .post("chat/completions", &request)
            .await
            .map_err(|e| NodeError::ExecutionError(format!("LLM API request failed: {}", e)))?;

        // Extract response content
        let content = extract_content(&response).ok_or_else(|| {
            NodeError::ExecutionError("Failed to extract content from LLM response".to_string())
        })?;

        println!("✅ Received response from {}", config.model);

        Ok(content)
    }

    async fn post(
        &mut self,
        store: &mut S,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        // Direct store operation - no abstraction layer needed
        store.set(self.response_key.clone(), exec_result).unwrap();

        Ok(self.next_action.clone())
    }

    fn name(&self) -> &str {
        "LlmNode"
    }
}

/// Result display node that demonstrates data retrieval patterns
struct ResultDisplayNode {
    data_key: String,
}

impl ResultDisplayNode {
    fn new(data_key: &str) -> Self {
        Self {
            data_key: data_key.to_string(),
        }
    }
}

#[async_trait]
impl<S: SharedStore + Send + Sync> Node<S> for ResultDisplayNode {
    type PrepResult = String;
    type ExecResult = ();
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &S,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        let data: String = store.get(&self.data_key).unwrap().unwrap();
        Ok(data)
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        println!("✨ AI Response: {}", prep_result);
        println!("🎉 Workflow completed successfully!");
        Ok(())
    }

    async fn post(
        &mut self,
        _store: &mut S,
        _prep_result: Self::PrepResult,
        _exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::simple("complete"))
    }

    fn name(&self) -> &str {
        "ResultDisplayNode"
    }
}

// ============================================================================
// MAIN EXAMPLE
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), FlowError> {
    println!("🌟 CosmoFlow Real LLM Integration Example");
    println!("==========================================");
    println!();
    println!("This example demonstrates real LLM API integration:");
    println!("• Direct HTTP calls to LLM APIs");
    println!("• Configuration from environment variables");
    println!("• Error handling for API failures");
    println!("• Clean, readable code with minimal overhead");
    println!();

    // Check environment variables early
    if let Err(e) = LlmConfig::from_env() {
        eprintln!("❌ Configuration Error: {}", e);
        eprintln!();
        eprintln!("Please set the following environment variables:");
        eprintln!("  LLM_API_KEY=your_api_key");
        eprintln!("  LLM_BASE_URL=https://api.openai.com/v1");
        eprintln!("  LLM_MODEL=gpt-3.5-turbo");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  export LLM_API_KEY=sk-...");
        eprintln!("  export LLM_BASE_URL=https://api.openai.com/v1");
        eprintln!("  export LLM_MODEL=gpt-3.5-turbo");
        eprintln!("  cargo run --bin llm_request -p cosmoflow-examples");
        std::process::exit(1);
    }

    // Create storage - just the core framework
    let mut store = MemoryStorage::new();

    // Build workflow using direct implementations
    let mut flow = FlowBuilder::new()
        .node(
            "start",
            SimpleLogNode::new("Starting real LLM workflow", Action::simple("setup")),
        )
        .node("setup", DataSetupNode::new(Action::simple("llm")))
        .node(
            "llm",
            LlmNode::new("user_prompt", "ai_response", Action::simple("display")),
        )
        .node("display", ResultDisplayNode::new("ai_response"))
        .route("start", "setup", "setup")
        .route("setup", "llm", "llm")
        .route("llm", "display", "display")
        .terminal_route("display", "complete") // Explicit termination
        .build();

    // Execute workflow
    let _result = flow.execute(&mut store).await?;

    println!();
    println!("✅ Workflow executed successfully!");
    println!();
    println!("Key benefits of this approach:");
    println!("• 🌐 Real API integration with environment configuration");
    println!("• 🪶 Lightweight: Minimal dependencies");
    println!("• 🎯 Direct: No unnecessary abstractions");
    println!("• 🔧 Flexible: Easy to customize for different APIs");
    println!("• 📖 Readable: Clear, understandable code");
    println!("• 📋 Copyable: Patterns you can use in your own projects");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_utilities() {
        // Test utility functions
        let messages = vec![user_message("Hello")];
        let request = create_chat_request("gpt-3.5-turbo", messages);

        assert_eq!(request["model"], "gpt-3.5-turbo");
        assert_eq!(request["messages"][0]["role"], "user");
        assert_eq!(request["messages"][0]["content"], "Hello");

        // Test response extraction
        let response = json!({
            "choices": [{
                "message": {
                    "content": "Hello there!"
                }
            }]
        });

        assert_eq!(extract_content(&response), Some("Hello there!".to_string()));
    }

    #[test]
    fn test_llm_config_validation() {
        // Test that config validation works
        // Note: This test will fail if environment variables are not set,
        // which is expected behavior
        match LlmConfig::from_env() {
            Ok(config) => {
                assert!(!config.api_key.is_empty());
                assert!(!config.base_url.is_empty());
                assert!(!config.model.is_empty());
            }
            Err(_) => {
                // Expected when environment variables are not set
                println!("Environment variables not set - this is expected in test environment");
            }
        }
    }

    #[tokio::test]
    async fn test_llm_client() {
        let client = LlmClient::new("https://api.example.com")
            .with_header("Authorization", "Bearer test-key");

        // Test that client is constructed properly
        assert_eq!(client.base_url, "https://api.example.com");
        assert_eq!(
            client.headers.get("Authorization"),
            Some(&"Bearer test-key".to_string())
        );
    }
}
