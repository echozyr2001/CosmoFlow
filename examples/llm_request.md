# LLM Request Example

A CosmoFlow example demonstrating real LLM API integration using direct HTTP calls with minimal dependencies.

## Quick Start

### 1. Set Environment Variables

```bash
export LLM_API_KEY="your_api_key"
export LLM_BASE_URL="https://api.openai.com/v1" 
export LLM_MODEL="gpt-3.5-turbo"
```

### 2. Run the Example

```bash
cargo run --bin llm_request -p cosmoflow-examples
```

## Features

- 🌐 **Real API Integration** - Makes actual HTTP calls to LLM APIs
- 🔧 **Environment Configuration** - Credentials loaded from environment variables  
- 🪶 **Lightweight** - Minimal dependencies, no heavy SDK overhead
- 🎯 **Direct Implementation** - Clean code without unnecessary abstractions
- 🔄 **Error Handling** - Comprehensive error handling for API failures
- 📋 **Copyable Patterns** - Reusable code patterns for your own projects

## Supported Providers

### OpenAI
```bash
export LLM_API_KEY="sk-your-openai-key"
export LLM_BASE_URL="https://api.openai.com/v1"
export LLM_MODEL="gpt-3.5-turbo"  # or gpt-4, gpt-4-turbo, etc.
```

### Anthropic Claude
```bash
export LLM_API_KEY="your-anthropic-key"
export LLM_BASE_URL="https://api.anthropic.com/v1"
export LLM_MODEL="claude-3-sonnet-20240229"
```

### Local APIs (Ollama, LocalAI)
```bash
export LLM_API_KEY="not-needed"
export LLM_BASE_URL="http://localhost:11434/v1"
export LLM_MODEL="llama2"
```

## Architecture

The example demonstrates a complete CosmoFlow workflow with these components:

1. **`LlmClient`** - Lightweight HTTP client for API calls
2. **`LlmConfig`** - Configuration management from environment variables
3. **`LlmNode`** - Workflow node that makes actual API calls
4. **Utility Functions** - Request/response handling helpers

### Workflow Steps

```
Start → Setup → LLM API Call → Display Result
```

1. **Start**: Initialize workflow with logging
2. **Setup**: Load configuration and prepare prompt
3. **LLM**: Make HTTP request to LLM API
4. **Display**: Show AI response

## Customization

### Change the Prompt

Edit the prompt in `DataSetupNode::post()`:

```rust
store.set("user_prompt".to_string(), "Your custom prompt").unwrap();
```

### Add Request Parameters

Extend `create_chat_request()` for additional options:

```rust
pub fn create_chat_request(model: &str, messages: Vec<Value>, temperature: f32) -> Value {
    json!({
        "model": model,
        "messages": messages,
        "temperature": temperature,
        "max_tokens": 1000
    })
}
```

### Support New Providers

1. Adjust request format in `create_chat_request()`
2. Modify response parsing in `extract_content()`
3. Update authentication in `LlmNode`

## Testing

```bash
cargo test --bin llm_request -p cosmoflow-examples
```

Tests cover:
- Utility function validation
- Configuration loading  
- HTTP client construction

## Troubleshooting

### Environment Variables Not Set
```
❌ Configuration Error: LLM_API_KEY environment variable not set
```
**Solution**: Export all required environment variables in your shell.

### API Request Failed
```
❌ LLM API request failed: ...
```
**Solutions**:
- Verify API key is valid and has sufficient credits
- Check base URL is correct for your provider
- Ensure model name is supported

### Response Parsing Failed
```
❌ Failed to extract content from LLM response
```
**Solution**: The API response format may differ. Check provider documentation and adjust `extract_content()` function.

## Security

- ✅ Never commit API keys to version control
- ✅ Use environment variables for credentials
- ✅ API keys are masked in logs (only first 10 chars shown)
