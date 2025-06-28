# Lexer DFA State Transition Diagram (Nested Flow Architecture)

## Overall Nested Flow Architecture

```
                        ┌─────────────────┐
                        │   MAIN FLOW     │
                        │   (LEXER)       │
                        └─────────┬───────┘
                                  │
                                  ▼
                        ┌─────────────────┐
                        │   DISPATCHER    │◄─────────────┐
                        │     NODE        │              │
                        └─────────┬───────┘              │
                                  │                      │
                    ┌─────────────┼─────────────────┐    │
                    │             │                 │    │
                    ▼             ▼                 ▼    │
          ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
          │  WHITESPACE     │ │  IDENTIFIER     │ │   INTEGER       │
          │   SUB-FLOW      │ │   SUB-FLOW      │ │   SUB-FLOW      │
          └─────────┬───────┘ └─────────┬───────┘ └─────────┬───────┘
                    │                   │                   │
                    └───────────────────┼───────────────────┘
                                        │
                        ┌───────────────┼───────────────┐
                        │               │               │
                        ▼               ▼               ▼
              ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
              │   STRING        │ │  OPERATOR       │ │  DELIMITER      │
              │   SUB-FLOW      │ │   SUB-FLOW      │ │   SUB-FLOW      │
              └─────────┬───────┘ └─────────┬───────┘ └─────────┬───────┘
                        │                   │                   │
                        └───────────────────┼───────────────────┘
                                            │
                        ┌───────────────────┼───────────────────┐
                        │                   │                   │
                        ▼                   ▼                   ▼
              ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
              │   COMMENT       │ │   UNKNOWN       │ │ END_OF_INPUT    │
              │   SUB-FLOW      │ │   SUB-FLOW      │ │     NODE        │
              └─────────┬───────┘ └─────────┬───────┘ └─────────┬───────┘
                        │                   │                   │
                        └───────────────────┼───────────────────┘
                                            │                   │
                                            ▼                   ▼
                                  ┌─────────────────┐ ┌─────────────────┐
                                  │ RETURN_TO_      │ │   TERMINAL      │
                                  │ DISPATCHER      │ │   (COMPLETE)    │
                                  └─────────┬───────┘ └─────────────────┘
                                            │
                                            └─────────────────────────┘
```

## Dispatcher Logic

The dispatcher analyzes the current character and routes to the appropriate sub-flow:

```
Current Character Analysis:
├── Whitespace (' ', '\t', '\n', '\r') → whitespace_flow
├── Letter or '_' → identifier_flow
├── Digit → integer_flow
├── Quote ('"') → string_flow
├── '/' (need to check next char for '//' comment) → comment_flow or operator_flow
├── Other Operators (+, -, *, =, <, >, !) → operator_flow
├── Delimiters ((, ), {, }, [, ], ;, ,, .) → delimiter_flow
├── End of Input → end_of_input
└── Others → unknown_flow
```
## Detailed Sub-flow DFAs

### 1. WHITESPACE Sub-flow DFA
```
START ──[whitespace]──► COLLECT ──[whitespace]──► COLLECT
                           │                        │
                           └──[other]──────────────► COMPLETE
```

### 2. IDENTIFIER Sub-flow DFA
```
START ──[a-zA-Z_]──► COLLECT ──[a-zA-Z0-9_]──► COLLECT
                       │                        │
                       └──[other]──────────────► CLASSIFY ──► COMPLETE
                                                   │
                                                   ├── keyword → Keyword Token
                                                   └── other → Identifier Token
```

### 3. INTEGER Sub-flow DFA
```
START ──[0-9]──► COLLECT ──[0-9]──► COLLECT
                   │                  │
                   └──[other]────────► COMPLETE
```

### 4. STRING Sub-flow DFA
```
START ──["]──► COLLECT ──[char≠"]──► COLLECT ──["]──► COMPLETE
                 │                     │       
                 │                     └──[\]──► ESCAPE ──[any]──┐
                 │                                               │
                 └─────────────────────────────────────────────┘
```

### 5. OPERATOR Sub-flow DFA
```
START ──[op_char]──► CHECK_SECOND ──[valid_second_char]──► COMPLETE (two-char op)
                         │                                    │
                         │         Examples: == != <= >= ++  │
                         │                   -- && ||        │
                         └──[other]────────────────────────► COMPLETE (single-char op)
```

### 6. DELIMITER Sub-flow DFA
```
START ──[delimiter]──► COLLECT ──► COMPLETE
```

### 7. COMMENT Sub-flow DFA
```
START ──[/]──► FIRST_SLASH ──[/]──► COLLECT_LINE ──[char≠\n]──► COLLECT_LINE
                   │                      │                        │
                   │                      └──[\n or EOF]──────────► COMPLETE
                   └──[other]──► ERROR (Invalid comment start)
```

### 8. UNKNOWN Sub-flow DFA
```
START ──[any]──► COLLECT ──► COMPLETE
```

## Main Flow Execution Cycle

```
DISPATCHER → SUB-FLOW → RETURN_TO_DISPATCHER → DISPATCHER → ...
     │                                              │
     └──[end_of_input]──► END_OF_INPUT ──► COMPLETE
```

## Nested Flow Benefits

### Modularity
- Each token type is handled by an independent sub-flow
- Sub-flows can be developed, tested, and maintained separately
- Easy to add new token types without modifying existing flows

### Composability
- Sub-flows can be reused in different contexts
- Main flow orchestrates sub-flows without knowing their internal structure
- Clear separation between dispatcher logic and token processing

### Extensibility
- New token types can be added by creating new sub-flows
- Existing sub-flows can be enhanced without affecting others
- Complex token processing can be implemented with multi-stage sub-flows

### Maintainability
- Clear flow structure and execution paths
- Isolated concerns and responsibilities
- Easy debugging and testing at the sub-flow level

## Sub-flow Return Handling

Each sub-flow completes and returns control to the dispatcher, which continues processing the next character until EOF is reached.

```
Sub-flow → "complete" → ReturnToDispatcherNode → "dispatch" → DispatcherNode
```

This creates a natural loop that processes the entire input character by character, with each token type handled by its specialized sub-flow.
