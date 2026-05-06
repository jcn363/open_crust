# OpenCrust

**The fastest, most secure AI coding agent** — built in Rust for terminal-native development.

OpenCrust empowers developers with a high-intelligence, secure, and fully observable AI partner for complex software engineering tasks. Unlike Python or Node.js-based alternatives, OpenCrust leverages Rust's zero-cost abstractions and memory safety to deliver blazing performance with minimal resource footprint.

### Why Rust?

- **⚡ Blazing Fast**: Native compilation means instant startup, sub-millisecond tool execution, and efficient concurrency for multi-agent workflows
- **🛡️ Memory Safe**: Rust's ownership model eliminates entire classes of vulnerabilities (buffer overflows, use-after-free) inherent in C/C++ tools
- **🔒 Security First**: Granular permissions, network gating, and persistent auditing — baked into the architecture, not bolted on
- **📦 Minimal Footprint**: Single static binary, no runtime dependencies, ideal for remote servers and containers

## 🚀 Key Features

### 🧠 Advanced Intelligence

- **Recursive Subagents**: Solves complex problems by spawning and managing specialized sub-agents.
- **Task Planner**: Generates multi-step execution plans with progress tracking in `plan.md`.
- **Semantic Search**: Concept-based retrieval using local heuristic-driven RAG.
- **Web Intelligence**: Integrated search and automated Markdown conversion for live research.
- **Global Refactoring**: Codebase-wide regex search & replace with file-glob scoping.

### 🛠️ Industrial Tooling

- **Full MCP & LSP Support**: Native integration with any Model Context Protocol and Language Server Protocol servers.
- **Runtime Server Addition**: Add new MCP servers interactively without restarting.
- **Custom Scripting**: Create custom tools using any scripting language (Python, Bash, etc.).

### 🛡️ Security & Observability

- **Granular Permissions**: Fine-grained control over file access and command execution.
- **Network Gating**: Domain-level whitelisting for all external web requests.
- **Persistent Auditing**: Every tool call is logged with timestamps, inputs, and results.
- **Usage Tracking**: Real-time token counts and cost estimation in USD.
- **Telemetry Export**: Session metrics exported to `telemetry.json` on exit.

### ⌨️ Professional UX

