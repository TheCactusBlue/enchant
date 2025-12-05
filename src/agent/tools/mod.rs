use genai::chat::Tool;
use schemars::{JsonSchema, schema_for};

use crate::agent::tools::grep::Grep;

pub mod grep;

pub trait EnchantTool {
    type Input: JsonSchema;

    fn get_info() -> EnchantToolInfo;
    fn execute(input: Self::Input) -> impl Future<Output = ()> + Send + Sync;
}

pub struct EnchantToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
}

impl EnchantToolInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            config: None,
        }
    }

    pub fn with_description(self, description: &str) -> Self {
        Self {
            description: Some(description.to_string()),
            ..self
        }
    }
}

pub fn build_tool<T: EnchantTool>() -> Tool {
    let info = T::get_info();

    Tool {
        name: info.name,
        description: info.description,
        schema: Some(schema_for!(T::Input).to_value()),
        config: info.config,
    }
}

pub fn get_default_tools() -> Vec<Tool> {
    vec![build_tool::<Grep>()]
}
