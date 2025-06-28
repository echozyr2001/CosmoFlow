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

### Three-Layer Hierarchical DFA Architecture

Our lexer implements a **sophisticated three-layer DFA architecture** using CosmoFlow's nested flows, where each layer represents a different level of abstraction:

#### Layer 1: Main Dispatcher DFA
- **Purpose**: High-level token type detection and routing
- **Responsibility**: Analyze current character and route to appropriate token-specific sub-flow
- **Implementation**: Single dispatcher node with character-based routing logic

#### Layer 2: Token-Type Sub-flows
- **Purpose**: Token-specific processing logic
- **Responsibility**: Handle the complete lifecycle of each token type
- **Implementation**: Dedicated sub-flows for each token type (identifier, integer, string, etc.)
- **Features**: Some flows include multi-stage processing (e.g., identifier → classification)

#### Layer 3: Character Collection Sub-flows
- **Purpose**: Fine-grained character collection and validation
- **Responsibility**: Implement the actual DFA logic for character processing
- **Implementation**: Nested sub-flows within token collectors
- **Advantage**: Maximum modularity - even character collection logic is a composable flow

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

### Three-Layer Nested Flow Architecture

The lexer uses a **hierarchical dispatcher-based architecture** with **three levels of nested sub-flows**:

#### 1. Main Flow Structure (Layer 1)
```
Dispatcher → Token Sub-flow → Return to Dispatcher → Repeat → End
```

#### 2. Token Sub-flow Structure (Layer 2)
```
Collector → [Optional Processing] → Complete
    ↓
Character Collection Sub-flow (Layer 3)
```

#### 3. Character Collection Sub-flow Structure (Layer 3)
```
Start → Character Processing Loop → Complete
```

#### Example: Identifier Processing Flow
```
Layer 1: DISPATCHER ──(identifier_flow)──> IDENTIFIER_SUB_FLOW
                                                   |
Layer 2:                            COLLECTOR ──> CLASSIFIER ──> COMPLETE
                                        |
Layer 3:                    CHAR_COLLECTION_SUB_FLOW
                                        |
                            (alphanumeric/underscore loop)
```

#### 4. Dispatcher Node (DispatcherNode)
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

#### 5. Token-Specific Sub-flows (Layer 2)
Each token type is handled by a dedicated sub-flow that coordinates character collection and processing:

- **`whitespace_flow`**: Direct character collection (simple case)
- **`identifier_flow`**: Character collection → Classification (keyword/identifier)
- **`integer_flow`**: Character collection → Token creation
- **`string_flow`**: Character collection with escape sequence handling
- **`operator_flow`**: Character collection with multi-character operator detection
- **`delimiter_flow`**: Single character collection
- **`comment_flow`**: Character collection until newline
- **`unknown_flow`**: Single unknown character handling

#### 6. Character Collection Sub-flows (Layer 3)
Each collector node uses its own dedicated character collection sub-flow:

```rust
// Example: IdentifierCollectorNode contains
struct IdentifierCollectorNode {
    char_collector_flow: Flow<MemoryStorage>,  // Layer 3 sub-flow
}

// Character collection sub-flow structure:
FlowBuilder::new()
    .start_node("collect_all")
    .node("collect_all", IdentifierCharCollectorNode)  // DFA logic
    .terminal_route("collect_all", "complete")
    .build()
```

#### 7. Sub-flow Internal Structure (Layer 2 Example)
Each token sub-flow is a complete Flow with its own nodes and routes:

```rust
// Example: identifier_flow (Layer 2)
FlowBuilder::new()
    .start_node("collect")
    .node("collect", IdentifierCollectorNode::new())  // Contains Layer 3 sub-flow
    .node("classify", IdentifierClassifierNode)       // Classification logic
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

## Complete DFA Architecture Diagram

Our lexer implements a **three-layer hierarchical DFA architecture** using CosmoFlow's nested flows:

### Layer 1: Main Dispatcher DFA

```
                    [START]
                       |
                       v
                 [DISPATCHER]
                       |
      ┌────────────────┼────────────────┐
      |                |                |
      v                v                v
[whitespace] →  [identifier] →    [integer] → ... (other token flows)
      |                |                |
      v                v                v
[return_to_dispatcher] ← ─ ─ ─ ─ ─ ─ ─ ─ ┘
      |
      v
[DISPATCHER] ─────────────────────────────────┐
      |                                      |
      v                                      |
[end_of_input] ──────────────> [END]        |
                                             |
                                    ─ ─ ─ ─ ─ ┘
                               (loop back)
```

### Layer 2: Token-Type Sub-flows

#### Identifier Flow DFA
```
[START] → [IdentifierCollector] → [IdentifierClassifier] → [COMPLETE]
                |                           |
                v                           v
    (uses char collector sub-flow)   (keyword/identifier)
```

#### Integer Flow DFA
```
[START] → [IntegerCollector] → [COMPLETE]
                |
                v
    (uses char collector sub-flow)
```

#### String Flow DFA
```
[START] → [StringCollector] → [COMPLETE]
                |
                v
    (uses char collector sub-flow)
```

#### Other Flows (Operator, Delimiter, Comment)
```
[START] → [Collector] → [COMPLETE]
                |
                v
    (uses char collector sub-flow)
