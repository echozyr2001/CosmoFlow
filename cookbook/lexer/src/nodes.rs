use crate::{LexerContext, TokenType};
use cosmoflow::prelude::*;

/// Dispatcher node - determines which sub-flow to route to
pub struct DispatcherNode;

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
pub struct EndOfInputNode;

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
pub struct ReturnToDispatcherNode;

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

/// Optimized whitespace collector node
pub struct WhitespaceCollectorNode;

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

/// Optimized identifier collector node with keyword classification
pub struct IdentifierCollectorNode;

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

/// Optimized integer collector node
pub struct IntegerCollectorNode;

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

/// Optimized string collector node
pub struct StringCollectorNode;

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

/// Optimized operator collector node
pub struct OperatorCollectorNode;

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

/// Optimized delimiter collector node
pub struct DelimiterCollectorNode;

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

        let mut lexeme = String::new();
        if let Some(ch) = lexer_ctx.advance() {
            lexeme.push(ch);
        }

        lexer_ctx.add_token(TokenType::Delimiter, lexeme);
        store.set("lexer_context".to_string(), &lexer_ctx).unwrap();

        Ok(Action::simple("complete"))
    }
}

/// Optimized comment collector node
pub struct CommentCollectorNode;

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

/// Optimized unknown token collector node
pub struct UnknownCollectorNode;

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
