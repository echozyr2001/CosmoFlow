//! LLM request queue processor built with CosmoFlow.
//!
//! The application is a real state machine: load configuration, enqueue work,
//! pick the next request, dispatch it, record success/failure, and continue
//! until the queue is empty. Retry is application logic, not a core framework
//! feature.

use async_trait::async_trait;
use cosmoflow::{
    action::Action,
    flow::FlowBuilder,
    node::{Node, NodeContext},
};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::error::Error;
use std::fmt;

const MAX_ATTEMPTS: u8 = 2;

#[derive(Debug)]
struct AppError(String);

impl AppError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for AppError {}

#[derive(Debug, Clone)]
enum ProviderConfig {
    Mock,
    Http {
        api_key: String,
        base_url: String,
        model: String,
    },
}

impl ProviderConfig {
    fn from_env() -> Result<Self, AppError> {
        match env::var("LLM_PROVIDER").as_deref() {
            Ok("mock") | Err(env::VarError::NotPresent) => Ok(Self::Mock),
            Ok("http") => Ok(Self::Http {
                api_key: required_env("LLM_API_KEY")?,
                base_url: required_env("LLM_BASE_URL")?,
                model: required_env("LLM_MODEL")?,
            }),
            Ok(provider) => Err(AppError::new(format!(
                "unsupported LLM_PROVIDER '{provider}', expected 'mock' or 'http'"
            ))),
            Err(error) => Err(AppError::new(format!(
                "failed to read LLM_PROVIDER: {error}"
            ))),
        }
    }
}

fn required_env(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::new(format!("{key} is required when LLM_PROVIDER=http")))
}

#[derive(Debug, Clone)]
struct LlmRequest {
    id: u64,
    prompt: String,
    attempts: u8,
}

#[derive(Debug, Clone)]
struct CompletedRequest {
    id: u64,
    prompt: String,
    response: String,
    attempts: u8,
}

#[derive(Debug, Clone)]
struct FailedRequest {
    id: u64,
    prompt: String,
    error: String,
    attempts: u8,
}

#[derive(Debug, Clone)]
struct CurrentRequest {
    request: LlmRequest,
    response: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Default)]
struct QueueState {
    provider: Option<ProviderConfig>,
    queue: VecDeque<LlmRequest>,
    current: Option<CurrentRequest>,
    completed: Vec<CompletedRequest>,
    failed: Vec<FailedRequest>,
}

impl QueueState {
    fn provider(&self) -> Result<ProviderConfig, AppError> {
        self.provider
            .clone()
            .ok_or_else(|| AppError::new("provider config has not been loaded"))
    }

    fn current(&self) -> Result<CurrentRequest, AppError> {
        self.current
            .clone()
            .ok_or_else(|| AppError::new("no current request selected"))
    }
}

#[derive(Debug, Clone)]
enum DispatchOutcome {
    Success(String),
    Failure(String),
}

struct LoadConfigNode;

#[async_trait]
impl Node<QueueState> for LoadConfigNode {
    type Prep = ();
    type Output = ProviderConfig;
    type Error = AppError;

    async fn prep(
        &mut self,
        _state: &QueueState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        ProviderConfig::from_env()
    }

    async fn post(
        &mut self,
        state: &mut QueueState,
        _prep: Self::Prep,
        provider: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state.provider = Some(provider);
        Ok(Action::new("enqueue"))
    }
}

struct EnqueueRequestsNode;

#[async_trait]
impl Node<QueueState> for EnqueueRequestsNode {
    type Prep = ();
    type Output = Vec<LlmRequest>;
    type Error = AppError;

    async fn prep(
        &mut self,
        _state: &QueueState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(vec![
            LlmRequest {
                id: 1,
                prompt: "Summarize CosmoFlow in one sentence.".to_string(),
                attempts: 0,
            },
            LlmRequest {
                id: 2,
                prompt: "Name one benefit of modeling apps as state machines.".to_string(),
                attempts: 0,
            },
            LlmRequest {
                id: 3,
                prompt: "fail once: demonstrate application-level retry".to_string(),
                attempts: 0,
            },
        ])
    }

    async fn post(
        &mut self,
        state: &mut QueueState,
        _prep: Self::Prep,
        requests: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state.queue.extend(requests);
        Ok(Action::new("pick"))
    }
}

