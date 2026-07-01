use crate::{LexerContext, TokenType};
use async_trait::async_trait;
use cosmoflow::{shared_store::backends::MemoryStorage, Action, Node, NodeContext, SharedStore};
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct LexerError(String);

impl LexerError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for LexerError {}

fn load_context(store: &MemoryStorage) -> Result<LexerContext, LexerError> {
    store
        .get("lexer_context")
        .map_err(|error| LexerError::new(error.to_string()))?
        .ok_or_else(|| LexerError::new("lexer_context not found"))
}

fn save_context(store: &mut MemoryStorage, context: LexerContext) -> Result<(), LexerError> {
    store
        .set("lexer_context".to_string(), context)
        .map_err(|error| LexerError::new(error.to_string()))
}

/// Dispatcher node determines which token sub-flow should run next.
pub struct DispatcherNode;

#[async_trait]
impl Node<MemoryStorage> for DispatcherNode {
    type Prep = LexerContext;
    type Output = &'static str;
    type Error = LexerError;

    async fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        load_context(store)
    }

    async fn exec(
        &mut self,
        lexer_ctx: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        if lexer_ctx.is_at_end() {
            return Ok("end_of_input");
        }

        Ok(match lexer_ctx.current_char() {
            Some(' ') | Some('\t') | Some('\n') | Some('\r') => "whitespace_flow",
            Some('a'..='z') | Some('A'..='Z') | Some('_') => "identifier_flow",
            Some('0'..='9') => "integer_flow",
            Some('"') => "string_flow",
            Some('/') if lexer_ctx.peek_char() == Some('/') => "comment_flow",
            Some('+') | Some('-') | Some('*') | Some('/') | Some('=') | Some('<') | Some('>')
            | Some('!') => "operator_flow",
            Some('(') | Some(')') | Some('{') | Some('}') | Some('[') | Some(']') | Some(';')
            | Some(',') | Some('.') => "delimiter_flow",
            _ => "unknown_flow",
        })
    }

    async fn post(
        &mut self,
        _store: &mut MemoryStorage,
        _prep: Self::Prep,
        output: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::new(output))
    }
}

/// End-of-input node appends the final EOF token and terminates naturally.
pub struct EndOfInputNode;

#[async_trait]
impl Node<MemoryStorage> for EndOfInputNode {
    type Prep = LexerContext;
    type Output = LexerContext;
    type Error = LexerError;

    async fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        load_context(store)
    }

    async fn exec(
        &mut self,
        lexer_ctx: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let mut lexer_ctx = lexer_ctx.clone();
        lexer_ctx.add_token(TokenType::EndOfInput, String::new());
        Ok(lexer_ctx)
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep: Self::Prep,
        output: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        save_context(store, output)?;
        Ok(Action::new("complete"))
    }
}

/// Node that returns control to the dispatcher after a token sub-flow completes.
pub struct ReturnToDispatcherNode;

#[async_trait]
impl Node<MemoryStorage> for ReturnToDispatcherNode {
    type Prep = ();
    type Output = ();
    type Error = LexerError;

    async fn prep(
        &mut self,
        _store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        Ok(())
    }

    async fn exec(
        &mut self,
        _prep: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        Ok(())
    }

    async fn post(
        &mut self,
        _store: &mut MemoryStorage,
        _prep: Self::Prep,
        _output: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        Ok(Action::new("dispatch"))
    }
}

macro_rules! collector_node {
    ($node:ident, $collector:ident) => {
        pub struct $node;

        #[async_trait]
        impl Node<MemoryStorage> for $node {
            type Prep = LexerContext;
            type Output = LexerContext;
            type Error = LexerError;

            async fn prep(
                &mut self,
                store: &MemoryStorage,
                _context: &NodeContext,
            ) -> Result<Self::Prep, Self::Error> {
                load_context(store)
            }

            async fn exec(
                &mut self,
                lexer_ctx: &Self::Prep,
                _context: &NodeContext,
            ) -> Result<Self::Output, Self::Error> {
                let mut lexer_ctx = lexer_ctx.clone();
                $collector(&mut lexer_ctx);
                Ok(lexer_ctx)
            }

            async fn post(
                &mut self,
                store: &mut MemoryStorage,
                _prep: Self::Prep,
                output: Self::Output,
                _context: &NodeContext,
            ) -> Result<Action, Self::Error> {
                save_context(store, output)?;
                Ok(Action::new("complete"))
            }
        }
    };
}

collector_node!(WhitespaceCollectorNode, collect_whitespace);
collector_node!(IdentifierCollectorNode, collect_identifier);
collector_node!(IntegerCollectorNode, collect_integer);
collector_node!(StringCollectorNode, collect_string);
collector_node!(OperatorCollectorNode, collect_operator);
collector_node!(DelimiterCollectorNode, collect_delimiter);
collector_node!(CommentCollectorNode, collect_comment);
collector_node!(UnknownCollectorNode, collect_unknown);

fn collect_whitespace(lexer_ctx: &mut LexerContext) {
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
}

fn collect_identifier(lexer_ctx: &mut LexerContext) {
    let mut lexeme = String::new();
    while let Some(ch) = lexer_ctx.current_char() {
        if ch.is_alphanumeric() || ch == '_' {
            lexeme.push(ch);
            lexer_ctx.advance();
        } else {
            break;
        }
    }

    let token_type = match lexeme.as_str() {
        "if" | "else" | "while" | "for" | "fn" | "let" | "const" | "var" | "return" | "true"
        | "false" => TokenType::Keyword,
        _ => TokenType::Identifier,
    };

    lexer_ctx.add_token(token_type, lexeme);
}

fn collect_integer(lexer_ctx: &mut LexerContext) {
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
}

fn collect_string(lexer_ctx: &mut LexerContext) {
    let mut lexeme = String::new();

    if let Some('"') = lexer_ctx.current_char() {
        lexeme.push('"');
        lexer_ctx.advance();
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
}

fn collect_operator(lexer_ctx: &mut LexerContext) {
    let mut lexeme = String::new();
    if let Some(ch) = lexer_ctx.current_char() {
        lexeme.push(ch);
        lexer_ctx.advance();

        if let Some(next_ch) = lexer_ctx.current_char() {
            let two_char = format!("{ch}{next_ch}");
            if matches!(
                two_char.as_str(),
                "==" | "!=" | "<=" | ">=" | "++" | "--" | "&&" | "||"
            ) {
                lexeme.push(next_ch);
                lexer_ctx.advance();
            }
        }
    }

    lexer_ctx.add_token(TokenType::Operator, lexeme);
}

fn collect_delimiter(lexer_ctx: &mut LexerContext) {
    let mut lexeme = String::new();
    if let Some(ch) = lexer_ctx.advance() {
        lexeme.push(ch);
    }

    lexer_ctx.add_token(TokenType::Delimiter, lexeme);
}

fn collect_comment(lexer_ctx: &mut LexerContext) {
    let mut lexeme = String::new();
    while let Some(ch) = lexer_ctx.current_char() {
        if ch == '\n' {
            break;
        }
        lexeme.push(ch);
        lexer_ctx.advance();
    }

    lexer_ctx.add_token(TokenType::Comment, lexeme);
}

fn collect_unknown(lexer_ctx: &mut LexerContext) {
    if let Some(ch) = lexer_ctx.advance() {
        lexer_ctx.add_token(TokenType::Unknown, ch.to_string());
    }
}
