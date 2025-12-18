# Enchant Usage Guide

## MCP Server

```json
{
  "mcp_servers": [
    {
      "name": "my-tools",
      "command": "node",
      "args": ["path/to/server.js"],
      "env": { "FOO": "bar" },
      "permission": "require_approval"
    }
  ]
}
```
