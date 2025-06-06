//! # CosmoFlow
//!
//! A type-safe workflow engine for Rust, inspired by PocketFlow and optimized for LLM applications.
//!
//! ## Quick Start
//!
//! ```rust
//! use cosmoflow::{Flow, SharedStore, ExecutionContext};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a shared store with memory backend
//! let store = SharedStore::memory();
//!
//! // Create a flow
//! let flow = Flow::builder("my-flow")
//!     .with_store(store)
//!     .build()?;
//!
//! // Execute the flow
//! let context = ExecutionContext::new();
//! let result = flow.execute(context).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! ### Storage Backends
//! - `storage-memory`: Fast in-memory storage
//! - `storage-file`: Persistent file-based storage  
//! - `storage-full`: Both memory and file storage
//!
//! ### Built-in Components
//! - `builtin`: Pre-built node implementations
//! - `builtin-full`: Built-ins with all storage backends
//!
//! ### Meta-Features
//! - `minimal`: No features enabled (absolute minimum)
//! - `basic`: Memory storage only (lightweight)
//! - `standard`: Memory storage + built-in nodes (recommended)
//! - `full`: All features enabled (kitchen sink)
//!
//! ## Architecture
//!
//! CosmoFlow uses a modular architecture with separate crates for different concerns:
//! - Storage backends (`storage` crate)
//! - Node execution system (`node` crate)
//! - Flow orchestration (`flow` crate)
//! - Action definitions (`action` crate)
//! - Built-in node types (`builtin` crate)
//! - Shared data store (`shared_store` crate)

// ============================================================================
// Core Exports - Always Available
// ============================================================================

// Re-export the main shared store
pub use shared_store::SharedStore;

// Main workflow types
pub use flow::errors::FlowError;
pub use flow::route::Route;
pub use flow::{Flow, FlowBuilder, FlowConfig, FlowExecutionResult};

// Node system essentials
pub use node::{ExecutionContext, Node, NodeBackend, NodeError};

// Action system
pub use action::{Action, ActionCondition};

// ============================================================================
// Organized Module Exports
// ============================================================================

/// Storage backend abstractions and implementations
///
/// Available backends depend on enabled features:
/// - `storage-memory`: [`MemoryStorage`]
/// - `storage-file`: [`FileStorage`]
pub mod storage {
    pub use storage::StorageBackend;

    #[cfg(feature = "storage-memory")]
    pub use storage::backends::MemoryStorage;

    #[cfg(feature = "storage-file")]
    pub use storage::backends::FileStorage;

    // Re-export from shared_store for convenience
    pub use shared_store::SharedStore;
}

/// Node execution system and traits
///
/// Core types for building and executing workflow nodes.
pub mod nodes {
    pub use node::*;
}

/// Action definition and condition evaluation
///
/// Types for defining workflow actions and conditions.
pub mod actions {
    pub use ::action::*;
}

/// Flow definition and execution
///
/// Core flow orchestration types and builders.
pub mod flows {
    pub use flow::*;
}

/// Built-in node implementations (optional)
///
/// Pre-built node types for common workflow operations.
/// Only available when the `builtin` feature is enabled.
#[cfg(feature = "builtin")]
pub mod builtin {
    pub use builtin::*;
}

// ============================================================================
// Type Aliases and Convenience
// ============================================================================

/// A convenient result type alias for CosmoFlow operations
pub type Result<T> = std::result::Result<T, FlowError>;

/// The prelude module for commonly used types
///
/// This module provides a convenient way to import the most commonly used
/// types and traits in CosmoFlow applications.
///
/// # Example
///
/// ```rust
/// use cosmoflow::prelude::*;
///
/// // Now you have access to Flow, SharedStore, Node, etc.
/// ```
pub mod prelude {
    pub use crate::{
        Action, ActionCondition, ExecutionContext, Flow, FlowBuilder, FlowConfig,
        FlowExecutionResult, Node, NodeBackend, SharedStore,
    };

    #[cfg(feature = "builtin")]
    pub use crate::builtin::*;

    pub use crate::storage::StorageBackend;

    #[cfg(feature = "storage-memory")]
    pub use crate::storage::MemoryStorage;

    #[cfg(feature = "storage-file")]
    pub use crate::storage::FileStorage;
}
