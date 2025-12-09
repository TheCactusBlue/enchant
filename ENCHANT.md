# Enchant - Developer Onboarding Guide

Welcome to Enchant, a terminal-based AI assistant written in Rust. This guide is designed to help you (or future versions of yourself) get started immediately.

## Quick Start

### Prerequisites
- Rust 1.70+ (project uses 2024 edition)
- Anthropic API key

### Setup & Run
```bash
# Set environment variable
cp .env.example .env
# Edit .env and add ENCHANT_KEY=your-api-key

# Build and run
cargo run
```

## Project Overview

**Enchant** is an interactive CLI tool that brings Claude directly to your terminal. It provides:
- Interactive chat interface with Claude
- Tool execution system (read files, write files, run bash, search code, etc.)
- Permission-based safety layer (user approves dangerous operations)
- TUI built with iocraft (React-like component system)
- LLM integration via genai crate

## Architecture Overview

### Core Modules

| Module | Purpose |
|--------|---------|
| `src/main.rs` | Entry point; renders the root App component |
| `src/agent/mod.rs` | **Session** - manages conversation with Claude, tool execution, permission tracking |
| `src/agent/tools/` | Tool implementations (Read, Write, Edit, Bash, Glob, Grep, Ls) |
| `src/agent/tools/tool.rs` | **Tool trait** - defines the interface for all tools |
| `src/agent/config.rs` | Configuration state (API keys, model settings) |
| `src/agent/prompt.rs` | System prompt construction |
| `src/components/` | iocraft UI components (App, InputBox, ThinkingIndicator, Message, etc.) |
| `src/error.rs` | Error types |

### Key Flow

1. User types a message in the CLI
2. Message is sent to `Session` via `message()`
3. `Session.think_step()` calls Claude with tools
4. Claude returns either:
   - A text response (done)
   - Tool calls (possibly requiring permission)
5. If permission required, UI shows `PermissionPrompt`
6. User approves/denies; tools execute with results sent back to Claude
7. Claude continues thinking or responds with final answer

## Understanding Key Components

### Session (`src/agent/mod.rs`)

Manages the conversation lifecycle:
- `messages` - chat history
- `tools` - available tool set
- `pending_calls` - tool calls awaiting permission or execution
- `approved_calls` / `denied_calls` - tracking user decisions

**Key methods:**
- `think_step()` - main loop, returns `ThinkResult` (Done, NeedsPermission, Continue)
- `message()` - add user message
- `approve_permission()` / `deny_permission()` - user decisions

### Tool System (`src/agent/tools/tool.rs`)

The `Tool` trait defines the interface:
```rust
pub trait Tool {
    type Input: Serialize + DeserializeOwned + JsonSchema + Send;
    
    fn get_info() -> ToolInfo;
    async fn execute(input: Self::Input) -> Result<String, ToolError>;
    
    // Optional overrides:
    fn requires_permission() -> bool { false }
    fn describe_action(input: &Self::Input) -> String { ... }
    async fn generate_preview(input: &Self::Input) -> Option<ToolPreview> { None }
}
```

**WrappedTool** trait bridges Tool types to runtime dispatch (using dyn trait).

**Toolset** manages multiple tools and handles dispatch by name.

### UI Components (`src/components/`)

- `App` - root component managing overall state
- `InputBox` - multiline text input with Ctrl+Enter submit
- `ThinkingIndicator` - animated loading (Minecraft enchanting table style)
- `Message` - displays chat messages with ANSI color support
- `PermissionPrompt` - shows tool execution requests with optional previews

## Adding a New Tool

1. **Create file**: `src/agent/tools/my_tool.rs`

2. **Define input struct** (must derive Serialize, Deserialize, JsonSchema):
```rust
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyToolInput {
    pub file_path: String,
    #[serde(default)]
    pub optional_param: Option<String>,
}
```

3. **Implement Tool trait**:
```rust
use async_trait::async_trait;
use crate::agent::tools::tool::{Tool, ToolInfo};
use crate::agent::tools::tool_error::ToolError;

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    type Input = MyToolInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("my_tool")
            .with_description("What this tool does")
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        // Implementation
        Ok("Result".to_string())
    }

    fn requires_permission() -> bool {
        true  // if user approval needed (e.g., for destructive ops)
    }

    fn describe_action(input: &Self::Input) -> String {
        format!("Doing something with {}", input.file_path)
    }

    async fn generate_preview(input: &Self::Input) -> Option<ToolPreview> {
        // Optional: return preview for permission prompts
        None
    }
}
```

