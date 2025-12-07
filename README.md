# Enchant CLI

A terminal-based AI assistant written in Rust that brings Claude directly to your command line. Enchant provides an interactive interface for conversing with Claude while maintaining full control over tool execution through a permission system.

## Features

- **Interactive CLI Interface** - Built with iocraft for a responsive, component-based TUI
- **AI-Powered Tool Execution** - Claude can read files, search code, edit files, run bash commands, and more
- **Permission-Based Safety** - Review and approve tool calls before execution with optional previews
- **File System Integration** - Built-in tools for reading, writing, searching, and navigating files
- **Animated UI** - Loading indicators with Minecraft enchanting table aesthetics

## Getting Started

### Prerequisites

- Rust 1.70+ (uses 2024 edition)
- Anthropic API key

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/enchant.git
cd enchant
```

2. Set up your environment:
```bash
cp .env.example .env
# Edit .env and add your ENCHANT_KEY (Anthropic API key)
```

3. Build and run:
```bash
cargo run
```

## Architecture

### Core Components

- **`src/agent/Session`** - Manages conversation history, tool invocation, and permission tracking via Claude's API
- **`src/agent/tools/`** - Tool system with pluggable implementations:
  - `Read` - Read file contents
  - `Write` - Create new files
  - `Edit` - Modify existing files
  - `Glob` - Find files matching patterns
  - `Grep` - Search file contents
  - `Ls` - List directories
  - `Bash` - Execute shell commands
- **`src/components/`** - UI components built with iocraft:
  - `App` - Main application component with state management
  - `InputBox` - Multiline user input
  - `Message` - Chat message display with ANSI color support
  - `PermissionPrompt` - Permission request UI with optional previews
  - `ThinkingIndicator` - Animated loading state

### Tool Execution Flow

1. User submits a message
2. Claude processes the request and optionally calls tools
3. If tools require permission (e.g., `Edit`, `Write`, `Bash`), a permission prompt appears
4. User approves/denies each tool call
5. Approved tools execute; denied tools return permission error
6. Claude continues reasoning with tool results

### Adding New Tools

1. Create a new file in `src/agent/tools/`:
```rust
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::agent::tools::tool::{Tool, ToolInfo};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MyToolInput {
    pub param: String,
}

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    type Input = MyToolInput;

    fn get_info() -> ToolInfo {
        ToolInfo::new("my_tool")
            .with_description("Description of what this tool does")
    }

    async fn execute(input: Self::Input) -> Result<String, ToolError> {
        // Implementation
        Ok("Result".to_string())
    }

    fn requires_permission() -> bool {
        true // if user approval is needed
    }

    fn describe_action(input: &Self::Input) -> String {
        format!("Performing action with param: {}", input.param)
    }
}
```

2. Register in `src/agent/mod.rs`:
```rust
pub mod tools;
// In Session::new():
Box::new(MyTool),
```

## Configuration

### Environment Variables

- `ENCHANT_KEY` - Anthropic API key (required)

### Model Selection

The default model is `claude-haiku-4-5`. To use a different model, modify the `model` field in `Session::new()` in `src/agent/mod.rs`.

## Development

### Build Commands

```bash
cargo build              # Compile the project
cargo run                # Run the application
cargo check              # Type check without building
cargo test               # Run tests
cargo clippy             # Lint checks
cargo doc --open         # Generate and open documentation
```

### Project Structure

```
src/
├── main.rs              # Application entry point
├── agent/
│   ├── mod.rs          # Session and chat logic
│   ├── prompt.rs       # System prompt configuration
│   └── tools/
│       ├── tool.rs     # Tool trait and framework
│       ├── tool_error.rs
│       └── [tool implementations]
├── components/
│   ├── app.rs          # Root component
│   ├── input_box.rs    # User input component
│   ├── message.rs      # Message display
│   ├── permission_prompt.rs
│   ├── thinking_indicator.rs
│   └── [other UI components]
├── commands/           # CLI argument parsing
├── error.rs            # Error types
└── util/               # Utilities
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `iocraft` | TUI framework with React-like components |
| `genai` | Multi-provider LLM client (Claude support) |
| `schemars` | JSON Schema generation for tool inputs |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `clap` | Command-line argument parsing |

## Limitations & Future Work

- Currently single-session only (no conversation history persistence)
- Tools execute in the current working directory
- Docker sandbox support in dependencies but not integrated

## License

MIT

## Contributing

Contributions welcome! Please ensure:
- Code passes `cargo clippy`
- New tools implement the `Tool` trait properly
- Permission-sensitive tools set `requires_permission() = true`
