# OpenCrust

**The fastest, most secure AI coding agent** — built in Rust for terminal-native development.

OpenCrust is a high-intelligence AI partner for complex software engineering. Unlike Python or Node.js-based alternatives, it leverages Rust's zero-cost abstractions and memory safety to deliver instant startup and minimal resource footprint. Every feature — from subagent orchestration to LSP integration to network gating — is compiled into a single static binary with no runtime dependencies.

### Why Rust?

- **⚡ Blazing Fast**: Native compilation means instant startup, sub-millisecond tool execution, and efficient concurrency for multi-agent workflows
- **🛡️ Memory Safe**: Rust's ownership model eliminates entire classes of vulnerabilities (buffer overflows, use-after-free) inherent in C/C++ tools
- **🔒 Security First**: Granular permissions, network gating, persistent auditing — baked into the architecture, not bolted on
- **📦 Minimal Footprint**: Single static binary, ideal for remote servers, containers, and resource-constrained environments

---

## 🚀 Key Features

### 🧠 Advanced Intelligence

- **Recursive Subagents**: Spawns and manages specialized sub-agents for complex, multi-step problems
- **Multi-Agent Orchestration**: Dedicated `orchestrator/` module for coordinating agent pools with task delegation
- **Task Planner**: Generates multi-step execution plans with progress tracking in `plan.md`
- **Semantic Search**: Vector-based code search using Ollama embeddings (`nomic-embed-text`). Index with `index_codebase`, then search with `semantic_search`
- **Web Intelligence**: Integrated search and automated Markdown conversion for live research
- **Global Refactoring**: Codebase-wide regex search & replace with file-glob scoping
- **Skill System**: Load specialized behavior profiles (skills) that reshape the agent's capabilities per-task

### 🛠️ Industrial Tooling

- **Full MCP & LSP Support**: Native integration with 2,500+ Model Context Protocol and Language Server Protocol servers
- **LSP Features**: Code completion, diagnostics (errors/warnings), code formatting via `textDocument/*` methods
- **Runtime Server Addition**: Add new MCP servers interactively without restarting
- **MCP Showcase Browser**: TUI browser to manage, enable/disable MCP servers in real-time (`Ctrl+M`)
- **Custom Scripting**: Create custom tools using any scripting language (Python, Bash, etc.), auto-discovered from `.opencrust/tools/`
- **ACP stdio Interface**: Agent Communication Protocol support for multi-agent interoperability
- **Git Integration**: Native git operations, branch management, and commit automation

### 👮 Security & Observability

- **Granular Permissions**: Fine-grained control over file access and command execution
- **Network Gating**: Domain-level whitelisting for all external web requests
- **Persistent Auditing**: Every tool call logged with timestamps, inputs, and results
- **Usage Tracking**: Real-time token counts and cost estimation in USD
- **Telemetry Export**: Session metrics exported to `telemetry.json` on exit
- **Security Module**: Additional security boundaries for sandboxed execution

### ⌨️ Professional UX

- **Tabbed Interface**: Switch between `Chat`, `Tasks`, and `MCP Showcase` views with `Tab`
- **File Tree Sidebar**: Collapsible project navigator with `Ctrl+B`
- **Command Palette**: Quick actions (provider switching, stats, context clear) with `Ctrl+K`
- **Plan Mode**: Review LLM-proposed changes file-by-file before approving (`P`)
- **Vim Mode**: Modal input editing with `Alt+V` toggle, supports h/j/k/l navigation and yank/delete
- **Background Agents**: Async task execution with dedicated `Tasks` tab (`Ctrl+T`)
- **Skill Browser**: Toggle skills active/inactive (`Ctrl+Shift+K` or `S` in normal mode)
- **Input Prediction**: Ghost text suggestions — `Tab` to accept, `Esc` to dismiss
- **Interactive Diff Viewer**: Approval-gate code modifications with a side-by-side TUI viewer
- **Context Pinning**: Permanently lock critical files into the agent's context
- **Context Budget Display**: Real-time token count, budget %, model name, and cost in status bar
- **Command History**: Persistent history across sessions; navigate with `↑` / `↓`
- **Customizable TUI**: Configurable keybinds and theme engine with RGB support (Catppuccin Mocha default)
- **Mission Control**: Visualize orchestrator task DAG with interactive navigation (`Ctrl+G`)

### 🖥️ Desktop Integration (Linux)

- **Smart Notifications**: DBus-first notifications with notify-send fallback for maximum compatibility
- **Native File Pickers**: Nemo (Cinnamon), Zenity, or KDialog backends with automatic detection
- **Desktop Detection**: Automatic Cinnamon/MATE/GNOME/Plasma environment detection with theme extraction
- **CLI Commands**: `opencrust desktop` and `opencrust session` for terminal-native workflows