struct PickNextNode;

#[async_trait]
impl Node<QueueState> for PickNextNode {
    type Prep = ();
    type Output = ();
    type Error = AppError;

    async fn prep(
        &mut self,
        _state: &QueueState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    async fn post(
        &mut self,
        state: &mut QueueState,
        _prep: Self::Prep,
        _ignored: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        match state.queue.pop_front() {
            Some(request) => {
                state.current = Some(CurrentRequest {
                    request,
                    response: None,
                    error: None,
                });
                Ok(Action::new("dispatch"))
            }
            None => Ok(Action::new("finish")),
        }
    }
}

struct DispatchNode;

#[async_trait]
impl Node<QueueState> for DispatchNode {
    type Prep = (ProviderConfig, LlmRequest);
    type Output = DispatchOutcome;
    type Error = AppError;

    async fn prep(
        &mut self,
        state: &QueueState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        let provider = state.provider()?;
        let current = state.current()?;
        Ok((provider, current.request))
    }

    async fn exec(
        &mut self,
        (provider, request): &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        match provider {
            ProviderConfig::Mock => Ok(dispatch_mock(request)),
            ProviderConfig::Http {
                api_key,
                base_url,
                model,
            } => dispatch_http(api_key, base_url, model, request).await,
        }
    }

    async fn post(
        &mut self,
        state: &mut QueueState,
        _prep: Self::Prep,
        outcome: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        let current = state
            .current
            .as_mut()
            .ok_or_else(|| AppError::new("no current request available for dispatch outcome"))?;

        match outcome {
            DispatchOutcome::Success(response) => {
                current.response = Some(response);
                current.error = None;
                Ok(Action::new("success"))
            }
            DispatchOutcome::Failure(error) => {
                current.response = None;
                current.error = Some(error);
                Ok(Action::new("failure"))
            }
        }
    }
}

fn dispatch_mock(request: &LlmRequest) -> DispatchOutcome {
    if request.prompt.contains("fail once") && request.attempts == 0 {
        return DispatchOutcome::Failure("mock provider intentionally failed once".to_string());
    }

    DispatchOutcome::Success(format!("mock response for request {}", request.id))
}

async fn dispatch_http(
    api_key: &str,
    base_url: &str,
    model: &str,
    request: &LlmRequest,
) -> Result<DispatchOutcome, AppError> {
    let client = LlmClient::new(base_url)
        .with_header("Authorization", format!("Bearer {api_key}"))
        .with_header("Content-Type", "application/json");

    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": request.prompt
            }
        ]
    });

    match client.post("chat/completions", &body).await {
        Ok(response) => match extract_content(&response) {
            Some(content) => Ok(DispatchOutcome::Success(content)),
            None => Ok(DispatchOutcome::Failure(
                "HTTP provider response did not contain choices[0].message.content".to_string(),
            )),
        },
        Err(error) => Ok(DispatchOutcome::Failure(error.to_string())),
    }
}

struct RecordSuccessNode;

#[async_trait]
impl Node<QueueState> for RecordSuccessNode {
    type Prep = CurrentRequest;
    type Output = CompletedRequest;
    type Error = AppError;

    async fn prep(
        &mut self,
        state: &QueueState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        state.current()
    }

    async fn exec(
        &mut self,
        current: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let response = current
            .response
            .clone()
            .ok_or_else(|| AppError::new("successful request missing response"))?;

        Ok(CompletedRequest {
            id: current.request.id,
            prompt: current.request.prompt.clone(),
            response,
            attempts: current.request.attempts + 1,
        })
    }

    async fn post(
        &mut self,
        state: &mut QueueState,
        _prep: Self::Prep,
        completed: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        state.completed.push(completed);
        state.current = None;
        Ok(Action::new("pick"))
    }
}

struct RecordFailureNode;

#[async_trait]
impl Node<QueueState> for RecordFailureNode {
    type Prep = CurrentRequest;
    type Output = Result<LlmRequest, FailedRequest>;
    type Error = AppError;

    async fn prep(
        &mut self,
        state: &QueueState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        state.current()
    }

