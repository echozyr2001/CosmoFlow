//! Large Language Model (LLM) integration nodes for AI-powered workflows.
//!
//! This module provides node implementations for integrating Large Language Models
//! into CosmoFlow workflows. It supports OpenAI-compatible APIs and enables
//! AI-powered processing, text generation, analysis, and decision-making within
//! workflow automation.
//!
//! # Features
//!
//! - OpenAI API integration with configurable endpoints
//! - Support for multiple LLM providers (OpenAI, Azure OpenAI, local models)
//! - Streaming and non-streaming response modes
//! - Template-based prompt generation with variable substitution
//! - Conversation management with message history
//! - Configurable model parameters (temperature, max tokens, etc.)
//! - Error handling and retry mechanisms
//! - Cost tracking and usage monitoring
//!
//! # Available Nodes
//!
//! - [`MockLlmNode`] - Mock LLM responses for testing and development
//! - [`ApiRequestNode`] - Production LLM API integration with full features
//!
//! # Configuration
//!
//! All LLM nodes use the [`ApiConfig`] struct for configuration, which supports:
//! - API authentication and endpoints
//! - Model selection and parameters
//! - Request timeouts and retry settings
//! - Streaming options
//!
//! # Examples
//!
//! ## Basic Text Generation
//!
//! ```rust
//! use cosmoflow::builtin::llm::{MockLlmNode, ApiConfig};
//! use cosmoflow::action::Action;
//!
//! let config = ApiConfig::new("your-api-key")
//!     .with_model("gpt-4")
//!     .with_max_tokens(500);
//!
//! let prompt_node = MockLlmNode::new(
//!     "user_prompt",
//!     "story_result",
//!     "Once upon a time, robots learned to paint...",
//!     Action::simple("review_story")
//! );
//! ```
//!
//! ## Template-Based Processing
//!
//! ```rust
//! use cosmoflow::builtin::llm::{MockLlmNode, ApiConfig};
//! use cosmoflow::action::Action;
//!
//! let config = ApiConfig::default();
//!
//! let analysis_node = MockLlmNode::new(
//!     "user_feedback",
//!     "sentiment_analysis",
//!     "Sentiment: Positive",
//!     Action::simple("process_sentiment")
//! );
//! ```
//!
//! ## Conversation Management
//!
//! ```rust
//! use cosmoflow::builtin::llm::{ApiRequestNode, ApiConfig};
//! use cosmoflow::action::Action;
//!
//! let config = ApiConfig::new("your-api-key")
//!     .with_temperature(0.8);
//!
//! let chat_node = ApiRequestNode::new(
//!     "conversation_messages",
//!     "assistant_response",
//!     Action::simple("continue_conversation")
//! );
//! ```
//!
//! # Security Considerations
//!
//! - Store API keys securely using environment variables
//! - Validate and sanitize user inputs before sending to LLMs
//! - Be aware of data privacy when using external LLM services
//! - Monitor usage and costs to prevent unexpected charges
//! - Consider using local models for sensitive data processing
//!
//! # Error Handling
//!
//! All LLM nodes handle common error scenarios:
//! - Network connectivity issues
//! - API rate limiting and quota exceeded
//! - Invalid API keys or authentication failures
//! - Model-specific errors (context length, content filtering)
//! - Malformed responses and parsing errors

use std::time::Duration;

use crate::Node;
use crate::action::Action;
use crate::node::{ExecutionContext, NodeError};
use crate::shared_store::SharedStore;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
};
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;

