use cosmoflow::prelude::*;

use crate::nodes::{
    CommentCollectorNode, DelimiterCollectorNode, IdentifierCollectorNode, IntegerCollectorNode,
    OperatorCollectorNode, StringCollectorNode, UnknownCollectorNode, WhitespaceCollectorNode,
};

/// Creates a sub-flow for handling whitespace tokens
pub fn create_whitespace_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", WhitespaceCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Creates a sub-flow for handling identifier tokens
pub fn create_identifier_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", IdentifierCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Creates a sub-flow for handling integer tokens
pub fn create_integer_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", IntegerCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Creates a sub-flow for handling string tokens
pub fn create_string_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", StringCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Creates a sub-flow for handling operator tokens
pub fn create_operator_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", OperatorCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Creates a sub-flow for handling delimiter tokens
pub fn create_delimiter_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", DelimiterCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Creates a sub-flow for handling comment tokens
pub fn create_comment_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", CommentCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}

/// Creates a sub-flow for handling unknown tokens
pub fn create_unknown_flow() -> Flow<MemoryStorage> {
    FlowBuilder::new()
        .start_node("collect")
        .node("collect", UnknownCollectorNode)
        .terminal_route("collect", "complete")
        .build()
}
