# Architecture Overview

```text
opencrust/
├── src/
│   ├── lib.rs             # Library crate (pub mod declarations, shared logic)
│   ├── main.rs            # Thin entry point, delegates to lib.rs
│   ├── app.rs             # Application state, tabs, history, background tasks
│   ├── ui.rs              # TUI rendering (ratatui)
│   │
│   ├── auth.rs            # GitHub Copilot / ChatGPT Plus OAuth device flow
│   ├── memory.rs          # Auto memory system (conversation persistence)
│   ├── recursive_agents.rs # Recursive sub-agent management (5 levels deep)
│   │
│   ├── desktop/           # Desktop integration (Linux, macOS, Windows)
│   │   ├── mod.rs
│   │   ├── detection.rs   # Desktop environment detection (Cinnamon, MATE, Plasma, GNOME, macOS, Windows)
│   │   ├── file_picker.rs # Native file pickers (Nemo, Zenity, KDialog, osascript)
│   │   ├── notifications/ # System notifications (notify-send, osascript)
│   │   └── menu_bar.rs    # macOS menu bar integration (cocoa/objc)
│   │
│   ├── providers/         # Provider abstraction layer
│   │   ├── mod.rs         # Generic Provider trait and registry
│   │   ├── desktop.rs     # Desktop provider trait
│   │   ├── notifications.rs # Notification provider trait
│   │   ├── file_picker.rs # File picker provider trait
│   │   ├── menu_bar.rs    # Menu bar provider trait
│   │   ├── tool.rs        # Tool provider trait
│   │   └── plugin.rs      # Plugin provider trait
│   │
│   ├── llm.rs             # LLM client, tool execution loop, context management
│   ├── tools.rs           # Tool schema definitions and routing
│   ├── config.rs          # Config loading/saving (34 providers, model_aliases, subagent cfg)
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
│   ├── sessions.rs        # Session persistence (save/load/list/fork)
│   ├── markdown.rs        # Markdown rendering
│   ├── events.rs          # Event bus and handler dispatch
│   ├── status_bar.rs      # Status bar component (model, tokens, cost)
│   ├── clipboard.rs       # Clipboard integration
│   ├── formatters.rs      # Auto-formatter integration
│   ├── json_utils.rs      # JSON path utilities
│   ├── jsonrpc.rs         # JSON-RPC protocol primitives
│   ├── acp.rs             # Agent Communication Protocol (ACP) stdio interface
│   ├── logging.rs         # Centralized logging setup
│   │
│   ├── models.rs          # Model list caching & fetcher
│   ├── compliance.rs      # Compliance & evidence package generation
│   │
│   ├── event_loop/        # TUI event loop and mode handlers
│   │   ├── mod.rs         # Main event loop, global key handling, mode dispatch
│   │   ├── modes/         # Mode-specific key handlers
│   │   │   ├── mod.rs     # dispatch_mode() router
│   │   │   ├── types.rs   # ModeHandler trait, ModeAction, HandlerContext
│   │   │   ├── normal.rs  # Normal mode navigation
│   │   │   ├── insert.rs  # Insert mode editing
│   │   │   ├── review.rs  # Review mode diff navigation
│   │   │   ├── servers.rs # MCP server browser
│   │   │   ├── skill_browser.rs # Skill browser
│   │   │   ├── plugin_browser.rs # Plugin browser
│   │   │   ├── command_palette.rs # Command palette
│   │   │   ├── help.rs    # Help mode
│   │   │   ├── mcp_showcase.rs # MCP showcase mode
│   │   │   └── mission_control.rs # Mission control mode
│   │   ├── keybinds.rs    # Keybind matching utilities
│   │   ├── slash_commands.rs # Slash command handling
│   │   ├── response_handler.rs # LLM response processing
│   │   ├── background_tasks.rs # Background task notifications
│   │   ├── input_prediction.rs # Input prediction (300ms debounce)
│   │   ├── skill_hot_reload.rs # Skill hot-reload detection
│   │   └── frame_limiter.rs # Frame rate limiting
│   │
│   ├── orchestrator/      # Multi-agent orchestration
│   │   ├── mod.rs         # Coordinator entry point
│   │   ├── task.rs        # Task representation and state
│   │   └── agent_pool.rs  # Agent pool management
│   │
│   ├── mission_control/   # Mission Control hub
│   │   ├── mod.rs         # Module exports
│   │   ├── state.rs       # MissionControlUI state management
│   │   ├── render.rs      # TUI rendering
│   │   ├── controls.rs    # Control components
│   │   ├── progress.rs    # Progress visualization
│   │   ├── types.rs       # Common types (Node, Edge, ViewMode, SpacePanel)
│   │   ├── spaces.rs      # Spaces/Projects - group agents and tasks by project
│   │   ├── dashboard.rs   # Agent Dashboard - real-time agent monitoring
│   │   ├── artifacts.rs   # Artifacts - generate/manage docs, test results
│   │   ├── workflows.rs   # Workflows - parameterized command library
│   │   └── scheduler.rs   # Scheduler - cron-like task automation
│   │
│   ├── providers/         # Provider abstraction layer
│   │   ├── mod.rs         # Generic Provider trait & ProviderRegistry
│   │   ├── desktop.rs     # DesktopProvider trait & DefaultDesktopProvider
│   │   ├── notifications.rs # NotificationProvider trait
│   │   ├── file_picker.rs # FilePickerProvider trait
│   │   ├── tool.rs        # ToolProvider trait
│   │   └── plugin.rs      # PluginProvider trait & PluginWrapper
│   │
│   └── desktop/           # Desktop integration (legacy, being migrated to providers/)
│       ├── mod.rs
│       ├── detection.rs   # Desktop environment detection
│       ├── notifications.rs # System notifications (DBus + notify-send)
│       └── file_picker.rs # Native file pickers (Nemo, Zenity, KDialog)
```

## How the pieces fit together

- **UI Layer (`ui.rs`)** renders the TUI and forwards user actions to the **LLM core**.
- **LLM core (`llm.rs`)** decides which tool to run, maintains the conversation context, and orchestrates **sub-agents**.
- **Tool integration (`tools.rs`, `mcp.rs`, `lsp.rs`, `custom_tools.rs`)** provides a uniform interface for external services (MCP servers, LSP, user-defined scripts).
- **Intelligence modules (`rag.rs`, `skills.rs`, `planner.rs`)** add semantic search, skill-based behaviour, and multi-step planning.
- **Security & Auditing (`permissions.rs`, `audit.rs`, `config.rs`)** enforce file-access policies, network gating, and log every tool call.
- **Session Management (`sessions.rs`)** lets users save, restore, and fork interactive sessions.
- **Orchestrator (`orchestrator/`)** pools multiple agents, schedules tasks, and visualises the DAG in *Mission Control*.
- **Provider abstraction (`providers/`)** provides extensible trait-based integrations for desktop, notifications, file pickers, tools, and plugins.
- **Desktop integration (`desktop/`)** handles OS-specific notifications and native file-picker dialogs (legacy, being migrated to providers/).

All modules communicate via **event streams** (`events.rs`) and **shared context** (`context.rs`), preserving the strict separation-of-concerns enforced by the architecture.