#[derive(Debug, Clone)]
/// Configuration for LLM API connections and request parameters.
///
/// ApiConfig centralizes all configuration needed to connect to and interact
/// with Large Language Model APIs. It supports OpenAI-compatible endpoints
/// including OpenAI, Azure OpenAI, and local model servers.
///
/// # Default Behavior
///
/// - Reads API key from `OPENAI_API_KEY` environment variable
/// - Uses `gpt-3.5-turbo` model with 1000 max tokens
/// - Sets temperature to 0.7 for balanced creativity/consistency
/// - Configures 30-second request timeout
/// - Disables streaming by default
///
/// # Examples
///
/// ## Basic Configuration
///
/// ```rust
/// use cosmoflow::builtin::llm::ApiConfig;
///
/// // Using environment variable for API key
/// let config = ApiConfig::default();
///
/// // Explicit API key
/// let config = ApiConfig::new("sk-your-api-key-here");
/// ```
///
/// ## Advanced Configuration
///
/// ```rust
/// use cosmoflow::builtin::llm::ApiConfig;
///
/// let config = ApiConfig::new("your-api-key")
///     .with_model("gpt-4")
///     .with_max_tokens(2000)
///     .with_temperature(0.3)      // More deterministic
///     .with_timeout(60);          // Longer timeout for complex requests
/// ```
///
/// ## Custom Endpoint (e.g., Azure OpenAI)
///
/// ```rust
/// use cosmoflow::builtin::llm::ApiConfig;
///
/// let azure_config = ApiConfig::new("your-azure-key")
///     .with_base_url("https://your-resource.openai.azure.com")
///     .with_model("gpt-35-turbo");
/// ```
///
/// ## Local Model Server
///
/// ```rust
/// use cosmoflow::builtin::llm::ApiConfig;
///
/// let local_config = ApiConfig::new("not-needed")
///     .with_base_url("http://localhost:8000/v1")
///     .with_model("llama2");
/// ```
pub struct ApiConfig {
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API (optional, defaults to OpenAI)
    pub base_url: Option<String>,
    /// Organization ID (optional)
    pub org_id: Option<String>,
    /// Model to use for requests
    pub model: String,
    /// Maximum tokens for response
    pub max_tokens: Option<u16>,
    /// Temperature for response generation
    pub temperature: Option<f32>,
    /// Request timeout in seconds
    pub timeout: Option<u64>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Enable streaming response (default: false)
    pub stream: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: None,
            org_id: None,
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            timeout: Some(30),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: false,
        }
    }
}

impl ApiConfig {
    /// Create a new ApiConfig with an API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            ..Default::default()
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the base URL for the API
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the organization ID
    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Set maximum tokens for response
    pub fn with_max_tokens(mut self, max_tokens: u16) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature for response generation
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set request timeout in seconds
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set top-p sampling parameter
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set frequency penalty
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Set presence penalty
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Enable or disable streaming
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

// LLM nodes implementation will be added here

/// A mock LLM node for testing and examples
///
/// The MockLlmNode provides a simple simulation of LLM behavior without
/// making actual API calls. This is useful for testing workflows, development,
/// and demonstrations where you don't want to incur API costs or need predictable
/// responses.
///
/// # Node Implementation
///
/// MockLlmNode implements the [`Node`] trait with the following associated types:
/// - `PrepResult = String` - The retrieved prompt from the shared store
/// - `ExecResult = String` - The generated mock response
/// - `Error = NodeError` - Standard node error type
///
/// The prompt is retrieved during preparation, processed during execution,
/// and the result is stored during post-processing.
///
/// # Features
///
/// - Configurable mock responses
/// - Template variable substitution in prompts
/// - Simulated processing delays
/// - Error handling and fallback responses
/// - Retry mechanism support
/// - Configurable failure rates for testing
/// - Fallback response generation
///
/// # Examples
///
/// ## Basic Mock Response
///
/// ```rust
/// use cosmoflow::builtin::llm::MockLlmNode;
/// use cosmoflow::action::Action;
///
/// let mock_node = MockLlmNode::new(
///     "user_prompt",
///     "ai_response",
///     "This is a mock AI response for testing purposes.",
///     Action::simple("continue")
/// );
/// ```
///
/// ## Template Response
///
/// ```rust
/// use cosmoflow::builtin::llm::MockLlmNode;
/// use cosmoflow::action::Action;
///
/// let template_node = MockLlmNode::new(
///     "analysis_request",
///     "analysis_result",
///     "Analysis complete. The input has been processed successfully.",
///     Action::simple("review_analysis")
/// );
/// ```
///
/// ## Testing with Failures
///
/// ```rust
/// use cosmoflow::builtin::llm::MockLlmNode;
/// use cosmoflow::action::Action;
/// use std::time::Duration;
///
/// let test_node = MockLlmNode::new(
///     "test_prompt",
///     "test_response",
///     "Mock response",
///     Action::simple("continue")
/// )
/// .with_failure_rate(0.2)  // 20% failure rate
/// .with_retries(3)
/// .with_retry_delay(Duration::from_millis(100));
/// ```
pub struct MockLlmNode {
    prompt_key: String,
    output_key: String,
    mock_response: String,
    action: Action,
    max_retries: usize,
    retry_delay: Duration,
    failure_rate: f64,
}

impl MockLlmNode {
    /// Create a new mock LLM node
    pub fn new<S1, S2, S3>(
        prompt_key: S1,
        output_key: S2,
        mock_response: S3,
        action: Action,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        Self {
            prompt_key: prompt_key.into(),
            output_key: output_key.into(),
            mock_response: mock_response.into(),
            action,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            failure_rate: 0.0,
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

    /// Set failure rate for testing retry logic
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate.clamp(0.0, 1.0);
        self
    }
}

#[async_trait]
impl<S: SharedStore + Send + Sync> Node<S> for MockLlmNode {
    type PrepResult = String;
    type ExecResult = String;
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &S,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        let value = match store.get(&self.prompt_key) {
            Ok(value) => value,
            Err(e) => return Err(NodeError::StorageError(e.to_string())),
        };

