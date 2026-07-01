use std::{error::Error, fmt, fs, path::Path, time::SystemTime};

use async_trait::async_trait;
use clap::Parser;
use cosmoflow::{Action, Node, NodeContext, SharedStore, prelude::MemoryStorage};
use tabled::{Table, settings::Color};

use crate::{
    Args, FileEntry,
    utils::{format_permissions, format_size, format_time},
};

#[derive(Debug)]
pub struct BetterLsError(String);

impl BetterLsError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for BetterLsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for BetterLsError {}

/// Input node that processes command line arguments using clap
pub struct InputNode {
    args: Args,
}

impl InputNode {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
        }
    }
}

impl Default for InputNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Node<MemoryStorage> for InputNode {
    type Prep = Args;
    type Output = Args;
    type Error = BetterLsError;

    async fn prep(
        &mut self,
        _store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        // Return the parsed arguments
        Ok(self.args.clone())
    }

    async fn exec(
        &mut self,
        prep_result: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        // Get directory path from arguments or default to current directory
        let target_path = prep_result.path.as_deref().unwrap_or(".");
        let path = Path::new(target_path);

        // Validate that the path exists and is a directory
        if !path.exists() {
            return Err(BetterLsError::new(format!(
                "path does not exist: {target_path}"
            )));
        }

        if !path.is_dir() {
            return Err(BetterLsError::new(format!(
                "path is not a directory: {target_path}"
            )));
        }

        Ok(prep_result.clone())
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::Prep,
        exec_result: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        // Store the parsed arguments
        store
            .set("args".to_string(), exec_result)
            .map_err(|e| BetterLsError::new(e.to_string()))?;

        Ok(Action::new("list"))
    }

    fn name(&self) -> &str {
        "InputNode"
    }
}

/// LS node that lists directory contents
pub struct LsNode;

#[async_trait]
impl Node<MemoryStorage> for LsNode {
    type Prep = Args;
    type Output = Vec<FileEntry>;
    type Error = BetterLsError;

    async fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        // Get the parsed arguments from storage
        let args: Args = store
            .get("args")
            .map_err(|e| BetterLsError::new(e.to_string()))?
            .ok_or_else(|| BetterLsError::new("arguments not found"))?;

        Ok(args)
    }

    async fn exec(
        &mut self,
        prep_result: &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let target_path = prep_result.path.as_deref().unwrap_or(".");
        let path = Path::new(target_path);

        // Read directory contents
        let entries = fs::read_dir(path)
            .map_err(|e| BetterLsError::new(format!("failed to read directory: {e}")))?;

        let mut file_entries = Vec::new();

        for entry in entries {
            let entry =
                entry.map_err(|e| BetterLsError::new(format!("failed to read entry: {e}")))?;

            let metadata = entry
                .metadata()
                .map_err(|e| BetterLsError::new(format!("failed to read metadata: {e}")))?;

            let name = entry.file_name().to_string_lossy().to_string();

            // Check if file is hidden (starts with .)
            let is_hidden = name.starts_with('.');

            // Skip hidden files unless --all flag is set
            if is_hidden && !prep_result.all {
                continue;
            }

            let is_directory = metadata.is_dir();

            // Check if file is executable
            let is_executable = {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    !is_directory && (metadata.permissions().mode() & 0o111) != 0
                }
                #[cfg(not(unix))]
                {
                    !is_directory
                        && (name.ends_with(".exe")
                            || name.ends_with(".bat")
                            || name.ends_with(".cmd"))
                }
            };

            // File type without color
            let file_type = if is_directory { "DIR" } else { "FILE" }.to_string();

            // Store original name without color
            let name = name.clone();

            let size_bytes = if is_directory { 0 } else { metadata.len() };

            // Format size without color
            let size = if is_directory {
                "<DIR>".to_string()
            } else if prep_result.human_readable {
                format_size(metadata.len())
            } else {
                format!("{} B", metadata.len())
            };

            // Enhanced permissions representation
            let permissions = format_permissions(&metadata);

            // Get timestamps
            let modified_timestamp = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let created_timestamp = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);

            // Format timestamps
            let modified = format_time(modified_timestamp);
            let created = format_time(created_timestamp);

            file_entries.push(FileEntry {
                file_type,
                name,
                size,
                permissions,
                modified,
                created,
                is_hidden,
                modified_timestamp,
                size_bytes,
                is_directory,
                is_executable,
            });
        }

        // Apply sorting based on options
        if prep_result.sort_time {
            // Sort by modification time
            file_entries.sort_by(|a, b| {
                if prep_result.reverse {
                    a.modified_timestamp.cmp(&b.modified_timestamp)
                } else {
                    b.modified_timestamp.cmp(&a.modified_timestamp)
                }
            });
        } else {
            // Default sort: directories first, then files, both alphabetically
            file_entries.sort_by(|a, b| {
                let a_is_dir = a.file_type.contains("DIR");
                let b_is_dir = b.file_type.contains("DIR");
                let ordering = match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                };

                if prep_result.reverse {
                    ordering.reverse()
                } else {
                    ordering
                }
            });
        }

        Ok(file_entries)
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::Prep,
        exec_result: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        // Store the file listing
        store
            .set("file_listing".to_string(), exec_result)
            .map_err(|e| BetterLsError::new(e.to_string()))?;

        Ok(Action::new("output"))
    }

    fn name(&self) -> &str {
        "LsNode"
    }
}

