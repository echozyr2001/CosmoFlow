use cosmoflow::prelude::*;
use serde::{Deserialize, Serialize};

/// Token types that our lexer can recognize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    /// Identifiers (variable names, function names, etc.)
    Identifier,
    /// Integer literals
    Integer,
    /// String literals (enclosed in quotes)
    String,
    /// Keywords (if, else, while, etc.)
    Keyword,
    /// Operators (+, -, *, /, =, ==, etc.)
    Operator,
    /// Delimiters (, ), {, }, [, ], ;, ,, etc.)
    Delimiter,
    /// Whitespace (spaces, tabs, newlines)
    Whitespace,
    /// Comments
    Comment,
    /// Unknown/invalid token
    Unknown,
    /// End of input
    EndOfInput,
}

/// A token produced by the lexer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: u32,
    pub column: u32,
}

/// Lexer context containing the input and current position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexerContext {
    pub input: String,
    pub position: usize,
    pub line: u32,
    pub column: u32,
    pub tokens: Vec<Token>,
}

impl LexerContext {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    pub fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    pub fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    pub fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current_char() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn add_token(&mut self, token_type: TokenType, lexeme: String) {
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
            column: self.column,
        });
    }

    pub fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

/// Dispatcher node - determines which sub-flow to route to
struct DispatcherNode;

impl<S: SharedStore> Node<S> for DispatcherNode {
    type PrepResult = ();
    type ExecResult = String;
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(
        &mut self,
        _prep_result: (),
        _context: &ExecutionContext,
    ) -> Result<String, Self::Error> {
        Ok("analyzed".to_string())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: String,
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        if lexer_ctx.is_at_end() {
            return Ok(Action::simple("end_of_input"));
        }

        match lexer_ctx.current_char() {
            Some(' ') | Some('\t') | Some('\n') | Some('\r') => {
                Ok(Action::simple("whitespace_flow"))
            }
            Some('a'..='z') | Some('A'..='Z') | Some('_') => Ok(Action::simple("identifier_flow")),
            Some('0'..='9') => Ok(Action::simple("integer_flow")),
            Some('"') => Ok(Action::simple("string_flow")),
            Some('/') if lexer_ctx.peek_char() == Some('/') => Ok(Action::simple("comment_flow")),
            Some('+') | Some('-') | Some('*') | Some('/') | Some('=') | Some('<') | Some('>')
            | Some('!') => Ok(Action::simple("operator_flow")),
            Some('(') | Some(')') | Some('{') | Some('}') | Some('[') | Some(']') | Some(';')
            | Some(',') | Some('.') => Ok(Action::simple("delimiter_flow")),
            _ => Ok(Action::simple("unknown_flow")),
        }
    }
}

/// End of input node
struct EndOfInputNode;

impl<S: SharedStore> Node<S> for EndOfInputNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        lexer_ctx.add_token(TokenType::EndOfInput, String::new());
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Node that returns control to the dispatcher after processing a token
struct ReturnToDispatcherNode;

impl<S: SharedStore> Node<S> for ReturnToDispatcherNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        _store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::simple("dispatch"))
    }
}

/// Creates a sub-flow for handling whitespace tokens
fn create_whitespace_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", WhitespaceCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Node for collecting whitespace characters
struct WhitespaceCollectorNode;

impl<S: SharedStore> Node<S> for WhitespaceCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        let mut lexeme = String::new();
        while let Some(ch) = lexer_ctx.current_char() {
            if ch.is_whitespace() {
                lexeme.push(ch);
                lexer_ctx.advance();
            } else {
                break;
            }
        }

        lexer_ctx.add_token(TokenType::Whitespace, lexeme);
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Creates a sub-flow for handling identifier tokens
fn create_identifier_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", IdentifierCollectorNode)
        .node("classify", IdentifierClassifierNode)
        .route("collect", "classify", "classify")
        .terminal_route("classify", "complete")
        .build()
}

/// Node for collecting identifier characters
struct IdentifierCollectorNode;

impl<S: SharedStore> Node<S> for IdentifierCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        let mut lexeme = String::new();
        while let Some(ch) = lexer_ctx.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                lexeme.push(ch);
                lexer_ctx.advance();
            } else {
                break;
            }
        }

        // Store the collected lexeme for classification
        store.set("temp_lexeme".to_string(), &lexeme).unwrap();
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("classify"))
    }
}