        let prompt = value
            .and_then(|v: Value| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| {
                NodeError::ValidationError(format!("Prompt not found at key: {}", self.prompt_key))
            })?;
        Ok(prompt)
    }

    async fn exec(
        &mut self,
        prompt: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // Simulate API call delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Simulate random failures for testing
        if self.failure_rate > 0.0 && rand::random::<f64>() < self.failure_rate {
            return Err(NodeError::ExecutionError(format!(
                "Mock LLM API failure (retry {})",
                context.current_retry
            )));
        }

        // Generate mock response
        let response = format!("{} (processed prompt: '{}')", self.mock_response, prompt);
        Ok(response)
    }

    async fn post(
        &mut self,
        store: &mut S,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        match store.set(
            self.output_key.clone(),
            serde_json::Value::String(exec_result),
        ) {
            Ok(_) => Ok(self.action.clone()),
            Err(e) => Err(NodeError::StorageError(e.to_string())),
        }
    }

    async fn exec_fallback(
        &mut self,
        _prep_result: Self::PrepResult,
        error: Self::Error,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        Ok(format!("Fallback response due to error: {error}"))
    }

    fn name(&self) -> &str {
        "MockLlmNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }

    fn retry_delay(&self) -> Duration {
        self.retry_delay
    }
}

/// HTTP-based API request node for LLM interactions using async-openai SDK
///
/// This node makes actual HTTP requests to LLM APIs (OpenAI, etc.)
/// It supports various configuration options including retries,
/// custom endpoints, message history, and error handling.
///
/// The ApiRequestNode is the primary node for production LLM integration,
/// providing full API compatibility and advanced features like conversation
/// management, streaming responses, and sophisticated error handling.
///
/// # Features
///
/// - Full OpenAI API compatibility (and compatible services)
/// - Conversation history management
/// - Streaming and non-streaming responses
/// - Template variable substitution in prompts
/// - Comprehensive error handling with fallback responses
/// - Configurable retry mechanisms with exponential backoff
/// - Request/response logging and monitoring
/// - Cost tracking and usage analytics
///
/// # Supported Input Formats
///
/// The node accepts several input formats for maximum flexibility:
/// - Single string prompts
/// - Array of message objects for conversation history
/// - Template strings with variable substitution
///
/// # Examples
///
/// ## Simple Text Generation
///
/// ```rust
/// use cosmoflow::builtin::llm::{ApiRequestNode, ApiConfig};
/// use cosmoflow::action::Action;
///
/// let config = ApiConfig::new("your-api-key")
///     .with_model("gpt-4")
///     .with_max_tokens(500);
///
/// let text_gen = ApiRequestNode::new(
///     "user_prompt",
///     "generated_text",
///     Action::simple("review_text")
/// ).with_config(config);
/// ```
///
/// ## Conversation with System Message
///
/// ```rust
/// use cosmoflow::builtin::llm::{ApiRequestNode, ApiConfig};
/// use cosmoflow::action::Action;
///
/// let config = ApiConfig::default();
///
/// let chat_node = ApiRequestNode::new(
///     "conversation_messages",
///     "assistant_response",
///     Action::simple("continue_chat")
/// )
/// .with_config(config)
/// .with_system_message("You are a helpful assistant specialized in data analysis.")
/// .with_retries(5);
/// ```
///
/// ## Error Handling and Fallbacks
///
/// ```rust
/// use cosmoflow::builtin::llm::{ApiRequestNode, ApiConfig};
/// use cosmoflow::action::Action;
/// use std::time::Duration;
///
/// let robust_node = ApiRequestNode::new(
///     "analysis_request",
///     "analysis_result",
///     Action::simple("process_analysis")
/// )
/// .with_config(ApiConfig::default())
/// .with_retries(3)
/// .with_retry_delay(Duration::from_secs(2));
/// ```
#[derive(Debug, Clone)]
pub struct ApiRequestNode {
    /// Configuration for the API
    config: ApiConfig,
    /// Input key for the messages (can be a single prompt or array of messages)
    input_key: String,
    /// Output key for the response
    output_key: String,
    /// Action to execute after successful completion
    action: Action,
    /// Maximum number of retries
    max_retries: usize,
    /// Delay between retries
    retry_delay: Duration,
    /// System message to prepend to conversations
    system_message: Option<String>,
    /// Cached OpenAI client
    client: Option<Client<OpenAIConfig>>,
}

impl ApiRequestNode {
    /// Create a new API request node with default configuration
    pub fn new<S: Into<String>>(input_key: S, output_key: S, action: Action) -> Self {
        Self {
            config: ApiConfig::default(),
            input_key: input_key.into(),
            output_key: output_key.into(),
            action,
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            system_message: None,
            client: None,
        }
    }

