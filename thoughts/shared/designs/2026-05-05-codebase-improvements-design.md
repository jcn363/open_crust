---
date: 2026-05-05
topic: "OpenCrust Codebase Improvements"
status: draft
---

# OpenCrust Codebase Improvement Design

## Problem Statement

The OpenCrust codebase has several security vulnerabilities, architectural issues, and code quality problems that need addressing:

1. **Security risks**: Shell command execution in `bash` tool, shell commands for global search/replace
2. **Architectural issues**: `llm.rs` is too large (436 lines) with mixed responsibilities
3. **Error handling**: Silent error ignoring with `let _ =` pattern
4. **Missing validation**: No input sanitization or path traversal protection

## Constraints

- Must maintain backward compatibility with existing config format
- Must keep the TUI functional during refactoring
- Rust 2024 edition with strict warnings-as-errors
- Minimize breaking changes to public APIs

## Approach

I'm taking a three-phase approach:

**Phase 1: Security & Safety (HIGH)**
- Replace shell command execution with Rust-native implementations
- Add path validation and sanitization
- Sandbox bash command execution

**Phase 2: Architecture Refactoring (MEDIUM)**
- Extract tool execution from `llm.rs` into separate module
- Create trait-based abstraction for tools
- Improve error handling with proper error types

**Phase 3: Code Quality (LOWER)**
- Add tests for core modules
- Improve documentation
- Add config validation

## Architecture

### Current State
```
llm.rs (436 lines)
├── LLM API calls (Ollama, OpenRouter, OpenAI)
├── Tool execution (massive match statement)
├── Message history management
└── Permission checking (inline)

tools.rs (309 lines)
├── Tool schema definitions
└── Basic built-in tool execution
```

### Target State
```
llm.rs (~150 lines)
├── LLM API calls
├── Message history management
└── Delegates tool execution to ToolExecutor

tool_executor.rs (new)
├── ToolExecutor trait
├── BuiltInTools implementation
├── MCP tool integration
└── Custom tool integration

tools.rs (~100 lines)
└── Tool schema definitions only

security.rs (new)
├── Path validation
├── Command sanitization
└── Sandbox utilities
```

## Components

### ToolExecutor Trait
```rust
// Conceptual - no code
trait ToolExecutor {
    async fn execute(&self, name: &str, args: &Value) -> String;
    fn get_schemas(&self) -> Value;
}
```

### Security Module
- `validate_path()`: Prevent path traversal attacks
- `sanitize_command()`: Validate bash commands
- `is_safe_path()`: Check against allowed directories

## Data Flow

1. `llm.rs` receives tool call from LLM
2. Delegates to `ToolExecutor.execute()`
3. `ToolExecutor` checks permissions via `PermissionManager`
4. Executes tool with security validation
5. Returns result

## Error Handling Strategy

Replace `let _ =` with:
- Proper `Result` propagation using `?`
- Centralized error type `OpenCrustError`
- Error logging to audit log
- User-friendly error messages in UI

## Testing Strategy

- Unit tests for `PermissionManager`
- Unit tests for path validation
- Unit tests for config loading/saving
- Integration tests for tool execution
- Mock LLM responses for testing

## Open Questions

- Should we use a crate like `shlex` for command parsing?
- Should bash execution be disabled by default?
- How to handle large file operations efficiently?