/// Node for classifying identifier as keyword or identifier
struct IdentifierClassifierNode;

impl<S: SharedStore> Node<S> for IdentifierClassifierNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        // Get the collected lexeme from temporary storage
        let lexeme = store
            .get::<String>("temp_lexeme")
            .unwrap_or_default()
            .unwrap_or_default();

        // Check if it's a keyword
        let token_type = match lexeme.as_str() {
            "if" | "else" | "while" | "for" | "fn" | "let" | "const" | "var" | "return"
            | "true" | "false" => TokenType::Keyword,
            _ => TokenType::Identifier,
        };

        lexer_ctx.add_token(token_type, lexeme);
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Creates a sub-flow for handling integer tokens
fn create_integer_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", IntegerCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Node for collecting integer characters
struct IntegerCollectorNode;

impl<S: SharedStore> Node<S> for IntegerCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        let mut lexeme = String::new();
        while let Some(ch) = lexer_ctx.current_char() {
            if ch.is_ascii_digit() {
                lexeme.push(ch);
                lexer_ctx.advance();
            } else {
                break;
            }
        }

        lexer_ctx.add_token(TokenType::Integer, lexeme);
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Creates a sub-flow for handling string tokens
fn create_string_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", StringCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Node for collecting string characters
struct StringCollectorNode;

impl<S: SharedStore> Node<S> for StringCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        let mut lexeme = String::new();

        // Consume opening quote
        if let Some('"') = lexer_ctx.current_char() {
            lexeme.push(lexer_ctx.advance().unwrap());
        }

        let mut escaped = false;
        while let Some(ch) = lexer_ctx.current_char() {
            lexeme.push(ch);
            lexer_ctx.advance();

            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                break;
            }
        }

        lexer_ctx.add_token(TokenType::String, lexeme);
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Creates a sub-flow for handling operator tokens
fn create_operator_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", OperatorCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Node for collecting operator characters
struct OperatorCollectorNode;

impl<S: SharedStore> Node<S> for OperatorCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        let mut lexeme = String::new();
        if let Some(ch) = lexer_ctx.current_char() {
            lexeme.push(ch);
            lexer_ctx.advance();

            // Handle two-character operators
            if let Some(next_ch) = lexer_ctx.current_char() {
                let two_char = format!("{}{}", ch, next_ch);
                match two_char.as_str() {
                    "==" | "!=" | "<=" | ">=" | "++" | "--" | "&&" | "||" => {
                        lexeme.push(next_ch);
                        lexer_ctx.advance();
                    }
                    _ => {}
                }
            }
        }

        lexer_ctx.add_token(TokenType::Operator, lexeme);
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Creates a sub-flow for handling delimiter tokens
fn create_delimiter_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", DelimiterCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Node for collecting delimiter characters
struct DelimiterCollectorNode;

impl<S: SharedStore> Node<S> for DelimiterCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        if let Some(ch) = lexer_ctx.advance() {
            lexer_ctx.add_token(TokenType::Delimiter, ch.to_string());
        }

        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Creates a sub-flow for handling comment tokens
fn create_comment_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", CommentCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Node for collecting comment characters
struct CommentCollectorNode;

impl<S: SharedStore> Node<S> for CommentCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        let mut lexeme = String::new();
        while let Some(ch) = lexer_ctx.current_char() {
            if ch == '\n' {
                break;
            }
            lexeme.push(ch);
            lexer_ctx.advance();
        }

        lexer_ctx.add_token(TokenType::Comment, lexeme);
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Creates a sub-flow for handling unknown tokens
fn create_unknown_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", UnknownCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Node for handling unknown characters
struct UnknownCollectorNode;

impl<S: SharedStore> Node<S> for UnknownCollectorNode {
    type PrepResult = ();
    type ExecResult = ();
    type Error = NodeError;

