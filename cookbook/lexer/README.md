# CosmoFlow Lexer with Nested Flows (DFA-based)

This is a lexer built with the CosmoFlow workflow engine, showcasing how to map Deterministic Finite Automata (DFA) concepts to CosmoFlow's nested flow functionality. This implementation demonstrates advanced workflow composition using sub-flows for maximum modularity and extensibility.

## Design Philosophy

Traditional lexers typically use DFAs (Deterministic Finite Automata) for lexical analysis:
- Each state represents a specific stage in the parsing process
- State transitions are based on input characters
- Final states produce tokens

Our design maps this concept to CosmoFlow using **nested flows**:
- **DFA States** → **CosmoFlow Sub-flows**
- **State Transitions** → **CosmoFlow Actions and Routes**
- **Input Processing** → **Lexical Context in Shared Store**

## Architecture Design

### Core Components

#### 1. Token Types (TokenType)
```rust
pub enum TokenType {
    Identifier,    // Identifiers (variable names, function names, etc.)
    Integer,       // Integer literals
    String,        // String literals (enclosed in quotes)
    Keyword,       // Keywords (if, else, while, etc.)
    Operator,      // Operators (+, -, *, /, =, ==, etc.)
    Delimiter,     // Delimiters (, ), {, }, [, ], ;, ,, etc.)
    Whitespace,    // Whitespace (spaces, tabs, newlines)
    Comment,       // Comments
    Unknown,       // Unknown/invalid token
    EndOfInput,    // End of input
}
```

#### 2. Lexical Analysis Context (LexerContext)
State information stored in CosmoFlow's shared store:
```rust
pub struct LexerContext {
    pub input: String,        // Input string
    pub position: usize,      // Current position
    pub line: u32,           // Current line number
    pub column: u32,         // Current column number
    pub tokens: Vec<Token>,  // Generated token list
}
```

### Nested Flow Architecture

The lexer uses a **dispatcher-based architecture** with **dedicated sub-flows** for each token type:

#### 1. Main Flow Structure
```
Dispatcher → Sub-flows → Return to Dispatcher → Repeat → End
```

#### 2. Dispatcher Node (DispatcherNode)
- **Function**: Analyzes the current character and determines which token-specific sub-flow to invoke
- **Routing Logic**:
  - Whitespace characters → `whitespace_flow`
  - Letters/underscore → `identifier_flow`
  - Digits → `integer_flow`
  - Quotes → `string_flow`
  - Operators → `operator_flow`
  - Delimiters → `delimiter_flow`
  - `//` → `comment_flow`
  - Others → `unknown_flow`
  - End of input → `end_of_input`

#### 3. Token-Specific Sub-flows
Each token type is handled by a dedicated sub-flow:

- **`whitespace_flow`**: Processes consecutive whitespace characters
- **`identifier_flow`**: Processes identifiers and keywords (with classification)
- **`integer_flow`**: Processes integer literals
- **`string_flow`**: Processes string literals with escape sequences
- **`operator_flow`**: Processes operators (supports multi-character operators like `==`, `!=`)
- **`delimiter_flow`**: Processes delimiters
- **`comment_flow`**: Processes line comments
- **`unknown_flow`**: Processes unknown characters

#### 4. Sub-flow Internal Structure
Each sub-flow is a complete Flow with its own nodes and routes:

```rust
// Example: identifier_flow
FlowBuilder::new()
    .start_node("collect")
    .node("collect", IdentifierCollectorNode)     // Collects characters
    .node("classify", IdentifierClassifierNode)   // Classifies as keyword/identifier
    .route("collect", "classify", "classify")
    .terminal_route("classify", "complete")
    .build()
```

### Workflow Routing Design

```
Dispatcher → Sub-flow → Return → Dispatcher → ... → End
```

