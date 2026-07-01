pub mod nodes;
pub mod utils;

use clap::Parser;
use cosmoflow::{FlowBuilder, prelude::MemoryStorage};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create our custom storage
    let mut storage = MemoryStorage::new();

    let mut flow = FlowBuilder::new()
        .node("input", InputNode::new())
        .node("ls", LsNode)
        .node("output", OutputNode)
        .route("input", "list", "ls")
        .route("ls", "output", "output")
        .build()?;

    flow.run_recorded(&mut storage).await?;

    Ok(())
}
