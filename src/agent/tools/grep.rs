use std::fs;
use std::path::Path;

use grep::regex::RegexMatcher;
use grep::searcher::Searcher;
use grep::searcher::sinks::UTF8;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{Tool, tool::ToolInfo, tool_error::ToolError};

pub struct Grep;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GrepInput {
    /// The regex pattern to search for.
    pub pattern: String,
    /// The directory or file path to search in. Must be an absolute path.
    pub path: String,
}

impl Tool for Grep {
    type Input = GrepInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("Grep")
            .with_description("Search for a regex pattern in a file or directory (recursive).")
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        let matcher = RegexMatcher::new(&input.pattern).map_err(|e| ToolError::Error {
            message: e.to_string(),
        })?;

        let path = Path::new(&input.path);
        let mut matches = Vec::new();
        let mut searcher = Searcher::new();

        search_path(&matcher, &mut searcher, path, &mut matches)?;

        if matches.is_empty() {
            Ok("No matches found.".to_string())
        } else {
            Ok(matches.join("\n"))
        }
    }
}

fn search_path(
    matcher: &RegexMatcher,
    searcher: &mut Searcher,
    path: &Path,
    matches: &mut Vec<String>,
) -> Result<(), ToolError> {
    if path.is_dir() {
        let entries = fs::read_dir(path).map_err(|e| ToolError::Error {
            message: e.to_string(),
        })?;

        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            // Skip hidden files/directories
            if entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            search_path(matcher, searcher, &entry_path, matches)?;
        }
    } else if path.is_file() {
        let file_path = path.display().to_string();
        let _ = searcher.search_path(
            matcher,
            path,
            UTF8(|line_num, line| {
                matches.push(format!("{}:{}  {}", file_path, line_num, line.trim_end()));
                Ok(true)
            }),
        );
    }
    Ok(())
}