---

## 📦 Installation

```bash
git clone https://github.com/opencrust/open_crust.git
cd open_crust
cargo install --path .
```

### Quick Start

```bash
# Launch the TUI
opencrust

# One-shot prompt without entering the TUI
opencrust -p "Explain the Rust ownership model"

# Start with a specific provider and model
opencrust --provider gemini --model gemini-2.0-flash
```

---

## ⚙️ Configuration

Configure your environment at `~/.config/open_crust/config.json`:

```json
{
  "provider": "gemini",
  "model": "gemini-2.0-flash",
  "gemini_api_key": "YOUR_GEMINI_API_KEY",
  "ollama_url": "http://localhost:11434",
  "mcp": {
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
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
      "app_exit": "ctrl+q",
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

**Supported providers (11):** `ollama`, `openrouter`, `openai`, `gemini`, `mistral`, `anthropic`, `groq`, `togetherai`, `replicate`, `deepseek`, `localai`.

Use any provider with model aliases like `big-pickle`, `fast`, or `powerful` via the `model_aliases` config section. Subagent-specific provider configs are also supported for multi-agent routing.

---

## ⌨️ Keybinds

| Key                 | Action                                    |
|---------------------|-------------------------------------------|
| `i`                 | Enter Insert (input) mode                 |
| `Esc`               | Return to Normal / dismiss                |
| `Tab`               | Cycle between Chat / Tasks / MCP views    |
| `Ctrl+B`            | Toggle file tree sidebar                  |
| `Ctrl+K`            | Open Command Palette                      |
| `Ctrl+M`            | MCP Showcase browser                      |
| `Ctrl+T`            | Open Background Agents tab                |
| `Ctrl+Shift+K`      | Open Skill Browser                        |
| `Alt+V`             | Toggle Vim Mode                           |
| `p` / `P`           | Enter Plan Mode                            |
| `k`                 | Command Palette (normal mode)             |
| `m`                 | MCP Showcase (normal mode)                |
| `s` / `S`           | Skill Browser / Server panel              |
| `a`                 | Approve proposed change                   |
| `d`                 | Deny proposed change                      |
| `Shift+A`           | Approve all proposed changes              |
| `↑` / `↓`           | Navigate command history / lists          |
| `Enter`             | Submit message / confirm                  |
| `Ctrl+Q`            | Quit                                      |

> **Note:** Default keybinds shown above. All keybinds are fully customizable in `config.json` under `tui.keybinds`.

---

## 🏗️ Architecture

```text
open_crust/
├── src/
│   ├── main.rs            # Entry point, CLI args, TUI event loop
│   ├── app.rs             # Application state, tabs, history, background tasks
│   ├── ui.rs              # TUI rendering (ratatui)
│   │
│   ├── llm.rs             # LLM client, tool execution loop, context management
│   ├── tools.rs           # Tool schema definitions and routing
│   ├── config.rs          # Config loading/saving (11 providers, model_aliases, subagent cfg)
│   ├── rules.rs           # Steering rules / AGENTS.md loading for context injection
│   ├── context.rs         # Context management (@file syntax, pinning, budget)
│   ├── skills.rs          # Skill discovery, loading, activation/deactivation
│   │
│   ├── mcp.rs             # MCP server management (JSON-RPC transport)
│   ├── mcp_showcase/      # MCP Showcase TUI browser
│   │   ├── mod.rs
│   │   └── tui.rs         # MCP Showcase TUI component
│   ├── lsp.rs             # LSP client (JSON-RPC) with completion, diagnostics, formatting
│   ├── custom_tools.rs    # Custom tool discovery from .opencrust/tools/
│   │
│   ├── planner.rs         # Task planner (multi-step plan generation)
│   ├── rag.rs             # Local semantic search (vector-based code search)
│   ├── web.rs             # Web search integration with markdown conversion
│   ├── git.rs             # Git operations (branch, commit, PR)
│   │
│   ├── permissions.rs     # Permission enforcement (file access, command exec)
│   ├── security.rs        # Additional security module
│   ├── audit.rs           # Audit logging (every tool call)
│   ├── stats.rs           # Token & cost tracking
│   ├── telemetry.rs       # Session telemetry export
│   │
│   ├── sessions.rs        # Session persistence (save/load/list/fork)
│   ├── markdown.rs        # Markdown rendering
│   ├── events.rs          # Event bus and handler dispatch
│   ├── status_bar.rs      # Status bar component (model, tokens, cost)
│   ├── clipboard.rs       # Clipboard integration
│   ├── formatters.rs      # Auto-formatter integration
│   ├── json_utils.rs      # JSON path utilities
│   ├── jsonrpc.rs         # JSON-RPC protocol primitives
│   ├── acp.rs             # Agent Communication Protocol (ACP) stdio interface
│   │
│   ├── orchestrator/      # Multi-agent orchestration
│   │   ├── mod.rs         # Coordinator entry point
│   │   ├── task.rs        # Task representation and state
│   │   └── agent_pool.rs  # Agent pool management
│   │
│   └── desktop/           # Desktop integration
│       ├── mod.rs
│       ├── detection.rs   # Desktop environment detection
│       ├── notifications.rs # System notifications (DBus + notify-send)
│       └── file_picker.rs # Native file pickers (Nemo, Zenity, KDialog)
```

---

## 🔌 MCP Ecosystem Integration

OpenCrust provides first-class support for the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/), giving you access to **2,500+ community servers** for databases, APIs, productivity tools, and more.

### MCP Showcase (TUI Browser)

Press `Ctrl+M` (or `M` in normal mode) to open the **MCP Showcase** — a terminal-native browser for managing your MCP servers:

- **Browse**: See all configured MCP servers with their status (enabled/disabled)
- **Toggle**: Press `Enter` to enable/disable any server
- **Navigate**: Use `↑` / `↓` arrows to move through the list
- **Changes** take effect immediately and are saved to your config automatically

### Quick Start

```bash
# List popular MCP servers
opencrust mcp list

