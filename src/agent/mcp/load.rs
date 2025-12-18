use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::agent::tools::{tool::WrappedTool, tool_error::ToolError};

use super::{tool::McpTool, McpClient, McpServerConfig};

/// Spawn all configured MCP servers and return their tools as `WrappedTool`s.
pub async fn load_mcp_tools(
    servers: &[McpServerConfig],
) -> Result<Vec<Box<dyn WrappedTool + Send + Sync + 'static>>, ToolError> {
    let mut out: Vec<Box<dyn WrappedTool + Send + Sync + 'static>> = vec![];

    // Note: sequential startup is simpler and avoids interleaved stdout parsing.
    for cfg in servers {
        if cfg.name.trim().is_empty() {
            return Err(ToolError::Error {
                message: "MCP server config missing 'name'".to_string(),
            });
        }
        if cfg.command.trim().is_empty() {
            return Err(ToolError::Error {
                message: format!("MCP server '{}' missing 'command'", cfg.name),
            });
        }

        let client = McpClient::spawn(cfg).await?;
        let client = Arc::new(Mutex::new(client));

        let defs = {
            let mut guard = client.lock().await;
            guard.list_tools().await?
        };

        // Dedupe by fq name (server+tool)
        let mut seen: HashMap<String, ()> = HashMap::new();

        for def in defs {
            let tool = McpTool::new(def, cfg, client.clone());
            let fq = tool.to_tool().name;
            if seen.contains_key(&fq) {
                continue;
            }
            seen.insert(fq, ());
            out.push(Box::new(tool));
        }
    }

    Ok(out)
}
