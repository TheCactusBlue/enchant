use std::fs;
use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{Tool, tool::ToolInfo, tool_error::ToolError};

pub struct Ls;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LsInput {
    /// The directory path to list. Must be an absolute path.
    pub path: String,
}

impl Tool for Ls {
    type Input = LsInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("LS").with_description(include_str!("./ls.md"))
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        let path = Path::new(&input.path);

        if !path.exists() {
            return Err(ToolError::Error {
                message: format!("Path does not exist: {}", input.path),
            });
        }

        if !path.is_dir() {
            return Err(ToolError::Error {
                message: format!("Path is not a directory: {}", input.path),
            });
        }

        let entries = fs::read_dir(path).map_err(|e| ToolError::Error {
            message: e.to_string(),
        })?;

        let mut items: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let path = e.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                if path.is_dir() {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect();

        items.sort();

        if items.is_empty() {
            Ok("(empty directory)".to_string())
        } else {
            Ok(items.join("\n"))
        }
    }
}