# Install a server (e.g., GitHub integration)
opencrust mcp install github
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

OpenCrust ships with 9 specialized skills that reshape AI behavior for specific tasks. Skills are auto-discovered from `.opencrust/skills/`.

| Skill | Description |
|-------|-------------|
| `rust-expert` | Cargo commands, crate selection, unsafe code review, performance optimization |
| `security-auditor` | OWASP Top 10 detection, vulnerability scanning, audit compliance |
| `git-workflow` | Branch management, conventional commits, PR preparation |
| `code-refactorer` | Pattern-based transformations, technical debt identification |
| `api-integrator` | REST/GraphQL client generation, auth flows, SDK integration |
| `test-generator` | Unit/integration test scaffolding, coverage analysis, fuzzing harnesses |
| `docs-writer` | API reference generation, architecture docs, README automation |
| `perf-profiler` | Criterion benchmarks, flamegraph generation, bottleneck analysis |
| `dep-manager` | Cargo dependency auditing, license compliance, version conflict resolution |

### Adding Custom Skills

Create a `SKILL.md` file in `.opencrust/skills/<skill-name>/`:

```markdown
---
name: my_skill
description: Does something useful
---

## Instructions

The agent should follow these steps to accomplish the task...
```

Skills can be activated/deactivated via the Skill Browser (`Ctrl+Shift+K`) or the CLI: `opencrust skills activate <name>`.

---

## 🔧 Custom Tools & Linters

OpenCrust auto-discovers executable scripts in `.opencrust/tools/`. Built-in linter tools include:

| Tool | Description |
|------|-------------|
| `clippy-check` | Runs `cargo clippy -- -D warnings` for strict lint compliance |
| `fmt-check` | Verifies code formatting with `cargo fmt -- --check` |

Add your own tools by creating an executable script with a comment header:

```bash
#!/bin/bash
# name: my-tool
# description: What it does

echo "Running my tool..."
```

---

## 🖥️ CLI Reference

### Desktop

```bash
opencrust desktop detect                    # Detect DE (Cinnamon/MATE/GNOME/Plasma)
opencrust desktop notify --title "X" --body "Y"  # Send notification
opencrust desktop file-picker [--directory]      # Native file picker
```

### Sessions

```bash
opencrust session list                      # List saved sessions
opencrust session show <id>                 # Show session details
opencrust session save --name "tag"         # Save current session
opencrust session delete <id>               # Delete a session
opencrust session fork <id>                 # Fork a session
```

### MCP

```bash
opencrust mcp list                          # List available MCP servers
opencrust mcp install <name>                # Install an MCP server
opencrust mcp remove <name>                 # Remove an MCP server
```

### Skills

```bash
opencrust skills list                       # List all skills
opencrust skills activate <name>            # Activate a skill
opencrust skills deactivate <name>          # Deactivate a skill
opencrust skills stats                      # Show skill usage stats
```

### Audit

