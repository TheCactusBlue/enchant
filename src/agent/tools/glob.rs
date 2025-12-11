use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
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
        let pattern = glob::Pattern::new(&input.pattern).map_err(|e| ToolError::Error {
            message: e.msg.to_string(),
        })?;

        let root = glob_root(&input.pattern);

        let mut builder = WalkBuilder::new(&root);
        builder
            .follow_links(false)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .ignore(true);

        let mut out = Vec::new();
        for entry in builder.build() {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }

            let p = entry.path();
            if pattern.matches_path(p) {
                out.push(p.display().to_string());
            }
        }

        Ok(out.join("\n"))
    }
}

/// Best-effort extraction of a filesystem root to walk from a glob pattern.
/// This is intentionally simple: it walks from the pattern's leading path
/// components until it hits a glob metachar.
fn glob_root(pattern: &str) -> PathBuf {
    let p = Path::new(pattern);
    let mut root = PathBuf::new();

    for comp in p.components() {
        let s = comp.as_os_str().to_string_lossy();
        // Glob metacharacters. We stop before the first one.
        if s.contains('*') || s.contains('?') || s.contains('[') || s.contains('{') {
            break;
        }
        root.push(comp);
    }

    if root.as_os_str().is_empty() {
        // Fallback: at least keep the parent directory if possible.
        if let Some(parent) = p.parent() {
            return parent.to_path_buf();
        }
        return PathBuf::from("/");
    }

    root
}
