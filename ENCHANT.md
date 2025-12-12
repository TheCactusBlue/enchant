# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build              # Compile the project
cargo run                # Run the application
cargo check              # Type check without building
cargo test               # Run tests
cargo clippy             # Lint checks
```

## Configuration

- Config directory: `~/.enchant/`
- Main config: `~/.enchant/enchant.json` (contains `default_model` setting)
- API keys: `~/.enchant/api-keys.json` (keyed by provider: "anthropic", "openai")
- Project-specific prompts: `ENCHANT.md` in working directory (auto-loaded as system message)

## Architecture

This is a terminal-based AI assistant built with Rust. It uses `iocraft` for a React-like component TUI and `genai` for multi-provider LLM access.

### Core Flow

1. `main.rs` renders the `App` component which loads config and spawns `Terminal`
2. `Session` (in `agent/mod.rs`) manages conversation history and tool execution
3. User input triggers `think_step()` which loops until done or needs permission
4. Tools requiring user approval return `ThinkResult::NeedsPermission`
5. `PermissionPrompt` component handles approve/deny UI

### Tool System

Tools are defined in `src/agent/tools/` and implement the `Tool` trait:
- `type Input` - JSON-serializable input struct with `JsonSchema` derive
- `get_info()` - Returns name and description
- `execute()` - Async execution returning `Result<String, ToolError>`
- `requires_permission()` - Returns `Permission` enum (Implicit, RequireApproval, AllowAutomatic, Never)
- `describe_action()` - Human-readable description for permission prompts
- `generate_preview()` - Optional preview for Edit/Write operations

Tools are registered in `Session::new_async()` in `agent/mod.rs`. Order matters for LLM performance.

### Permission Levels

- `Permission::Implicit` - No user approval needed (Read, Glob, Grep, Ls)
- `Permission::RequireApproval` - Always prompts user (Edit, Write)
- `Permission::AllowAutomatic` - Can be auto-approved based on rules
- `Permission::Never` - Tool disabled

### Component Structure

UI components in `src/components/` use iocraft's hook-based pattern:
- `App` - Root component, loads config
- `Terminal` - Main chat interface with state management
- `InputBox` - Multiline user input
- `Message` - Chat message display with ANSI support
- `PermissionPrompt` - Tool approval UI with diff previews
- `ThinkingIndicator` - Animated loading state

### VSCode Extension

`enchant-vscode/` contains a companion VSCode extension (TypeScript). Uses pnpm for package management:
```bash
cd enchant-vscode
pnpm install
pnpm run compile
```