```bash
opencrust audit export --from <DATE> --to <DATE> [--format csv|json] [--output <FILE>]
opencrust audit query [--from <DATE>] [--to <DATE>] [--action <PATTERN>] [--status approved|denied]
opencrust audit evidence [--output-dir <DIR>]
opencrust audit report [--from <DATE>] [--to <DATE>]
```

### Serve & Headless

```bash
opencrust serve --stdio                     # MCP server mode (JSON-RPC stdio)
opencrust -p "prompt"                       # One-shot prompt
opencrust -f prompt.txt                     # Prompt from file
opencrust -p "..." --project /path          # Override project directory
opencrust -p "..." --provider ollama        # Override provider
opencrust -p "..." --model llama3           # Override model
opencrust --agent planner                   # Launch in subagent mode
```

---

## 📚 Documentation Structure

OpenCrust documentation is organized by use case and audience. Start with **Tier 1** — it contains everything you need to get productive.

### 🟢 Tier 1: Essential (Start Here)

**For first-time users:**
- **[CONTRIBUTING.md](./CONTRIBUTING.md)** — Setup, development environment, PR workflow, testing checklist
- **[docs/MODULES.md](./docs/MODULES.md)** — Reference guide to all 40+ source modules; lookup table for "where do I change X?"

**For daily usage:**
- **[docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md)** — How-to guides: add custom tools, create skills, integrate MCP servers, add CLI commands, extend TUI, add config options, setup LSP
- **[docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md)** — Problem-solution guide for installation, runtime, config, performance, and feature issues

### 🟡 Tier 2: Advanced (Deep Dives)

**For understanding internals:**
- **[docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)** — System design, data flow, concurrency model, module interactions, security boundaries; includes ASCII diagrams
- **[docs/SECURITY.md](./docs/SECURITY.md)** — Permissions model, audit system, network gating, sandboxing, secrets management, threat model

**For configuration & optimization:**
- **[docs/CONFIGURATION.md](./docs/CONFIGURATION.md)** — Complete config reference; all 11 LLM providers with examples; MCP/LSP setup; permissions and security settings
- **[docs/PERFORMANCE.md](./docs/PERFORMANCE.md)** — Profiling, startup/runtime/memory optimization, benchmarking, tuning for production

### 🔵 Tier 3: Practical (Copy-Paste)

**For hands-on tutorials:**
- **[docs/EXAMPLES.md](./docs/EXAMPLES.md)** — 8 copy-paste-ready tutorials: custom tool, Ollama setup, GitHub MCP, debugging, custom skill, multi-model workflow, keybindings, production lockdown
- **[docs/TESTING.md](./docs/TESTING.md)** — Unit tests, integration tests, property-based tests, mocking, async testing, benchmarks, PR verification checklist

### Repository Guidelines

- **[AGENTS.md](./AGENTS.md)** — Coding standards, module organization, error handling, testing expectations, commit conventions
- **[docs/MCP_SERVERS.md](./docs/MCP_SERVERS.md)** — Complete reference for all 2,500+ MCP servers
- **[docs/skills.md](./docs/skills.md)** — How to create and customize skills

---

### Quick Navigation

**I want to...**

| Goal | Document |
|------|----------|
| Get started with OpenCrust | [CONTRIBUTING.md](./CONTRIBUTING.md) |
| Add a custom tool | [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md#1-adding-a-custom-tool) → [docs/EXAMPLES.md](./docs/EXAMPLES.md#example-1-create-your-first-custom-tool) |
| Set up local LLM (Ollama) | [docs/EXAMPLES.md](./docs/EXAMPLES.md#example-2-set-up-ollama-for-local-llm) |
| Integrate GitHub | [docs/EXAMPLES.md](./docs/EXAMPLES.md#example-3-add-github-integration-via-mcp) |
| Debug a failing tool | [docs/EXAMPLES.md](./docs/EXAMPLES.md#example-4-debug-a-failing-tool) |
| Create a skill | [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) → [docs/EXAMPLES.md](./docs/EXAMPLES.md#example-5-create-a-custom-skill) |
| Configure everything | [docs/CONFIGURATION.md](./docs/CONFIGURATION.md) |
| Understand how it works | [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) |
| Security & permissions | [docs/SECURITY.md](./docs/SECURITY.md) |
| Fix a problem | [docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md) |
| Optimize performance | [docs/PERFORMANCE.md](./docs/PERFORMANCE.md) |
| Write tests | [docs/TESTING.md](./docs/TESTING.md) |
| Find where code lives | [docs/MODULES.md](./docs/MODULES.md) |

---

Built with 🦀 by the OpenCrust Team.
