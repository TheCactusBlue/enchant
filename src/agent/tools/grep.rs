use std::path::Path;

use grep::regex::RegexMatcher;
use grep::searcher::Searcher;
use grep::searcher::sinks::UTF8;
use ignore::WalkBuilder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    agent::tools::{Tool, tool::ToolInfo, tool_error::ToolError},
    util::format_path,
};

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
        ToolInfo::new("Grep").with_description(include_str!("./grep.md"))
    }

    fn describe_action(input: &Self::Input) -> String {
        format!(
            "Grep({}, pattern: {})",
            format_path(&input.path).display(),
            &input.pattern
        )
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
    // WalkBuilder respects:
    // - .gitignore and .git/info/exclude
    // - global gitignore (core.excludesFile) when available
    // - .ignore files
    // and it skips hidden files by default.
    let mut builder = WalkBuilder::new(path);
    builder
        .follow_links(false)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true);

    for entry in builder.build() {
        let entry = entry.map_err(|e| ToolError::Error {
            message: e.to_string(),
        })?;

        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }

        let file_path = entry.path();
        let file_path_display = file_path.display().to_string();

        // If a file can't be searched (e.g. permission denied), just skip it.
        let _ = searcher.search_path(
            matcher,
            file_path,
            UTF8(|line_num, line| {
                matches.push(format!(
                    "{}:{}  {}",
                    file_path_display,
                    line_num,
                    line.trim_end()
                ));
                Ok(true)
            }),
        );
    }

    Ok(())
}
