use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use crate::agent::tools::{
    Tool,
    tool::{ToolInfo, ToolPreview},
    tool_error::ToolError,
};

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

    fn requires_permission() -> bool {
        true
    }

    fn describe_action(input: &Self::Input) -> String {
        format!("Edit file: {}", input.path)
    }

    async fn generate_preview(input: &Self::Input) -> Option<ToolPreview> {
        // Read the current file content
        let old_file = tokio::fs::read_to_string(&input.path).await.ok()?;

        // Generate what the new file would look like
        let new_file = old_file.replacen(&input.old_string, &input.new_string, 1);

        // If no change would be made, return None
        if old_file == new_file {
            return None;
        }

        // Generate unified diff using similar
        let text_diff = TextDiff::from_lines(&old_file, &new_file);

        let mut diff = String::new();
        for change in text_diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            diff.push_str(&format!("{}{}", sign, change));
        }

        Some(ToolPreview::Edit { diff })
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        let old_file = tokio::fs::read_to_string(input.path.clone()).await?;
        let new_file = old_file.replacen(&input.old_string, &input.new_string, 1);
        tokio::fs::write(input.path, new_file.clone()).await?;
        Ok(new_file)
    }
}
