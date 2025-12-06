use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{Tool, tool::ToolInfo, tool_error::ToolError};

pub struct Edit;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EditInput {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
}

impl Tool for Edit {
    type Input = EditInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("Edit").with_description(include_str!("./edit.md"))
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        let old_file = tokio::fs::read_to_string(input.path.clone()).await?;
        let new_file = old_file.replacen(&input.old_string, &input.new_string, 1);
        tokio::fs::write(input.path, new_file.clone()).await?;
        Ok(new_file)
    }
}
