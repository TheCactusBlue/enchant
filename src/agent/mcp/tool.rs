use std::sync::Arc;

use async_trait::async_trait;
use genai::chat::Tool as AITool;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::agent::{
    Session,
    tools::{
        permission::Permission,
        tool::{ToolPreview, WrappedTool},
        tool_error::ToolError,
    },
};

use super::{McpClient, McpServerConfig, McpToolDef};

fn tool_fq_name(server_name: &str, tool_name: &str) -> String {
    format!("mcp.{}.{}", server_name, tool_name)
}

fn parse_permission(s: Option<&str>) -> Permission {
    match s.unwrap_or("require_approval") {
        "implicit" => Permission::Implicit,
        "allow_automatic" => Permission::AllowAutomatic,
        "require_approval" => Permission::RequireApproval,
        "never" => Permission::Never,
        _ => Permission::RequireApproval,
    }
}

/// A dynamic tool backed by an MCP server tool.
pub struct McpTool {
    pub server_name: String,
    pub tool_name: String,
    pub description: Option<String>,
    pub input_schema: Value,

    permission: Permission,
    client: Arc<Mutex<McpClient>>,
}

impl McpTool {
    pub fn new(
        def: McpToolDef,
        server_cfg: &McpServerConfig,
        client: Arc<Mutex<McpClient>>,
    ) -> Self {
        let permission = parse_permission(server_cfg.permission.as_deref());
        Self {
            server_name: def.server,
            tool_name: def.name,
            description: def.description,
            input_schema: def.input_schema,
            permission,
            client,
        }
    }

    fn fq_name(&self) -> String {
        tool_fq_name(&self.server_name, &self.tool_name)
    }
}

#[async_trait]
impl WrappedTool for McpTool {
    async fn call(&self, input: Value) -> Result<String, ToolError> {
        // MCP expects "arguments" object; we pass input as-is.
        let mut client = self.client.lock().await;
        client.call_tool(&self.tool_name, input).await
    }

    fn to_tool(&self) -> AITool {
        AITool {
            name: self.fq_name(),
            description: self.description.clone(),
            schema: Some(self.input_schema.clone()),
            config: None,
        }
    }

    fn requires_permission(
        &self,
        _session: &Session,
        _input: &Value,
    ) -> Result<Permission, ToolError> {
        Ok(self.permission.clone())
    }

    fn describe_action(&self, input: &Value) -> String {
        format!("{}({})", self.fq_name(), input)
    }

    async fn generate_preview(&self, _input: &Value) -> Option<ToolPreview> {
        None
    }
}
