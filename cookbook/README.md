# CosmoFlow Cookbook

`cookbook/` contains complete applications built with CosmoFlow. These projects are not API snippets or framework feature demos; each one starts from a meaningful application and uses CosmoFlow to make its state machine explicit.

For focused feature samples, start with [`../cosmoflow/examples`](../cosmoflow/examples).

## Applications

| Project | Application | What it demonstrates |
| --- | --- | --- |
| `better-ls` | CLI directory listing tool | Flow-organized CLI parsing, filesystem work, and formatted output |
| `lexer` | DFA-style tokenizer | Nested flows, dispatcher routing, and shared lexical state |
| `chat-assistant` | Chat loop simulation | Stateful conversation flow and loop termination |
| `llm-request-handler` | LLM request queue processor | Queue state, provider dispatch, result recording, and application-level retry |

## Run

From the workspace root:

```bash
cargo run -p better-ls -- .
cargo run -p lexer
cargo run -p chat-assistant
LLM_PROVIDER=mock cargo run -p llm-request-handler
```

## Adding A Cookbook App

Every direct child of `cookbook/` is a workspace member. A new cookbook directory must be a valid Cargo package with a `Cargo.toml`, otherwise `cargo metadata` and rust-analyzer will fail.

Use cookbook projects for complete applications. Use `cosmoflow/examples/` for small framework feature demonstrations.