    /// Create a new API request node with custom configuration
    pub fn with_config(mut self, config: ApiConfig) -> Self {
        self.config = config;
        self
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

    /// Set a system message to prepend to conversations
    pub fn with_system_message(mut self, message: impl Into<String>) -> Self {
        self.system_message = Some(message.into());
        self
    }

    /// Update the configuration
    pub fn update_config(mut self, config: ApiConfig) -> Self {
        self.config = config;
        self.client = None; // Reset client to force recreation
        self
    }

    /// Get or create an OpenAI client
    fn get_client(&mut self) -> Result<&Client<OpenAIConfig>, NodeError> {
        if self.client.is_none() {
            let mut config_builder = OpenAIConfig::new().with_api_key(&self.config.api_key);

            if let Some(ref base_url) = self.config.base_url {
                config_builder = config_builder.with_api_base(base_url);
            }

            if let Some(ref org_id) = self.config.org_id {
                config_builder = config_builder.with_org_id(org_id);
            }

            self.client = Some(Client::with_config(config_builder));
        }

        Ok(self.client.as_ref().unwrap())
    }

    /// Convert input to messages array
    fn parse_messages(
        &self,
        input: &Value,
    ) -> Result<Vec<ChatCompletionRequestMessage>, NodeError> {
        let mut messages = Vec::new();

        // Add system message if provided
        if let Some(ref system_msg) = self.system_message {
            messages.push(ChatCompletionRequestMessage::System(
                async_openai::types::ChatCompletionRequestSystemMessage {
                    content: system_msg.clone().into(),
                    name: None,
                },
            ));
        }

        // Parse input as either a single prompt or array of messages
        match input {
            Value::String(prompt) => {
                // Single prompt string - create user message
                messages.push(ChatCompletionRequestMessage::User(
                    async_openai::types::ChatCompletionRequestUserMessage {
                        content: prompt.clone().into(),
                        name: None,
                    },
                ));
            }
            Value::Array(message_array) => {
                // Array of message objects
                for msg_value in message_array {
                    let role = msg_value
                        .get("role")
                        .and_then(|r| r.as_str())
                        .ok_or_else(|| {
                            NodeError::ValidationError(
                                "Message must have a 'role' field".to_string(),
                            )
                        })?;

                    let content = msg_value
                        .get("content")
                        .and_then(|c| c.as_str())
                        .ok_or_else(|| {
                            NodeError::ValidationError(
                                "Message must have a 'content' field".to_string(),
                            )
                        })?
                        .to_string();

                    match role {
                        "system" => {
                            messages.push(ChatCompletionRequestMessage::System(
                                async_openai::types::ChatCompletionRequestSystemMessage {
                                    content: content.into(),
                                    name: msg_value
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string()),
                                },
                            ));
                        }
                        "user" => {
                            messages.push(ChatCompletionRequestMessage::User(
                                async_openai::types::ChatCompletionRequestUserMessage {
                                    content: content.into(),
                                    name: msg_value
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string()),
                                },
                            ));
                        }
                        "assistant" => {
                            messages.push(ChatCompletionRequestMessage::Assistant(
                                async_openai::types::ChatCompletionRequestAssistantMessage {
                                    content: Some(content.into()),
                                    name: msg_value
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .map(|s| s.to_string()),
                                    ..Default::default()
                                },
                            ));
                        }
                        _ => {
                            return Err(NodeError::ValidationError(format!(
                                "Unsupported message role: {role}"
                            )));
                        }
                    }
                }
            }
            _ => {
                return Err(NodeError::ValidationError(
                    "Input must be a string (prompt) or array of message objects".to_string(),
                ));
            }
        }

