use genai::chat::Tool as AITool;
use schemars::schema_for;

use crate::agent::tools::tool::{Tool, Toolset};

pub mod grep;
pub mod tool;

pub fn build_tool<T: Tool>() -> AITool {
    let info = T::get_info();

    AITool {
        name: info.name,
        description: info.description,
        schema: Some(schema_for!(T::Input).to_value()),
        config: info.config,
    }
}

pub fn get_default_tools() -> Toolset {
    Toolset::new(vec![])
}
