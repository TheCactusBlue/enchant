use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{
    Tool,
    tool::{ToolInfo, ToolPreview},
    tool_error::ToolError,
};

pub struct Write;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WriteInput {
    /// Absolute path to the file that will be created
    pub path: String,
    /// Content of the new file.
    pub content: String,
}

impl Tool for Write {
    type Input = WriteInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("Write").with_description(include_str!("./write.md"))
    }

    fn requires_permission() -> bool {
        true
    }

    fn describe_action(input: &Self::Input) -> String {
        format!("Write file: {}", input.path)
    }

    async fn generate_preview(input: &Self::Input) -> Option<ToolPreview> {
        Some(ToolPreview::Write {
            content: input.content.clone(),
        })
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        tokio::fs::write(input.path, input.content.clone()).await?;
        Ok(input.content)
    }
}
