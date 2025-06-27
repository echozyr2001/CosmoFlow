mod flows;
mod nodes;

use cosmoflow::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    flows::{
        create_comment_flow, create_delimiter_flow, create_identifier_flow, create_integer_flow,
        create_operator_flow, create_string_flow, create_unknown_flow, create_whitespace_flow,
    },
    nodes::{DispatcherNode, EndOfInputNode, ReturnToDispatcherNode},
};

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

/// CosmoFlow-based lexer that implements a DFA-based tokenizer
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
