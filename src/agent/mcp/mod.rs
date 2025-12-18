use std::{collections::HashMap, process::Stdio};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
};

use crate::agent::tools::tool_error::ToolError;

pub mod load;
pub mod tool;

/// MCP stdio server configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Permission policy applied to all tools from this server.
    /// One of: "implicit", "allow_automatic", "require_approval", "never".
    #[serde(default)]
    pub permission: Option<String>,
}

#[derive(Clone, Debug)]
pub struct McpToolDef {
    pub server: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

/// Minimal MCP stdio client (JSON-RPC over newline-delimited stdio).
///
/// Supported subset:
/// - initialize / initialized
/// - tools/list
/// - tools/call
pub struct McpClient {
    server_name: String,
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    next_id: u64,
}

impl McpClient {
    pub async fn spawn(cfg: &McpServerConfig) -> Result<Self, ToolError> {
        let mut cmd = Command::new(&cfg.command);
        cmd.args(&cfg.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        for (k, v) in &cfg.env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().ok_or(ToolError::Error {
            message: "Failed to open stdin for MCP server".to_string(),
        })?;
        let stdout = child.stdout.take().ok_or(ToolError::Error {
            message: "Failed to open stdout for MCP server".to_string(),
        })?;

        let mut client = Self {
            server_name: cfg.name.clone(),
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };

        client.initialize().await?;
        Ok(client)
    }

    async fn write_jsonrpc(&mut self, method: &str, params: serde_json::Value) -> Result<u64, ToolError> {
        let id = self.next_id;
        self.next_id += 1;

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let mut line = serde_json::to_vec(&req).map_err(|e| ToolError::Error {
            message: format!("MCP serialize error: {e}"),
        })?;
        line.push(b'\n');

        self.stdin.write_all(&line).await?;
        self.stdin.flush().await?;
        Ok(id)
    }

    async fn read_response(&mut self, id: u64) -> Result<serde_json::Value, ToolError> {
        loop {
            let mut buf = String::new();
            let n = self.stdout.read_line(&mut buf).await?;
            if n == 0 {
                return Err(ToolError::Error {
                    message: format!("MCP server '{}' closed stdout", self.server_name),
                });
            }

            let v: serde_json::Value = serde_json::from_str(&buf).map_err(|e| ToolError::Error {
                message: format!("MCP parse error: {e}. line={buf:?}"),
            })?;

            if v.get("id").and_then(|x| x.as_u64()) != Some(id) {
                continue;
            }

            if let Some(err) = v.get("error") {
                return Err(ToolError::Error {
                    message: format!("MCP error from '{}': {}", self.server_name, err),
                });
            }

            return Ok(v.get("result").cloned().unwrap_or(serde_json::Value::Null));
        }
    }

    async fn initialize(&mut self) -> Result<(), ToolError> {
        let id = self
            .write_jsonrpc(
                "initialize",
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "clientInfo": { "name": "enchant", "version": env!("CARGO_PKG_VERSION") }
                }),
            )
            .await?;
        let _ = self.read_response(id).await?;

        // Best-effort notification.
        let _ = self.write_jsonrpc("initialized", serde_json::json!({})).await;
        Ok(())
    }

    pub async fn list_tools(&mut self) -> Result<Vec<McpToolDef>, ToolError> {
        let id = self.write_jsonrpc("tools/list", serde_json::json!({})).await?;
        let result = self.read_response(id).await?;

        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or(ToolError::Error {
                message: format!("MCP '{}' tools/list returned unexpected shape: {result}", self.server_name),
            })?;

        let mut defs = vec![];
        for t in tools {
            let name = t.get("name").and_then(|x| x.as_str()).ok_or(ToolError::Error {
                message: format!("MCP '{}' tool missing name: {t}", self.server_name),
            })?;
            let description = t
                .get("description")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());
            let input_schema = t
                .get("inputSchema")
                .cloned()
                .unwrap_or(serde_json::json!({ "type": "object" }));

            defs.push(McpToolDef {
                server: self.server_name.clone(),
                name: name.to_string(),
                description,
                input_schema,
            });
        }
        Ok(defs)
    }

    pub async fn call_tool(&mut self, tool_name: &str, arguments: serde_json::Value) -> Result<String, ToolError> {
        let id = self
            .write_jsonrpc(
                "tools/call",
                serde_json::json!({
                    "name": tool_name,
                    "arguments": arguments,
                }),
            )
            .await?;
        let result = self.read_response(id).await?;

        if let Some(content) = result.get("content") {
            if let Some(arr) = content.as_array() {
                let mut out = String::new();
                for block in arr {
                    if block.get("type").and_then(|x| x.as_str()) == Some("text") {
                        if let Some(t) = block.get("text").and_then(|x| x.as_str()) {
                            if !out.is_empty() {
                                out.push('\n');
                            }
                            out.push_str(t);
                        }
                    }
                }
                if !out.is_empty() {
                    return Ok(out);
                }
            }
            return Ok(content.to_string());
        }

        Ok(result.to_string())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}
