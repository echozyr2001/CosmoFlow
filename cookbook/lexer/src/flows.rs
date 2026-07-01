use cosmoflow::prelude::*;

use crate::nodes::{
    CommentCollectorNode, DelimiterCollectorNode, IdentifierCollectorNode, IntegerCollectorNode,
    OperatorCollectorNode, StringCollectorNode, UnknownCollectorNode, WhitespaceCollectorNode,
};

/// Creates a sub-flow for handling whitespace tokens
pub fn create_whitespace_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", WhitespaceCollectorNode)
        .build()
        .expect("whitespace token flow should be valid")
}

/// Creates a sub-flow for handling identifier tokens
pub fn create_identifier_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", IdentifierCollectorNode)
        .build()
        .expect("identifier token flow should be valid")
}

/// Creates a sub-flow for handling integer tokens
pub fn create_integer_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", IntegerCollectorNode)
        .build()
        .expect("integer token flow should be valid")
}

/// Creates a sub-flow for handling string tokens
pub fn create_string_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", StringCollectorNode)
        .build()
        .expect("string token flow should be valid")
}

/// Creates a sub-flow for handling operator tokens
pub fn create_operator_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", OperatorCollectorNode)
        .build()
        .expect("operator token flow should be valid")
}

/// Creates a sub-flow for handling delimiter tokens
pub fn create_delimiter_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", DelimiterCollectorNode)
        .build()
        .expect("delimiter token flow should be valid")
}

/// Creates a sub-flow for handling comment tokens
pub fn create_comment_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", CommentCollectorNode)
        .build()
        .expect("comment token flow should be valid")
}

/// Creates a sub-flow for handling unknown tokens
pub fn create_unknown_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .node("collect", UnknownCollectorNode)
        .build()
        .expect("unknown token flow should be valid")
}
