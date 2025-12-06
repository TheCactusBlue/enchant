# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build              # Build the project
cargo run                # Run the application (requires .env with ENCHANT_KEY)
cargo check              # Type check without building
cargo test               # Run tests
cargo clippy             # Run lints
```

## Environment Setup

The application requires an `ENCHANT_KEY` environment variable (Anthropic API key) set in a `.env` file at the project root.

## Architecture

Enchant is a terminal-based AI assistant built with Rust, using `iocraft` for the TUI and `genai` for LLM interactions.

### Core Modules

- **`src/main.rs`** - Application entry point and root `App` component using iocraft's component system
- **`src/agent/`** - AI session management and tool execution
  - `Session` manages conversation history and tool calls via the `genai` crate
  - Uses Claude claude-haiku-4-5 model by default
- **`src/agent/tools/`** - Tool system for LLM function calling
  - `Tool` trait defines the interface for tools (input schema via `schemars`, async execute)
  - `WrappedTool` provides runtime dispatch and conversion to `genai::chat::Tool`
  - `Toolset` manages tool registration and invocation by name
- **`src/components/`** - iocraft UI components
  - `InputBox` - Multiline text input with submit handling
  - `ThinkingIndicator` - Animated loading indicator with Minecraft enchanting table characters
  - `COLOR_PRIMARY` - Shared purple accent color (RGB 181, 128, 255)
- **`src/error.rs`** - Error types wrapping `genai::Error`

### Adding New Tools

1. Create a new file in `src/agent/tools/`
2. Define an input struct deriving `Serialize`, `Deserialize`, `JsonSchema`
3. Implement the `Tool` trait with `get_info()` and `execute()`
4. Register the tool in the `Toolset` constructor

### Key Dependencies

- `iocraft` - Declarative TUI framework with React-like components and hooks
- `genai` - LLM client library supporting multiple providers
- `schemars` - JSON Schema generation for tool inputs
- `tokio` - Async runtime