/// Output node that formats and displays the results
pub struct OutputNode;

#[async_trait]
impl Node<MemoryStorage> for OutputNode {
    type Prep = (Args, Vec<FileEntry>);
    type Output = String;
    type Error = BetterLsError;

    async fn prep(
        &mut self,
        store: &MemoryStorage,
        _context: &NodeContext,
    ) -> Result<Self::Prep, Self::Error> {
        // Get both the args and file listing
        let args: Args = store
            .get("args")
            .map_err(|e| BetterLsError::new(e.to_string()))?
            .ok_or_else(|| BetterLsError::new("arguments not found"))?;

        let file_listing: Vec<FileEntry> = store
            .get("file_listing")
            .map_err(|e| BetterLsError::new(e.to_string()))?
            .ok_or_else(|| BetterLsError::new("file listing not found"))?;

        Ok((args, file_listing))
    }

    async fn exec(
        &mut self,
        (_args, file_listing): &Self::Prep,
        _context: &NodeContext,
    ) -> Result<Self::Output, Self::Error> {
        let mut output = String::new();

        if file_listing.is_empty() {
            output.push_str("Directory is empty\n");
        } else {
            use tabled::settings::{Alignment, Style, object::Columns};

            // Create table with proper alignment and colors using tabled's Color enum
            let table_string = {
                Table::new(file_listing.clone())
                    .with(Style::rounded())
                    .modify(tabled::settings::object::Rows::new(1..), Alignment::left())
                    .modify(Columns::new(2..=2), Alignment::right())
                    // Apply colors to specific columns
                    .modify(Columns::new(0..=0), Color::FG_BLUE) // Type column - blue for DIR
                    .modify(Columns::new(1..=1), Color::FG_GREEN) // Name column - green
                    .modify(Columns::new(2..=2), Color::FG_YELLOW) // Size column - yellow
                    .to_string()
            };

            output.push_str(&table_string);
        }

        Ok(output)
    }

    async fn post(
        &mut self,
        store: &mut MemoryStorage,
        _prep_result: Self::Prep,
        exec_result: Self::Output,
        _context: &NodeContext,
    ) -> Result<Action, Self::Error> {
        // Store formatted output for potential reuse - demonstrating CosmoFlow's data flow
        store
            .set("formatted_output".to_string(), exec_result.clone())
            .map_err(|e| BetterLsError::new(e.to_string()))?;

        // Display the formatted output
        println!("{exec_result}");

        Ok(Action::new("complete"))
    }

    fn name(&self) -> &str {
        "OutputNode"
    }
}
