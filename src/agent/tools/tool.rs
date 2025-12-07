use std::collections::HashMap;

use async_trait::async_trait;
use genai::chat::Tool as AITool;
use schemars::{JsonSchema, schema_for};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::agent::tools::tool_error::ToolError;

/// Preview content for permission prompts
#[derive(Clone, Debug)]
pub enum ToolPreview {
    /// Preview for edit operations showing old and new file content
    Edit { old_file: String, new_file: String },
    /// Preview for write operations showing the new file content
    Write { content: String },
}

pub trait Tool {
    type Input: Serialize + DeserializeOwned + JsonSchema + Send;

    fn get_info() -> ToolInfo;
    fn execute(input: Self::Input)
    -> impl Future<Output = Result<String, ToolError>> + Send + Sync;

    /// Returns true if this tool requires user permission before execution.
    /// Default is false for backwards compatibility.
    fn requires_permission() -> bool {
        false
    }

    /// Returns a human-readable description of the action for permission prompts.
    /// Only called when requires_permission() returns true.
    fn describe_action(_input: &Self::Input) -> String {
        format!("Execute {}", Self::get_info().name)
    }

    /// Generate a preview for permission prompts.
    /// Returns None by default. Tools like Edit and Write can override this.
    fn generate_preview(
        _input: &Self::Input,
    ) -> impl Future<Output = Option<ToolPreview>> + Send + Sync {
        async { None }
    }

    /// Returns a concise display message for the tool call in chat history.
    /// By default returns None, which uses the standard "Tool(json)" format.
    /// Override to provide a custom, more concise display.
    fn display_message(_input: &Self::Input) -> Option<String> {
        None
    }
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

/// A request for user permission to execute a tool.
#[derive(Clone, Debug)]
pub struct PermissionRequest {
    pub call_id: String,
    pub tool_name: String,
    pub description: String,
    pub input: Value,
    /// Optional preview to display for the operation
    pub preview: Option<ToolPreview>,
}

#[async_trait]
pub trait WrappedTool {
    async fn call(&self, input: Value) -> Result<String, ToolError>;
    fn to_tool(&self) -> AITool;
    fn requires_permission(&self) -> bool;
    fn describe_action(&self, input: &Value) -> String;
    async fn generate_preview(&self, input: &Value) -> Option<ToolPreview>;
    fn display_message(&self, input: &Value) -> Option<String>;
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

    async fn call(&self, input: Value) -> Result<String, ToolError> {
        let value: T::Input = serde_json::from_value(input).unwrap();
        Ok(T::execute(value).await?)
    }

    fn requires_permission(&self) -> bool {
        T::requires_permission()
    }

    fn describe_action(&self, input: &Value) -> String {
        match serde_json::from_value::<T::Input>(input.clone()) {
            Ok(typed_input) => T::describe_action(&typed_input),
            Err(_) => format!("Execute {}", T::get_info().name),
        }
    }

    async fn generate_preview(&self, input: &Value) -> Option<ToolPreview> {
        match serde_json::from_value::<T::Input>(input.clone()) {
            Ok(typed_input) => T::generate_preview(&typed_input).await,
            Err(_) => None,
        }
    }

    fn display_message(&self, input: &Value) -> Option<String> {
        match serde_json::from_value::<T::Input>(input.clone()) {
            Ok(typed_input) => T::display_message(&typed_input),
            Err(_) => None,
        }
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

    pub fn requires_permission(&self, name: &str) -> bool {
        self.tools
            .get(name)
            .map(|t| t.requires_permission())
            .unwrap_or(false)
    }

    pub fn describe_action(&self, name: &str, input: &Value) -> String {
        self.tools
            .get(name)
            .map(|t| t.describe_action(input))
            .unwrap_or_else(|| format!("Execute {}", name))
    }

    pub async fn call(&self, name: String, input: Value) -> Result<String, ToolError> {
        let tool = self.tools.get(&name).unwrap();
        Ok(tool.call(input).await?)
    }

    pub async fn generate_preview(&self, name: &str, input: &Value) -> Option<ToolPreview> {
        self.tools.get(name)?.generate_preview(input).await
    }

    pub fn get_display_message(&self, name: &str, input: &Value) -> Option<String> {
        self.tools.get(name)?.display_message(input)
    }
}
