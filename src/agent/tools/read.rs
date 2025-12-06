use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::agent::tools::{
    Tool,
    tool::{ToolError, ToolInfo},
};

pub struct Read;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadInput {
    pub path: String,
}

impl Tool for Read {
    type Input = ReadInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("Read").with_description("Reads a file from the local filesystem. You can access any file directly by using this tool. Assume this tool is able to read all files on the machine. If the User provides a path to a file assume that path is valid. It is okay to read a file that does not exist; an error will be returned.")
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        let res = tokio::fs::read_to_string(input.path).await.unwrap();
        Ok(res)
    }
}
