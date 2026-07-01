# CosmoFlow Lexer

`lexer` is a DFA-style tokenizer implemented as a CosmoFlow application. It uses a dispatcher flow to choose token-specific nested flows.

## What It Demonstrates

- Mapping DFA states to CosmoFlow nodes and routes.
- Using built `Flow<MemoryStorage>` values as nested flow nodes.
- Keeping lexical state in `MemoryStorage`.
- Natural termination when the end-of-input node returns `complete` without a matching route.

## Architecture

```text
main dispatcher flow
  dispatch
    -> whitespace_flow
    -> identifier_flow
    -> integer_flow
    -> string_flow
    -> operator_flow
    -> delimiter_flow
    -> comment_flow
    -> unknown_flow
    -> end_of_input
```

Each token flow consumes one token and returns `complete`. The main flow routes that action to `return_to_dispatcher`, which then routes back to `dispatch`. The `end_of_input` node appends an EOF token and naturally terminates the flow.

The two-layer structure is intentional:

- The main dispatcher flow owns token selection and loop control.
- Token sub-flows own token-specific scanning logic.

That keeps the application close to a DFA while still making each token recognizer independently understandable.

## Run

From the workspace root:

```bash
cargo run -p lexer
```

The binary tokenizes two sample inputs and prints the generated tokens.

## Token Types

- `Identifier`
- `Keyword`
- `Integer`
- `String`
- `Operator`
- `Delimiter`
- `Whitespace`
- `Comment`
- `Unknown`
- `EndOfInput`
