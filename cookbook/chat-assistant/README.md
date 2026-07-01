# Chat Assistant

`chat-assistant` is a small conversation loop application built with CosmoFlow. It simulates input, processing, output, and quit states.

## What It Demonstrates

- A loop-shaped flow for repeated chat turns.
- Conversation state stored in `MemoryStorage`.
- Natural termination through a `quit` route into a final node.
- Async node implementations around stateful application logic.

## Why CosmoFlow Fits

A chat loop is naturally stateful: read the next input, decide whether to continue, produce a response, update conversation history, and eventually quit. CosmoFlow makes that loop and exit state explicit while keeping the assistant behavior in application code.

## Run

From the workspace root:

```bash
cargo run -p chat-assistant
```

The program runs a bounded simulated chat session and prints the execution summary.