#### Main Flow Routes:
```rust
// From dispatcher to sub-flows
.route("dispatch", "whitespace_flow", "whitespace_flow")
.route("dispatch", "identifier_flow", "identifier_flow")
.route("dispatch", "integer_flow", "integer_flow")
// ... other routes

// From sub-flows back to dispatcher
.route("whitespace_flow", "complete", "return_to_dispatcher")
.route("identifier_flow", "complete", "return_to_dispatcher")
// ... other returns

// Continuation loop
.route("return_to_dispatcher", "dispatch", "dispatch")
.terminal_route("end_of_input", "complete")
// Terminal route
.terminal_route("end_of_input", "complete")
```

## Usage Example

```rust
use lexer::CosmoFlowLexer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer = CosmoFlowLexer::new();
    
    let input = r#"
        fn main() {
            let x = 42;
            if x > 0 {
                println!("Hello, world!");
            }
        }
    "#;

    let tokens = lexer.tokenize(input)?;
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{:3}: {:?}", i, token);
    }
    
    Ok(())
}
```

## Sample Output

```
  0: Token { token_type: Whitespace, lexeme: "\n        ", line: 2, column: 9 }
  1: Token { token_type: Keyword, lexeme: "fn", line: 2, column: 11 }
  2: Token { token_type: Whitespace, lexeme: " ", line: 2, column: 12 }
  3: Token { token_type: Identifier, lexeme: "main", line: 2, column: 16 }
  4: Token { token_type: Delimiter, lexeme: "(", line: 2, column: 17 }
  5: Token { token_type: Delimiter, lexeme: ")", line: 2, column: 18 }
  ...
```

## Technical Advantages

### 1. Modular Design with Nested Flows
- Each token type's processing logic is encapsulated in a dedicated sub-flow
- Easy to extend with new token types
- Clear separation of concerns
- State transition logic is explicit and maintainable

### 2. Type Safety
- Leverages Rust's type system to ensure correctness of state transitions
- CosmoFlow's shared store ensures data consistency
- Compile-time guarantees for workflow structure

### 3. Extensibility
- Easy to add new token types and corresponding sub-flows
- Support for complex state transition logic
- Can add conditional routing for more sophisticated syntax rules
- Sub-flows can be independently tested and developed

### 4. Observability
- CosmoFlow provides execution path tracking
- Easy to debug and analyze the lexical analysis process
- Can record processing time and results for each state
- Clear flow visualization capabilities

### 5. Error Handling
- Built-in retry mechanisms
- Detailed error information with context
- Graceful error recovery
- Isolated error handling within sub-flows

### 6. Performance Benefits
- Efficient nested flow execution
- Minimal overhead from CosmoFlow's runtime
- Clear execution path optimization opportunities
- Memory-efficient token processing

## Advanced Features

### Sub-flow Composition
This implementation demonstrates CosmoFlow's powerful sub-flow composition capabilities:
- **Hierarchical Workflows**: Main flow orchestrates sub-flows
- **Reusable Components**: Sub-flows can be reused in different contexts
- **Independent Testing**: Each sub-flow can be tested in isolation
- **Incremental Development**: Add new token types without modifying existing flows

### DFA State Machine Mapping
The nested flow architecture provides a clean mapping from traditional DFA concepts:
- **States** → **Sub-flows** (complete workflows for token processing)
- **Transitions** → **Routes between flows** (dispatcher logic)
- **Acceptance States** → **Terminal routes** (token completion)
- **Error States** → **Error handling sub-flows** (unknown token processing)

## 运行程序

```bash
cd cookbook/lexer
cargo run
```

## 扩展可能性

1. **语法分析**: 可以进一步扩展为语法分析器，使用类似的DFA→CosmoFlow映射
2. **多语言支持**: 通过配置不同的节点和路由支持多种编程语言
3. **增量解析**: 利用CosmoFlow的状态管理实现增量词法分析
4. **并行处理**: 使用CosmoFlow的异步特性实现并行词法分析
5. **IDE集成**: 可以集成到编辑器中提供实时语法高亮和错误检测

## 结论

这个示例展示了如何将传统的编译原理概念（DFA）与现代工作流引擎（CosmoFlow）相结合，创造出模块化、可扩展、类型安全的词法分析器。这种设计模式可以应用到其他需要状态机的场景中，如协议解析、游戏AI状态管理等。