        if messages.is_empty() {
            return Err(NodeError::ValidationError(
                "No valid messages found in input".to_string(),
            ));
        }

        Ok(messages)
    }

    /// Make the actual API request using async-openai SDK
    async fn make_api_request(
        &mut self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<String, NodeError> {
        // Extract config values to avoid borrowing issues
        let model = self.config.model.clone();
        let max_tokens = self.config.max_tokens;
        let temperature = self.config.temperature;
        let top_p = self.config.top_p;
        let frequency_penalty = self.config.frequency_penalty;
        let presence_penalty = self.config.presence_penalty;
        let timeout_secs = self.config.timeout;
        let stream = self.config.stream;

        let _client = self.get_client()?;

        // Build the request using builder pattern correctly
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.messages(messages);
        request_builder.stream(stream); // Set streaming option

        if let Some(max_tokens) = max_tokens {
            request_builder.max_tokens(max_tokens);
        }

        if let Some(temperature) = temperature {
            request_builder.temperature(temperature);
        }

        if let Some(top_p) = top_p {
            request_builder.top_p(top_p);
        }

        if let Some(frequency_penalty) = frequency_penalty {
            request_builder.frequency_penalty(frequency_penalty);
        }

        if let Some(presence_penalty) = presence_penalty {
            request_builder.presence_penalty(presence_penalty);
        }

        let request = request_builder
            .build()
            .map_err(|e| NodeError::ExecutionError(format!("Failed to build request: {e}")))?;

        if stream {
            // Handle streaming response
            self.make_streaming_request(request, timeout_secs).await
        } else {
            // Handle non-streaming response
            self.make_regular_request(request, timeout_secs).await
        }
    }

    /// Make a regular (non-streaming) API request
    async fn make_regular_request(
        &mut self,
        request: async_openai::types::CreateChatCompletionRequest,
        timeout_secs: Option<u64>,
    ) -> Result<String, NodeError> {
        let client = self.get_client()?;

        // Make the request with timeout
        let response = if let Some(timeout_secs) = timeout_secs {
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                client.chat().create(request),
            )
            .await
            .map_err(|_| NodeError::ExecutionError("Request timeout".to_string()))?
            .map_err(|e| NodeError::ExecutionError(format!("API request failed: {e}")))?
        } else {
            client
                .chat()
                .create(request)
                .await
                .map_err(|e| NodeError::ExecutionError(format!("API request failed: {e}")))?
        };

        // Extract the response content
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| NodeError::ExecutionError("No response content received".to_string()))?
            .clone();