```

### Layer 3: Character Collection Sub-flows

#### Identifier Character Collection DFA
```
                    [START]
                       |
                       v
                [collect_all]
                       |
         ┌─────────────┼─────────────┐
         |   (alphanumeric or '_')   |
         v                           |
  [add char & advance] ──────────────┘
         |
         v (not alphanumeric/underscore)
    [COMPLETE]
```

#### Integer Character Collection DFA
```
                    [START]
                       |
                       v
                [collect_all]
                       |
         ┌─────────────┼─────────────┐
         |      (is digit)          |
         v                           |
  [add char & advance] ──────────────┘
         |
         v (not digit)
    [COMPLETE]
```

#### String Character Collection DFA
```
                    [START]
                       |
                       v
                [collect_all]
                       |
                       v
              [consume opening quote]
                       |
         ┌─────────────┼─────────────┐
         |    (not closing quote)    |
         v                           |
   [handle escape &    ──────────────┘
    add char & advance]
         |
         v (closing quote found)
    [COMPLETE]
```

#### Operator Character Collection DFA
```
                    [START]
                       |
                       v
                [collect_all]
                       |
                       v
              [consume first char]
                       |
                       v
            [check for two-char operators]
                   |       |
            (==, !=, etc)  |  (single char)
                   |       |
                   v       v
           [consume second] [keep first only]
                   |       |
                   └───────┘
                       |
                       v
                  [COMPLETE]
```

#### Comment Character Collection DFA
```
                    [START]
                       |
                       v
                [collect_all]
                       |
         ┌─────────────┼─────────────┐
         |    (not newline)         |
         v                           |
  [add char & advance] ──────────────┘
         |
         v (newline found)
    [COMPLETE]
```

#### Delimiter Character Collection DFA
```
                    [START]
                       |
                       v
                [collect_all]
                       |
                       v
              [consume single char]
                       |
                       v
                  [COMPLETE]
```

### Complete Flow Integration

```
Main Flow:
  DISPATCHER ──(route)──> Token Sub-flow ──(complete)──> RETURN_TO_DISPATCHER
      ^                                                           |
      |                                                           |
      └──────────────────(dispatch)────────────────────────────── ┘

Token Sub-flow:
  COLLECTOR ──(uses)──> Character Collection Sub-flow ──(complete)──> [Next Node or COMPLETE]

Character Collection Sub-flow:
  Specific character collection logic based on token type
```

### Dispatcher Routing Logic

```rust
match current_char {
    ' ' | '\t' | '\n' | '\r'           → whitespace_flow
    'a'..='z' | 'A'..='Z' | '_'        → identifier_flow  
    '0'..='9'                          → integer_flow
    '"'                                → string_flow
    '/' if next_char == '/'            → comment_flow
    '+' | '-' | '*' | '/' | '=' | ...  → operator_flow
    '(' | ')' | '{' | '}' | ...        → delimiter_flow
    _                                  → unknown_flow
    EOF                                → end_of_input
}
```

## Technical Advantages

### 1. Three-Layer Hierarchical Design with Nested Flows
- **Layer 1**: High-level token type routing and flow orchestration
- **Layer 2**: Token-specific processing logic with dedicated sub-flows  
- **Layer 3**: Fine-grained character collection DFAs as composable sub-flows
- **Benefits**: Maximum modularity, each layer has clear responsibilities
- Easy to extend with new token types at any layer
- Clear separation of concerns across all three layers
- State transition logic is explicit and maintainable at each level

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

### Three-Layer Sub-flow Composition
This implementation demonstrates CosmoFlow's most advanced sub-flow composition capabilities:

#### Hierarchical Workflows (3 Levels)
- **Level 1**: Main flow orchestrates token-type sub-flows
- **Level 2**: Token sub-flows orchestrate character collection sub-flows  
- **Level 3**: Character collection sub-flows implement pure DFA logic
- **Result**: True hierarchical state machine composition

#### Reusable Components at Every Layer
- **Layer 1**: Dispatcher pattern reusable for other parsing tasks
- **Layer 2**: Token sub-flows reusable across different language lexers
- **Layer 3**: Character collection sub-flows reusable for any string processing
- **Cross-Layer**: Any sub-flow can be extracted and reused independently

#### Independent Testing and Development
- **Each Layer**: Can be tested in complete isolation
- **Character Collection**: Pure DFA logic easily unit tested
- **Token Processing**: Integration tested with mock character collectors
- **Main Flow**: End-to-end tested with all layers integrated
- **Incremental Development**: Add new token types without touching existing layers

### Advanced DFA State Machine Mapping
The three-layer nested flow architecture provides the most sophisticated mapping from traditional DFA concepts:

#### Traditional DFA → Three-Layer CosmoFlow Mapping
- **DFA States** → **Three types of sub-flows** (dispatcher, token, character)
- **State Transitions** → **Inter-layer and intra-layer routes** 
- **Acceptance States** → **Terminal routes at appropriate layers**
- **Error States** → **Error handling sub-flows at each layer**
- **Nested DFAs** → **True nested sub-flows** (DFA within DFA within DFA)

#### Complex State Machine Benefits
- **Composability**: DFA logic can be composed at multiple levels
- **Extensibility**: New states can be added at the appropriate layer
- **Maintainability**: State logic is isolated within appropriate abstractions
- **Reusability**: DFA components can be reused across different contexts

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