    async fn exec(
        &mut self,
        current: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let error = current
            .error
            .clone()
            .ok_or_else(|| AppError::new("failed request missing error"))?;
        let attempts = current.request.attempts + 1;

        if attempts < MAX_ATTEMPTS {
            let mut retry = current.request.clone();
            retry.attempts = attempts;
            Ok(Ok(retry))
        } else {
            Ok(Err(FailedRequest {
                id: current.request.id,
                prompt: current.request.prompt.clone(),
                error,
                attempts,
            }))
        }
    }

    async fn post(
        &mut self,
        state: &mut QueueState,
        _prep: Self::Prep,
        outcome: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        match outcome {
            Ok(retry) => state.queue.push_back(retry),
            Err(failed) => state.failed.push(failed),
        }
        state.current = None;
        Ok(Action::new("pick"))
    }
}

struct ReportNode;

#[async_trait]
impl Node<QueueState> for ReportNode {
    type Prep = (Vec<CompletedRequest>, Vec<FailedRequest>);
    type Output = String;
    type Error = AppError;

    async fn prep(
        &mut self,
        state: &QueueState,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok((state.completed.clone(), state.failed.clone()))
    }

    async fn exec(
        &mut self,
        (completed, failed): &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let mut report = String::new();
        report.push_str("LLM queue processing summary\n");
        report.push_str("============================\n");
        report.push_str(&format!("completed: {}\n", completed.len()));
        report.push_str(&format!("failed: {}\n\n", failed.len()));

        for item in completed {
            report.push_str(&format!(
                "ok #{id} attempts={attempts}: {prompt} -> {response}\n",
                id = item.id,
                attempts = item.attempts,
                prompt = item.prompt,
                response = item.response
            ));
        }

        for item in failed {
            report.push_str(&format!(
                "failed #{id} attempts={attempts}: {prompt} -> {error}\n",
                id = item.id,
                attempts = item.attempts,
                prompt = item.prompt,
                error = item.error
            ));
        }

        Ok(report)
    }

    async fn post(
        &mut self,
        _state: &mut QueueState,
        _prep: Self::Prep,
        report: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        println!("{report}");
        Ok(Action::new("complete"))
    }
}

/// Lightweight HTTP client for OpenAI-compatible chat completion endpoints.
struct LlmClient {
    client: reqwest::Client,
    base_url: String,
    headers: HashMap<String, String>,
}

impl LlmClient {
    fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            headers: HashMap::new(),
        }
    }

    fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    async fn post(&self, endpoint: &str, body: &Value) -> Result<Value, reqwest::Error> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );
        let mut request = self.client.post(&url).json(body);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request.send().await?.json().await
    }
}

fn extract_content(response: &Value) -> Option<String> {
    response
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()
        .map(str::to_string)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut flow = FlowBuilder::new()
        .node("load_config", LoadConfigNode)
        .node("enqueue", EnqueueRequestsNode)
        .node("pick", PickNextNode)
        .node("dispatch", DispatchNode)
        .node("record_success", RecordSuccessNode)
        .node("record_failure", RecordFailureNode)
        .node("report", ReportNode)
        .route("load_config", "enqueue", "enqueue")
        .route("enqueue", "pick", "pick")
        .route("pick", "dispatch", "dispatch")
        .route("pick", "finish", "report")
        .route("dispatch", "success", "record_success")
        .route("dispatch", "failure", "record_failure")
        .route("record_success", "pick", "pick")
        .route("record_failure", "pick", "pick")
        .build()?;

    let mut state = QueueState::default();
    let execution = flow.run_recorded(&mut state).await?;

    println!("execution path: {:?}", execution.path);
    println!("final action: {}", execution.final_action);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_content_reads_chat_completion_shape() {
        let response = json!({
            "choices": [{
                "message": {
                    "content": "hello"
                }
            }]
        });

        assert_eq!(extract_content(&response), Some("hello".to_string()));
    }

    #[test]
    fn mock_provider_fails_marked_request_once() {
        let first = LlmRequest {
            id: 7,
            prompt: "fail once".to_string(),
            attempts: 0,
        };
        let retry = LlmRequest {
            attempts: 1,
            ..first.clone()
        };

        assert!(matches!(
            dispatch_mock(&first),
            DispatchOutcome::Failure(message) if message.contains("failed once")
        ));
        assert!(matches!(
            dispatch_mock(&retry),
            DispatchOutcome::Success(message) if message.contains("request 7")
        ));
    }
}
