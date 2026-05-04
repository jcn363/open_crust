# OpenCrust

**OpenCrust** is a production-grade, agentic AI assistant built in Rust. It follows the **OpenCode** specifications for interoperability, safety, and extensibility.

## Features

- **Agentic Core**: Multi-step reasoning with tool-use capabilities.
- **MCP Integration**: Connect to any Model Context Protocol server for expanded tools.
- **LSP Intelligence**: Built-in support for Language Server Protocol (Goto Definition, Hover, References).
- **Rules Engine**: Dynamic project-specific instructions discovered from `.opencode/rules/`.
- **Custom Skills**: Reusable agent instructions from `.opencode/skills/`.
- **Custom Script Tools**: Extend the agent with your own scripts in `.opencode/tools/`.
- **ACP Mode**: Integrated Agent Client Protocol for seamless use within editors like Zed or Neovim.
- **Safety First**: Granular permission system (Allow/Ask/Deny) for all tool executions.
- **Customizable TUI**: Fully configurable keybindings and themes.

## Installation

```bash
cargo install --path .
```

## Configuration

Configuration is stored in `~/.config/open_crust/`:

- `config.json`: Provider settings, model choice, and MCP/LSP server definitions.
- `tui.json`: Keyboard shortcuts.
- `theme.json`: TUI colors and aesthetics.

## Usage

### TUI Mode

Simply run:

```bash
open_crust
```

- Press `i` to enter Insert Mode.
- Press `Esc` to return to Normal Mode.
- Press `Ctrl+C` to exit.

### ACP Mode (Editor Integration)

```bash
open_crust acp
```

## Architecture

OpenCrust is designed with modularity at its core:

- `LlmClient`: Orchestrates the conversation and tool execution loop.
- `McpManager`: Handles connections to external MCP servers.
- `LspManager`: Manages interactions with language servers.
- `PermissionManager`: Gates tool calls based on user policy.
- `CustomToolManager`: Discovers and executes script-based extensions.

## Contributing

We follow the OpenCode standard. Contributions are welcome!
