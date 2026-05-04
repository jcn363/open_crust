# OpenCrust

**OpenCrust** is a production-grade, TUI-native agentic coding platform built in Rust. It empowers developers with a high-intelligence, secure, and fully observable AI partner for complex software engineering tasks.

## 🚀 Key Features

### 🧠 Advanced Intelligence

- **Recursive Subagents**: Solves complex problems by spawning and managing specialized sub-agents.
- **Task Planner**: Generates multi-step execution plans with progress tracking in `plan.md`.
- **Semantic Search**: Concept-based retrieval using local heuristic-driven RAG.
- **Web Intelligence**: Integrated search and automated Markdown conversion for live research.

### 🛠️ Industrial Tooling

- **Full MCP & LSP Support**: Native integration with any Model Context Protocol and Language Server Protocol servers.
- **Custom Scripting**: Create custom tools using any scripting language (Python, Bash, etc.).
- **Global Refactoring**: Perform codebase-wide regex transformations with safety-guaranteed inclusion globs.

### 🛡️ Security & Observability

- **Granular Permissions**: Fine-grained control over file access and command execution.
- **Network Gating**: Domain-level whitelisting for all external web requests.
- **Persistent Auditing**: Every tool call is logged with timestamps, inputs, and results.
- **Usage Tracking**: Real-time token counts and cost estimation in USD.

### ⌨️ Professional UX

- **Interactive Diff Viewer**: Approval-gate code modifications with a side-by-side TUI viewer.
- **Context Pinning**: Permanently lock critical files into the agent's context.
- **Customizable TUI**: Fully configurable keybinds and theme engine with RGB support.

## 📦 Installation

```bash
git clone https://github.com/opencrust/open_crust.git
cd open_crust
cargo install --path .
```

## ⚙️ Configuration

Configure your environment in `~/.config/open_crust/config.json`:

```json
{
  "provider": "ollama",
  "model": "llama3",
  "mcp": {
    "weather": { "command": "npx", "args": ["-y", "@modelcontextprotocol/server-weather"], "enabled": true }
  },
  "allowed_domains": ["api.brave.com", "github.com"]
}
```

## ⌨️ Quick Start Keybinds

- `i`: Enter Insert Mode
- `s`: Open Server Management
- `Esc`: Back to Normal Mode
- `Ctrl+C`: Exit App (with Telemetry Export)

---
Built with 🦀 by the OpenCrust Team.
