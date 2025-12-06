use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{
    Tool,
    tool::{ToolError, ToolInfo},
};

pub struct Glob;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GlobInput {
    /// Glob pattern. Must use absolute path.
    pub pattern: String,
}

impl Tool for Glob {
    type Input = GlobInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("Glob").with_description("Match files using glob patterns. This tool is very fast and will works with even big codebases.")
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        // TODO: Port glob to use async
        let v: Vec<_> = glob::glob(&input.pattern)
            .unwrap()
            .filter_map(|x| x.ok())
            .map(|x| x.display().to_string())
            .collect();
        Ok(v.join("\n"))
    }
}
