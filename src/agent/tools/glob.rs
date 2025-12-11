use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    agent::tools::{Tool, tool::ToolInfo, tool_error::ToolError},
    util::format_path,
};

pub struct Glob;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobInput {
    /// Glob pattern. Must use absolute path.
    pub pattern: String,
}

impl Tool for Glob {
    type Input = GlobInput;

    fn describe_action(input: &Self::Input) -> String {
        format!("Glob({})", format_path(&input.pattern).display())
    }

    fn get_info() -> ToolInfo {
        ToolInfo::new("Glob").with_description(include_str!("./glob.md"))
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        // TODO: Port glob to use async
        let v: Vec<_> = glob::glob(&input.pattern)
            .map_err(|x| ToolError::Error {
                message: x.msg.to_string(),
            })?
            .filter_map(|x| x.ok())
            .filter(|path| should_include_path(path))
            .map(|x| x.display().to_string())
            .collect();
        Ok(v.join("\n"))
    }
}

fn should_include_path(path: &Path) -> bool {
    // Skip symlinks to prevent cycles
    if path.is_symlink() {
        return false;
    }

    // Skip hidden files/directories and common large directories
    for component in path.components() {
        if let Some(name) = component.as_os_str().to_str() {
            if name.starts_with('.')
                || name == "target"
                || name == "node_modules"
                || name == "vendor"
                || name == "dist"
                || name == "build"
                || name == ".git"
            {
                return false;
            }
        }
    }

    true
}
