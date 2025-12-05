use std::collections::HashMap;

use async_trait::async_trait;
use genai::chat::Tool as AITool;
use schemars::{JsonSchema, schema_for};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub trait Tool {
    type Input: Serialize + DeserializeOwned + JsonSchema;

    fn get_info() -> ToolInfo;
    fn execute(input: Self::Input) -> impl Future<Output = ()> + Send + Sync;
}

pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
}

impl ToolInfo {
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

#[async_trait]
pub trait WrappedTool {
    async fn call(&self, input: Value);
    fn to_tool(&self) -> AITool;
}

#[async_trait]
impl<T: Tool + Sync> WrappedTool for T {
    fn to_tool(&self) -> AITool {
        let info = T::get_info();

        AITool {
            name: info.name,
            description: info.description,
            schema: Some(schema_for!(T::Input).to_value()),
            config: info.config,
        }
    }
    async fn call(&self, input: Value) {
        let value: T::Input = serde_json::from_value(input).unwrap();
        T::execute(value).await;
    }
}

pub struct Toolset {
    order: Vec<AITool>, // order of tools matter for LLM performance
    tools: HashMap<String, Box<dyn WrappedTool + Send + Sync + 'static>>,
}

impl Toolset {
    pub fn new(tools: Vec<Box<dyn WrappedTool + Send + Sync + 'static>>) -> Self {
        let order: Vec<_> = tools.iter().map(|x| x.to_tool()).collect();
        let tools = tools.into_iter().map(|t| (t.to_tool().name, t)).collect();
        Self { order, tools }
    }

    pub fn list_tools(&self) -> Vec<AITool> {
        self.order.clone()
    }

    pub async fn call(&self, name: String, input: Value) {
        let tool = self.tools.get(&name).unwrap();
        tool.call(input).await;
    }
}
