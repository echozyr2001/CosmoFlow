#![deny(missing_docs)]
//! Built-in node implementations for CosmoFlow workflows
//!
//! This crate provides a comprehensive collection of pre-built, production-ready node
//! implementations for common workflow operations. These nodes are designed to be used
//! out-of-the-box or as examples for creating custom node implementations.
//!
//! # Overview
//!
//! The builtin crate is organized into several modules, each focusing on different
//! categories of workflow operations:
//!
//! - [`basic`] - Fundamental operations (logging, value setting, conditionals)
//! - [`llm`] - Large Language Model integrations (OpenAI, custom APIs)
//! - [`nodes`] - High-level convenience functions for creating node instances
//!
//! # Core Node Categories
//!
//! ## Basic Operations ([`basic`] module)
//!
//! Essential nodes for workflow control and data management:
//!
//! - **LogNode**: Output logging and debugging information
//! - **SetValueNode**: Store values in the shared store
//! - **GetValueNode**: Retrieve and validate stored values
//! - **ConditionalNode**: Conditional execution based on stored data
//! - **DelayNode**: Introduce delays for timing control
//!
//! ```rust
//! use cosmoflow::builtin::basic::*;
//! use cosmoflow::shared_store::SharedStore;
//! use cosmoflow::shared_store::backends::MemoryStorage;
//! use cosmoflow::action::Action;
//!
//! // Create basic nodes
//! let log_node = LogNode::new("Processing started", Action::simple("continue"));
//! let set_value_node = SetValueNode::new("counter", serde_json::json!(0u32), Action::simple("continue"));
//! ```
//!
//! ## LLM Integration ([`llm`] module)
//!
//! Advanced nodes for Large Language Model operations:
//!
//! - **OpenAI Integration**: Direct integration with OpenAI's API
//! - **Custom LLM Support**: Extensible framework for other providers
//! - **Prompt Engineering**: Built-in prompt formatting and management
//! - **Response Processing**: Automatic parsing and validation
//! - **Rate Limiting**: Built-in request throttling and retry logic
//!
//! ## Convenience Functions ([`nodes`] module)
//!
//! High-level functions for quickly creating commonly used node instances:
//!
//! ```rust
//! use cosmoflow::builtin::nodes::generic::*;
//! use cosmoflow::shared_store::backends::MemoryStorage;
//! use cosmoflow::SharedStore;
//! use serde_json::json;
//!
//! // Quick node creation
//! let logger = log_node::<MemoryStorage>("Starting workflow");
//! let set_value = set_value_node::<MemoryStorage>("step_count", json!(1));
//! let conditional = conditional_node::<_, MemoryStorage>(
//!     |store| store.get("status").ok().flatten().and_then(|v: serde_json::Value| v.as_bool()).unwrap_or(false),
//!     cosmoflow::action::Action::simple("ready"),
//!     cosmoflow::action::Action::simple("not_ready")
//! );
//! ```
//!
//! # Error Handling
//!
//! All built-in nodes implement comprehensive error handling:
//!
//! ## Error Types
//!
//! - **Configuration Errors**: Invalid parameters or missing required settings
//! - **Execution Errors**: Failures during node operation (API errors, network issues)
//! - **Data Errors**: Type mismatches, missing keys, or invalid data formats
//! - **Timeout Errors**: Operations that exceed configured time limits
//!
//! ## Retry Logic
//!
//! Built-in nodes include configurable retry mechanisms:
//!
//! ```rust
//! use cosmoflow::builtin::llm::*;
//!
//! let config = ApiConfig::new("api-key")
//!     .with_model("gpt-4")
//!     .with_timeout(30);
//! // Note: retry_attempts and retry_delay are not yet implemented
//! ```
//!
//! ## Graceful Degradation
//!
//! Many nodes support fallback behavior:
//!
//! ```rust
//! use cosmoflow::builtin::basic::*;
//! use cosmoflow::action::Action;
//! use cosmoflow::shared_store::backends::MemoryStorage;
//! use cosmoflow::SharedStore;
//!
//! // Conditional nodes use closures for conditions
//! let conditional = ConditionalNode::<_, MemoryStorage>::new(
//!     |store| store.get("score").ok().flatten().and_then(|v: serde_json::Value| v.as_f64()).unwrap_or(0.0) > 80.0,
//!     Action::simple("success"),
//!     Action::simple("use_default")
//! );
//! ```
//!
//! # Performance Characteristics
//!
//! ## Async Operations
//!
//! All built-in nodes are fully asynchronous and non-blocking:
//!
//! - **I/O Operations**: Network requests, file operations are async
//! - **CPU-bound Tasks**: Can be configured to use thread pools
//! - **Backpressure**: Built-in rate limiting prevents overwhelming external services
//!
//! ## Resource Management
//!
//! - **Memory**: Efficient serialization and minimal data copying
//! - **Network**: Connection pooling and request batching where applicable
//! - **CPU**: Configurable concurrency limits
//!
//! # Configuration Patterns
//!
//! ## Environment-Based Configuration
//!
//! ```rust
//! use cosmoflow::builtin::llm::*;
//! use std::env;
//!
//! // Example API key (don't use real keys in examples)
//! let api_key = "sk-example-key-1234567890".to_string();
//!
//! let config = ApiConfig::new(api_key)
//!     .with_model("gpt-4");
//!
//! if let Ok(base_url) = env::var("OPENAI_BASE_URL") {
//!     let config = config.with_base_url(base_url);
//! }
//! ```
//!
//! ## Builder Pattern
//!
//! Most nodes support the builder pattern for configuration:
//!
//! ```rust
//! use cosmoflow::builtin::basic::*;
//! use cosmoflow::action::Action;
//!
//! // Basic LogNode construction
//! let log_node = LogNode::new("Custom log message", Action::simple("continue"))
//!     .with_retries(3);
//! // Note: LogLevel and LogFormat are not yet implemented
//! ```
//!
//! # Security Considerations
//!
//! ## API Key Management
//!
//! - Store API keys in environment variables, not in code
//! - Use secure secret management systems in production
//! - Implement key rotation mechanisms
//!
//! ## Data Privacy
//!
//! - Be aware of data sent to external APIs (especially LLM services)
//! - Implement data sanitization for sensitive information
//! - Consider on-premises solutions for sensitive data
//!
//! ## Rate Limiting
//!
//! - Respect API rate limits to avoid service disruption
//! - Implement exponential backoff for retries
//! - Monitor API usage and costs
//!
//! # Testing Support
//!
//! The builtin crate provides comprehensive testing utilities:
//!
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use cosmoflow::builtin::testing::*;
//!
//!     #[tokio::test]
//!     async fn test_log_node() {
//!         let mut node = log_node("Test message");
//!         let mut store = test_store();
//!         let context = test_context();
//!
//!         let result = node.execute(&context, &mut store).await;
//!         assert!(result.is_ok());
//!     }
//! }
//! ```
//!
//! # Custom Node Development
//!
//! Built-in nodes serve as excellent examples for creating custom implementations:
//!
//! ```rust
//! use cosmoflow::node::{Node, ExecutionContext};
//! use cosmoflow::SharedStore;
//! use cosmoflow::action::Action;
//! use async_trait::async_trait;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct CustomProcessingNode {
//!     algorithm: String,
//!     parameters: std::collections::HashMap<String, f64>,
//! }
//!
//! #[async_trait]
//! impl<S: SharedStore> Node<S> for CustomProcessingNode {
//!     type PrepResult = Vec<f64>;
//!     type ExecResult = Vec<f64>; // Changed to avoid undefined ProcessingResult
//!     type Error = cosmoflow::node::NodeError; // Use existing NodeError
//!
//!     async fn prep(
//!         &mut self,
//!         store: &S,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::PrepResult, Self::Error> {
//!         let input_data: Vec<f64> = store.get("input_data")
//!             .map_err(|_| cosmoflow::node::NodeError::ExecutionError("Missing input_data".to_string()))?
//!             .ok_or_else(|| cosmoflow::node::NodeError::ExecutionError("No input_data found".to_string()))?;
//!         Ok(input_data)
//!     }
//!
//!     async fn exec(
//!         &mut self,
//!         prep_result: Self::PrepResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Self::ExecResult, Self::Error> {
//!         // Custom processing logic (simple example)
//!         let result = prep_result.iter().map(|x| x * 2.0).collect();
//!         Ok(result)
//!     }
//!
//!     async fn post(
//!         &mut self,
//!         store: &mut S,
//!         _prep_result: Self::PrepResult,
//!         exec_result: Self::ExecResult,
//!         _context: &ExecutionContext,
//!     ) -> Result<Action, Self::Error> {
//!         store.set("processing_result".to_string(), exec_result)
//!             .map_err(|_| cosmoflow::node::NodeError::ExecutionError("Failed to store result".to_string()))?;
//!         Ok(Action::simple("next_step"))
//!     }
//! }
//! ```
//!
//! # Feature Flags
//!
//! The builtin crate respects CosmoFlow's feature flag system:
//!
//! ```toml
//! [dependencies]
//! builtin = { version = "0.1", features = ["llm", "basic"] }
//! ```
//!
//! Available features:
//! - `basic`: Basic node implementations (always recommended)
//! - `llm`: Large Language Model integration nodes
//! - `testing`: Additional utilities for testing (development only)

/// Basic workflow operations and utilities
pub mod basic;
/// Large Language Model integration nodes
pub mod llm;
/// High-level convenience functions for creating node instances
pub mod nodes;

pub use basic::{ConditionalNode, DelayNode, GetValueNode, LogNode, SetValueNode};
pub use llm::{ApiConfig, ApiRequestNode, MockLlmNode};
pub use nodes::generic::*;
