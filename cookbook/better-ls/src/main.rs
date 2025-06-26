pub mod nodes;
pub mod utils;

use clap::Parser;
use cosmoflow::{
    flow::{FlowBackend, macros::flow},
    prelude::MemoryStorage,
};
use serde::Serialize;

use std::time::SystemTime;
use tabled::Tabled;

use crate::nodes::{InputNode, LsNode, OutputNode};

/// CosmoFlow LS Command - Enhanced directory listing with metadata and table display
#[derive(Parser, Debug, Clone, Serialize, serde::Deserialize)]
#[command(name = "ls_command")]
#[command(about = "A feature-rich directory listing tool built with CosmoFlow")]
#[command(version = "1.0")]
pub struct Args {
    /// Directory path to list (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<String>,

    /// Show all files including hidden ones (starting with .)
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Sort by modification time (newest first)
    #[arg(short = 't', long = "time")]
    pub sort_time: bool,

    /// Reverse sort order
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,

    #[arg(long = "human-readable", default_value = "true")]
    /// Show raw byte sizes instead of human readable format
    #[arg(long = "no-human-readable", action = clap::ArgAction::SetFalse)]
    pub human_readable: bool,
}

/// Represents a file or directory entry with comprehensive metadata
#[derive(Debug, Clone, Serialize, serde::Deserialize, Tabled)]
pub struct FileEntry {
    #[tabled(rename = "Type")]
    pub file_type: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Size")]
    pub size: String,
    #[tabled(rename = "Permissions")]
    pub permissions: String,
    #[tabled(rename = "Modified")]
    pub modified: String,
    #[tabled(rename = "Created")]
    pub created: String,
    // Hidden fields for sorting and filtering
    #[tabled(skip)]
    pub is_hidden: bool,
    #[tabled(skip)]
    pub modified_timestamp: SystemTime,
    #[tabled(skip)]
    pub size_bytes: u64,
    #[tabled(skip)]
    pub is_directory: bool,
    #[tabled(skip)]
    pub is_executable: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create our custom storage
    let mut storage = MemoryStorage::new();

    // Build the enhanced workflow using flow! macro - showcasing CosmoFlow's declarative syntax
    let mut flow = flow! {
        storage: MemoryStorage,
        start: "input",
        nodes: {
            "input": InputNode::new(),
            "ls": LsNode,
            "output": OutputNode,
        },
        routes: {
            "input" - "list" => "ls",
            "ls" - "output" => "output",
        },
        terminals: {
            "output" - "complete",
        }
    };

    // Validate the flow - demonstrates CosmoFlow's built-in validation
    if let Err(e) = flow.validate() {
        eprintln!("❌ Flow validation failed: {e}");
        return Err(e.into());
    }

    // Execute the flow - showcasing CosmoFlow's execution engine
    if let Err(e) = flow.execute(&mut storage) {
        eprintln!("❌ Execution failed: {e}");
        return Err(e.into());
    }

    Ok(())
}
