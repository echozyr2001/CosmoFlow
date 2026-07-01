# Lexer Flow Diagram

The lexer is organized as two layers:

- A main dispatcher flow chooses which token recognizer should run next.
- Token sub-flows consume one token and return control to the dispatcher.

## Main Flow

```text
dispatch
  ├─ whitespace_flow ─┐
  ├─ identifier_flow ─┤
  ├─ integer_flow ────┤
  ├─ string_flow ─────┤
  ├─ operator_flow ───┤
  ├─ delimiter_flow ──┤
  ├─ comment_flow ────┤
  ├─ unknown_flow ────┤
  │                   v
  │          return_to_dispatcher
  │                   |
  └─ end_of_input     └── dispatch
```

`end_of_input` appends the EOF token and returns `complete`. Because the main flow has no route for that action from `end_of_input`, execution naturally terminates.

## Dispatcher Routes

```text
whitespace       -> whitespace_flow
letter or '_'    -> identifier_flow
digit            -> integer_flow
quote            -> string_flow
'//'             -> comment_flow
operator char    -> operator_flow
delimiter char   -> delimiter_flow
end of input     -> end_of_input
other            -> unknown_flow
```

## Token Sub-Flows

Each sub-flow is a focused recognizer:

```text
whitespace:  collect contiguous whitespace
identifier:  collect identifier chars, then classify keyword vs identifier
integer:     collect contiguous digits
string:      collect until the closing quote, including escapes
operator:    recognize one- or two-character operators
delimiter:   consume one delimiter
comment:     consume a line comment
unknown:     consume one unknown character
```

All token sub-flows return `complete` after appending a token. The main flow routes that action to `return_to_dispatcher`, then loops back to `dispatch`.
