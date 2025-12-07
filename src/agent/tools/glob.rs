use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{agent::tools::{Tool, tool::ToolInfo, tool_error::ToolError}, util::format_path};

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
            .map(|x| x.display().to_string())
            .collect();
        Ok(v.join("\n"))
    }
}