- **Tabbed Interface**: Switch between `Chat` and `Tasks` views with `[Tab]`.
- **File Tree Sidebar**: Collapsible project navigator with `[Ctrl+B]`.
- **Command History**: Persistent history across sessions; navigate with `[↑]`/`[↓]`.
- **Interactive Diff Viewer**: Approval-gate code modifications with a side-by-side TUI viewer.
- **Context Pinning**: Permanently lock critical files into the agent's context.
- **Customizable TUI**: Configurable keybinds and theme engine with RGB support.

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
  "provider": "gemini",
  "model": "gemini-pro",
  "gemini_api_key": "YOUR_GEMINI_API_KEY",
  "ollama_url": "http://localhost:11434",
  "mcp": {
    "weather": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-weather"],
      "enabled": true
    }
  },
  "lsp": {
    "rust": {
      "command": ["rust-analyzer"],
      "extensions": ["rs"],
      "disabled": false
    }
  },
  "allowed_domains": ["api.brave.com", "github.com"],
  "tui": {
    "keybinds": {
      "leader": "ctrl+x",
      "app_exit": "ctrl+c,ctrl+d",
      "input_submit": "return"
    }
  },
  "theme": {
    "background": "#1e1e2e",
    "foreground": "#cdd6f4",
    "accent": "#89b4fa",
    "border": "#313244"
  }
}
```

Supported providers: `ollama`, `openrouter`, `openai`, `gemini`.

## ⌨️ Keybinds

| Key                 | Action                                    |
|---------------------|-------------------------------------------|
| `i`                 | Enter Insert (input) mode                 |
| `Esc`               | Return to Normal mode                     |
| `Tab`               | Cycle between Chat / Tasks views          |
| `Ctrl+B`            | Toggle file tree sidebar                  |
| `↑` / `↓`           | Navigate command history (in Insert mode) |
| `s`                 | Open Server Management panel              |
| `Enter`             | Submit message                            |
| `a`                 | Approve proposed change                   |
| `d`                 | Deny proposed change                      |
| `Ctrl+C` / `Ctrl+D` | Quit                                      |

## 🏗️ Architecture

```text
open_crust/
├── src/
│   ├── main.rs        # Entry point, TUI event loop
│   ├── app.rs         # Application state, tabs, history
│   ├── ui.rs          # TUI rendering (ratatui)
│   ├── llm.rs         # LLM client, tool execution loop
│   ├── tools.rs       # Tool schema definitions
│   ├── config.rs      # Config loading/saving
│   ├── mcp.rs         # MCP server management (JSON-RPC)
│   ├── lsp.rs         # LSP client (JSON-RPC)
│   ├── skills.rs      # Skill discovery and loading
│   ├── sessions.rs    # Session persistence
│   ├── planner.rs     # Task planner
│   ├── rag.rs         # Local semantic search
│   ├── audit.rs       # Audit logging
│   ├── stats.rs       # Token & cost tracking
│   ├── telemetry.rs   # Session telemetry export
│   ├── permissions.rs # Permission enforcement
│   ├── web.rs         # Web search integration
│   └── acp.rs         # ACP stdio interface
│
│   ├── desktop/       # Cinnamon desktop integration
│   │   ├── mod.rs     # Module entry point
│   │   ├── detection.rs    # Desktop environment detection
│   │   ├── notifications.rs # System notifications
│   │   └── file_picker.rs  # Native file pickers
```

## 🔌 MCP Ecosystem Integration

OpenCrust provides first-class support for the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/), giving you access to **12,000+ community servers** for databases, APIs, productivity tools, and more.

### Quick Start

```bash
# List popular MCP servers
opencrust mcp list

# Install a server (e.g., GitHub integration)
opencrust mcp install github

# Restart open_crust to load the new server
```

### Recommended MCP Servers

**Tier 1: Essential** (Install first)
| Server | Description | Install Command |
|--------|-------------|-----------------|
| `context7` | Version-accurate library docs (eliminates API hallucinations) | `opencrust mcp install context7` |
| `github` | Repos, issues, PRs, CI/CD | `opencrust mcp install github` |
| `postgres` | Natural language DB queries | `opencrust mcp install postgres` |
| `brave-search` | Privacy-focused web search | `opencrust mcp install brave-search` |
| `filesystem` | Enhanced file operations | `opencrust mcp install filesystem` |

**Tier 2: High Value**
| Server | Description | Install Command |
|--------|-------------|-----------------|
| `playwright` | Browser automation & E2E testing | `opencrust mcp install playwright` |
| `supabase` | RLS-aware database access | `opencrust mcp install supabase` |
| `sentry` | Error monitoring | `opencrust mcp install sentry` |
| `linear` | Issue tracking | `opencrust mcp install linear` |
| `e2b` | Secure cloud sandbox for code execution | `opencrust mcp install e2b` |

Browse all servers at [mcpdirectory.app](https://mcpdirectory.app/) (2,500+ servers)

---

## 🧠 Built-in Skills

OpenCrust ships with specialized skills that enhance AI behavior for specific tasks. Skills are automatically discovered from `.opencrust/skills/`.

### Included Skills

| Skill | Description |
|-------|-------------|
| `rust-expert` | Cargo commands, crate selection, unsafe code review, performance optimization |
| `security-auditor` | OWASP Top 10 detection, vulnerability scanning, audit compliance |
| `git-workflow` | Branch management, conventional commits, PR preparation |
| `code-refactorer` | Pattern-based transformations, technical debt identification |

### Adding Custom Skills

Create a SKILL.md file in `.opencrust/skills/<skill-name>/`:

```markdown
---
name: my_skill
description: Does something useful
---

## Instructions

The agent should follow these steps to accomplish the task...
```

See `docs/skills.md` for detailed skill creation guide.

---
Built with 🦀 by the OpenCrust Team.