4. **Register in Toolset** (`src/agent/mod.rs`):
```rust
pub fn new(config: &ConfigState) -> Self {
    Self {
        // ...
        tools: Arc::new(Toolset::new(vec![
            Box::new(Read),
            Box::new(Glob),
            Box::new(Grep),
            Box::new(Ls),
            Box::new(Edit),
            Box::new(Write),
            Box::new(Bash),
            Box::new(MyTool),  // Add here
        ])),
        // ...
    }
}
```

5. **Add module** to `src/agent/tools/mod.rs`:
```rust
pub mod my_tool;
pub use my_tool::MyTool;
```

## Understanding Permission Flow

Dangerous operations (Edit, Write, Bash) require user approval:

1. Claude generates tool call
2. `requires_permission()` returns true
3. Tool call added to `pending_calls`
4. UI renders `PermissionPrompt` with:
   - Tool name and description
   - Action description (from `describe_action()`)
   - Optional preview (from `generate_preview()`)
5. User selects approve/deny
6. Session records decision in `approved_calls`/`denied_calls`
7. Next `think_step()` executes approved calls or returns error for denied

## Configuration

### Environment Variables
- `ENCHANT_KEY` - Anthropic API key (required)

### Model Selection
Default model is `claude-haiku-4-5`. To change, modify in `src/agent/mod.rs`:
```rust
model: config
    .base
    .default_model
    .clone()
    .unwrap_or("claude-3-5-sonnet-20241022".to_string()),
```

## Build & Development

```bash
cargo build              # Debug build
cargo build --release   # Optimized build
cargo run               # Run with debug build
cargo check             # Type check only (fast)
cargo test              # Run tests
cargo clippy            # Lint warnings
cargo doc --open        # Generate and view docs
```

## Important Files & Their Purposes

| File | Purpose |
|------|---------|
| `Cargo.toml` | Project metadata and dependencies |
| `.env` | Runtime environment (ENCHANT_KEY) |
| `src/agent/prompt.rs` | System prompt that instructs Claude |
| `src/agent/config.rs` | Configuration and auth management |
| `src/error.rs` | Error type wrappers |
| `src/util/mod.rs` | Utility functions |

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `genai` | 0.4.4 | LLM client (Claude support) |
| `iocraft` | 0.7.16 | TUI framework with React-like components |
| `tokio` | 1.48.0 | Async runtime (with full features) |
| `schemars` | 1.1.0 | JSON Schema generation for tool inputs |
| `serde` / `serde_json` | - | Serialization/deserialization |
| `clap` | 4.5.53 | CLI argument parsing |
| `async-trait` | 0.1.89 | Async trait support |

## Common Patterns

### Error Handling
```rust
// Tools return ToolError from tool_error.rs
use crate::agent::tools::tool_error::ToolError;

async fn my_operation() -> Result<String, ToolError> {
    // ... operation ...
    Ok("Success".to_string())
}
```

### Working with Paths
Session stores `working_directory` - use this for relative paths:
```rust
let full_path = self.working_directory.join(&relative_path);
```

### Async Operations
Everything is async (tokio runtime). Use `#[async_trait]` for trait methods:
```rust
#[async_trait]
impl Tool for MyTool {
    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        // async operations here
    }
}
```

## Debugging Tips

1. **See what Claude is thinking**: Check `src/agent/prompt.rs` for the system prompt
2. **Tool not called**: Verify it's registered in Toolset and has correct name
3. **Permission issues**: Check `requires_permission()` and approval logic in `Session`
4. **Parse errors**: Ensure your tool Input struct perfectly matches what Claude sends (check schemars derivation)
5. **Component rendering**: Use iocraft's built-in debugging or print to stderr (println!)

## Testing Locally

```bash
# Set up test environment
export ENCHANT_KEY="your-test-key"

# Run the app
cargo run

# In the CLI, try:
# "Read the file at /Users/hayley/Projects/enchant/README.md"
# "List files in the current directory"
```

## Next Steps

- Review `src/agent/mod.rs` to understand the Session lifecycle
- Look at a simple tool implementation like `src/agent/tools/read.rs`
- Understand the iocraft component model in `src/components/app.rs`
- Run the app and experiment with different requests

---

**Remember**: This is an AI-first project. The system prompt in `src/agent/prompt.rs` is critical - it tells Claude what it can do. When adding features, often you just need to add tools, not change the UI logic.