    fn prep(&mut self, _store: &S, _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn exec(&mut self, _prep_result: (), _context: &ExecutionContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post(
        &mut self,
        store: &mut S,
        _prep_result: (),
        _exec_result: (),
        _context: &ExecutionContext,
    ) -> Result<Action, Self::Error> {
        let lexer_ctx: Option<LexerContext> = store.get("lexer_context").unwrap_or_default();
        let mut lexer_ctx = lexer_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        if let Some(ch) = lexer_ctx.advance() {
            lexer_ctx.add_token(TokenType::Unknown, ch.to_string());
        }

        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Lexer that uses CosmoFlow to implement a DFA-based tokenizer with nested flows
pub struct CosmoFlowLexer {
    flow: Flow<MemoryStorage>,
}

impl Default for CosmoFlowLexer {
    fn default() -> Self {
        // Create sub-flows for each token type
        let whitespace_flow = create_whitespace_flow();
        let identifier_flow = create_identifier_flow();
        let integer_flow = create_integer_flow();
        let string_flow = create_string_flow();
        let operator_flow = create_operator_flow();
        let delimiter_flow = create_delimiter_flow();
        let comment_flow = create_comment_flow();
        let unknown_flow = create_unknown_flow();

        // Build the main flow with dispatcher and sub-flows
        let flow = FlowBuilder::new()
            .start_node("dispatch")
            .node("dispatch", DispatcherNode)
            .node("end_of_input", EndOfInputNode)
            .node("return_to_dispatcher", ReturnToDispatcherNode)
            // Add sub-flows as nodes
            .node("whitespace_flow", whitespace_flow)
            .node("identifier_flow", identifier_flow)
            .node("integer_flow", integer_flow)
            .node("string_flow", string_flow)
            .node("operator_flow", operator_flow)
            .node("delimiter_flow", delimiter_flow)
            .node("comment_flow", comment_flow)
            .node("unknown_flow", unknown_flow)
            // Routes from dispatcher to sub-flows
            .route("dispatch", "whitespace_flow", "whitespace_flow")
            .route("dispatch", "identifier_flow", "identifier_flow")
            .route("dispatch", "integer_flow", "integer_flow")
            .route("dispatch", "string_flow", "string_flow")
            .route("dispatch", "operator_flow", "operator_flow")
            .route("dispatch", "delimiter_flow", "delimiter_flow")
            .route("dispatch", "comment_flow", "comment_flow")
            .route("dispatch", "unknown_flow", "unknown_flow")
            .route("dispatch", "end_of_input", "end_of_input")
            // Routes from sub-flows back to dispatcher
            .route("whitespace_flow", "complete", "return_to_dispatcher")
            .route("identifier_flow", "complete", "return_to_dispatcher")
            .route("integer_flow", "complete", "return_to_dispatcher")
            .route("string_flow", "complete", "return_to_dispatcher")
            .route("operator_flow", "complete", "return_to_dispatcher")
            .route("delimiter_flow", "complete", "return_to_dispatcher")
            .route("comment_flow", "complete", "return_to_dispatcher")
            .route("unknown_flow", "complete", "return_to_dispatcher")
            // Route from return node back to dispatcher
            .route("return_to_dispatcher", "dispatch", "dispatch")
            // Terminal route for end of input
            .terminal_route("end_of_input", "complete")
            .build();

        Self { flow }
    }
}

impl CosmoFlowLexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tokenize(&mut self, input: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        let mut store = MemoryStorage::new();
        let lexer_ctx = LexerContext::new(input.to_string());
        store.set("lexer_context".to_string(), &lexer_ctx)?;

        let _result = self.flow.execute(&mut store)?;

        let final_ctx: Option<LexerContext> = store.get("lexer_context")?;
        let final_ctx = final_ctx.unwrap_or_else(|| LexerContext::new(String::new()));

        Ok(final_ctx.tokens)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer = CosmoFlowLexer::new();

    // Test the lexer with a simple program
    let input = r#"
        fn main() {
            let x = 42;
            if x > 0 {
                println!("Hello, world!");
            }
        }
    "#;

    println!("Tokenizing input:");
    println!("{}", input);
    println!("\nTokens:");

    let tokens = lexer.tokenize(input)?;
    for (i, token) in tokens.iter().enumerate() {
        println!("{:3}: {:?}", i, token);
    }

    // Test with a more complex example
    let complex_input = "x = y + 42 * (z - 1) // comment";
    println!("\n\nTokenizing complex input:");
    println!("{}", complex_input);
    println!("\nTokens:");

    let mut lexer2 = CosmoFlowLexer::new();
    let tokens2 = lexer2.tokenize(complex_input)?;
    for (i, token) in tokens2.iter().enumerate() {
        println!("{:3}: {:?}", i, token);
    }

    Ok(())
}
