use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::agent::tools::{
    Tool, bash::parse::parse_bash_expr, permission::Permission, tool::ToolInfo,
    tool_error::ToolError,
};

pub mod bashtree;
pub mod parse;

pub struct Bash;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct BashInput {
    /// Bash command to run.
    pub command: String,
}

impl Tool for Bash {
    type Input = BashInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("Bash").with_description(include_str!("./bash.md"))
    }

    fn requires_permission(_session: &crate::agent::Session, input: &Self::Input) -> Result<Permission, ToolError> {
        let perm = if parse_bash_expr(&input.command)?.is_safe() {
            Permission::Implicit
        } else {
            Permission::RequireApproval
        };
        Ok(perm)
    }

    fn describe_action(input: &Self::Input) -> String {
        format!("Bash({})", input.command)
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(input.command)
            .output()
            .await?;
        if output.status.success() {
            Ok(str::from_utf8(&output.stdout).unwrap().to_string())
        } else {
            Err(ToolError::Error {
                message: str::from_utf8(&output.stderr).unwrap().to_string(),
            })
        }
    }
}
