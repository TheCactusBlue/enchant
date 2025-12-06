use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{Tool, tool::ToolInfo, tool_error::ToolError};

pub struct Read;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadInput {
    pub path: String,
}

impl Tool for Read {
    type Input = ReadInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("Read").with_description(include_str!("./read.md"))
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        let res = tokio::fs::read_to_string(input.path).await?;
        Ok(res)
    }
}