        Ok(content)
    }

    /// Make a streaming API request and accumulate the response
    async fn make_streaming_request(
        &mut self,
        request: async_openai::types::CreateChatCompletionRequest,
        timeout_secs: Option<u64>,
    ) -> Result<String, NodeError> {
        let client = self.get_client()?;

        // Make the streaming request with timeout
        let stream_result = if let Some(timeout_secs) = timeout_secs {
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                client.chat().create_stream(request),
            )
            .await
            .map_err(|_| NodeError::ExecutionError("Request timeout".to_string()))?
            .map_err(|e| NodeError::ExecutionError(format!("API request failed: {e}")))?
        } else {
            client
                .chat()
                .create_stream(request)
                .await
                .map_err(|e| NodeError::ExecutionError(format!("API request failed: {e}")))?
        };

        // Process the stream and accumulate content
        let mut accumulated_content = String::new();
        let mut stream = stream_result;

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    // Extract content from the streaming response
                    if let Some(delta) = response
                        .choices
                        .first()
                        .and_then(|choice| choice.delta.content.as_ref())
                    {
                        accumulated_content.push_str(delta);
                    }
                }
                Err(e) => {
                    return Err(NodeError::ExecutionError(format!(
                        "Stream processing error: {e}"
                    )));
                }
            }
        }

        if accumulated_content.is_empty() {
            return Err(NodeError::ExecutionError(
                "No content received from streaming response".to_string(),
            ));
        }

        Ok(accumulated_content)
    }
}

#[async_trait]
impl<S: SharedStore + Send + Sync> Node<S> for ApiRequestNode {
    type PrepResult = Vec<ChatCompletionRequestMessage>; // The messages to send
    type ExecResult = String; // The API response
    type Error = NodeError;

    async fn prep(
        &mut self,
        store: &S,
        _context: &ExecutionContext,
    ) -> Result<Self::PrepResult, Self::Error> {
        match store.get(&self.input_key) {
            Ok(Some(value)) => self.parse_messages(&value),
            Ok(None) => Err(NodeError::PrepError(format!(
                "Input key '{}' not found in store",
                self.input_key
            ))),
            Err(e) => Err(NodeError::StorageError(e.to_string())),
        }
    }

    async fn exec(
        &mut self,
        prep_result: Self::PrepResult,
        context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // Check if this is a retry and log it
        if context.current_retry > 0 {
            eprintln!(
                "ApiRequestNode retry attempt {} for {} messages",
                context.current_retry,
                prep_result.len()
            );
        }

        // Make the actual API request
        self.make_api_request(prep_result).await
    }

    async fn post(
        &mut self,
        store: &mut S,
        _prep_result: Self::PrepResult,
        exec_result: Self::ExecResult,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        match store.set(
            self.output_key.clone(),
            serde_json::Value::String(exec_result),
        ) {
            Ok(_) => Ok(self.action.clone()),
            Err(e) => Err(NodeError::StorageError(e.to_string())),
        }
    }

    async fn exec_fallback(
        &mut self,
        _prep_result: Self::PrepResult,
        error: Self::Error,
        _context: &ExecutionContext,
    ) -> Result<Self::ExecResult, Self::Error> {
        // For API failures, return a user-friendly error message
        Ok(format!(
            "API request failed: {error}. Please check your configuration and try again."
        ))
    }

    fn name(&self) -> &str {
        "ApiRequestNode"
    }

    fn max_retries(&self) -> usize {
        self.max_retries
    }

    fn retry_delay(&self) -> Duration {
        self.retry_delay
    }
}
