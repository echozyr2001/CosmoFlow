# LLM Request Handler

`llm-request-handler` is a queue processor for LLM requests. It is intentionally modeled as an application state machine:

```text
load_config -> enqueue -> pick -> dispatch
                         ^        |
                         |        v
               record_success  record_failure
                         |        |
                         +--------+
                         |
                         v
                       report
```

## What It Demonstrates

- Request queue state with current, completed, and failed requests.
- Provider dispatch through a mock provider or an explicit HTTP provider.
- Application-level retry without adding retry semantics to CosmoFlow core.
- Result recording and reporting after the queue is exhausted.

## Why CosmoFlow Fits

This is not just a single HTTP call. The useful application is the queue: loading configuration, creating work, selecting the next request, dispatching it, recording success or failure, retrying when appropriate, and stopping when no work remains. Those are state transitions, so the flow structure carries real meaning.

## Run With Mock Provider

The mock provider is the default local mode and requires no network access:

```bash
LLM_PROVIDER=mock cargo run -p llm-request-handler
```

If `LLM_PROVIDER` is omitted, the app also uses the mock provider.

## Run With HTTP Provider

HTTP mode is only used when explicitly requested. The app does not silently fall back to mock mode when HTTP configuration is missing.

```bash
LLM_PROVIDER=http \
LLM_API_KEY="..." \
LLM_BASE_URL="https://api.openai.com/v1" \
LLM_MODEL="..." \
cargo run -p llm-request-handler
```

The HTTP provider expects an OpenAI-compatible `chat/completions` response shape.
