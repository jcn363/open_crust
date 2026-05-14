# OpenCrust Module Reference

Quick reference for all modules in OpenCrust. Each entry shows what the module does, key types, when to modify it, and related modules.

For how to extend modules, see **docs/DEVELOPMENT.md**.  
For coding standards, see **AGENTS.md**.  
For architecture context, see **docs/ARCHITECTURE.md**.

---

## Core Application Modules

### main.rs
**Purpose:** Entry point, CLI argument parsing, event loop, TUI initialization  
**Key types:** `Config`, `Args`, `AppState`  
**Key functions:** `main()`, `run_tui()`, `handle_cli_args()`  
**When to modify:**
- Adding new CLI commands or flags
- Changing startup behavior
- Modifying event loop or shutdown

**Related modules:** `app.rs`, `config.rs`, `ui.rs`  
**Lines of code:** ~500

---

### app.rs
**Purpose:** Application state management, tabs, input history, background tasks  
**Key types:** `App`, `Tab`, `InputMode`, `ChatMessage`  
**Key functions:** `update()`, `handle_input()`, `render()`  
**When to modify:**
- Adding new tabs or views
- Changing input handling
- Modifying state transitions

**Related modules:** `ui.rs`, `events.rs`, `sessions.rs`  
**Lines of code:** ~800

---

### ui.rs
**Purpose:** Terminal UI rendering with Ratatui, layout, components, keybinds  
**Key types:** `TerminalUI`, `Layout`, `Widget`  
**Key functions:** `draw()`, `render_chat()`, `render_sidebar()`  
**When to modify:**
- Changing UI layout or appearance
- Adding new visual components
- Modifying keybind handling

**Related modules:** `app.rs`, `events.rs`, `markdown.rs`, `status_bar.rs`  
**Lines of code:** ~1200

---

### llm.rs
**Purpose:** LLM client, tool execution loop, context management, streaming responses  
**Key types:** `LLMClient`, `Message`, `ToolCall`, `LLMResponse`  
**Key functions:** `execute_agent_loop()`, `call_llm()`, `execute_tools()`  
**When to modify:**
- Adding new model provider
- Changing tool execution strategy
- Modifying context window management

**Related modules:** `tools.rs`, `context.rs`, `config.rs`, `permissions.rs`  
**Lines of code:** ~1000

---

## Tool Integration Modules

### tools.rs
**Purpose:** Tool schema definitions, routing, validation  
**Key types:** `Tool`, `ToolSchema`, `ToolCall`, `ToolResult`  
**Key functions:** `register_tool()`, `validate_call()`, `execute()`  
**When to modify:**
- Adding new built-in tool
- Changing tool schema
- Modifying tool execution

**Related modules:** `llm.rs`, `mcp.rs`, `lsp.rs`, `custom_tools.rs`  
**Lines of code:** ~600

---

### mcp.rs
**Purpose:** MCP server management, JSON-RPC transport, server discovery  
**Key types:** `MCPServer`, `MCPClient`, `JsonRpcRequest`  
**Key functions:** `start_server()`, `send_request()`, `handle_response()`  
**When to modify:**
- Adding MCP server support
- Changing JSON-RPC protocol handling
- Modifying server discovery

**Related modules:** `tools.rs`, `jsonrpc.rs`, `config.rs`  
**Lines of code:** ~800

---

### lsp.rs
**Purpose:** LSP client, completion, diagnostics, code formatting  
**Key types:** `LSPClient`, `CompletionRequest`, `DiagnosticResponse`  
**Key functions:** `initialize()`, `get_completions()`, `format_document()`  
**When to modify:**
- Adding language support
- Changing completion behavior
- Modifying diagnostic handling

**Related modules:** `tools.rs`, `config.rs`  
**Lines of code:** ~600

---

### custom_tools.rs
**Purpose:** Discovery and execution of user scripts in .opencrust/tools/  
**Key types:** `CustomTool`, `ToolScript`, `ExecutionResult`  
**Key functions:** `discover_tools()`, `load_script()`, `execute_script()`  
**When to modify:**
- Changing tool discovery paths
- Modifying script execution model
- Adding script language support

**Related modules:** `tools.rs`, `config.rs`, `permissions.rs`  
**Lines of code:** ~400

---

### tool_executor.rs
**Purpose:** Coordinated execution of tools with timeout, error recovery  
**Key types:** `ToolExecutor`, `ExecutionContext`, `ExecutionResult`  
**Key functions:** `execute()`, `with_timeout()`, `handle_error()`  
**When to modify:**
- Changing timeout behavior
- Adding execution retry logic
- Modifying error recovery

**Related modules:** `tools.rs`, `llm.rs`  
**Lines of code:** ~300

---

### acp.rs
**Purpose:** Agent Communication Protocol (ACP) stdio interface for multi-agent interop  
**Key types:** `ACPMessage`, `ACPServer`, `ACPClient`  
**Key functions:** `start_server()`, `parse_message()`, `send_response()`  
**When to modify:**
- Adding ACP protocol support
- Modifying inter-agent communication
- Adding new ACP message types

**Related modules:** `llm.rs`, `tools.rs`, `jsonrpc.rs`  
**Lines of code:** ~400

---

## Intelligence & Planning Modules

### rag.rs
**Purpose:** Semantic search using vector embeddings (Ollama embeddings)  
**Key types:** `SemanticIndex`, `EmbeddingVector`, `SearchResult`  
**Key functions:** `index_codebase()`, `search()`, `build_index()`  
**When to modify:**
- Changing embedding model
- Modifying search algorithm
- Adding result filtering

**Related modules:** `context.rs`, `web.rs`  
**Lines of code:** ~500

---

### skills.rs
**Purpose:** Skill discovery, loading, activation/deactivation  
**Key types:** `Skill`, `SkillRegistry`, `SkillMetadata`  
**Key functions:** `load_skills()`, `activate()`, `deactivate()`, `get_active_skills()`  
**When to modify:**
- Changing skill discovery paths
- Modifying skill activation logic
- Adding skill versioning

**Related modules:** `config.rs`, `rules.rs`  
**Lines of code:** ~400

---

### planner.rs
**Purpose:** Multi-step task planning, plan generation, execution tracking  
**Key types:** `Plan`, `Task`, `Step`, `ExecutionState`  
**Key functions:** `generate_plan()`, `execute_step()`, `track_progress()`  
**When to modify:**
- Changing plan generation strategy
- Modifying step execution
- Adding plan optimization

**Related modules:** `llm.rs`, `app.rs`  
**Lines of code:** ~500

---

## Security & Auditing Modules

### permissions.rs
**Purpose:** Granular permission enforcement (file access, command execution)  
**Key types:** `Permission`, `PermissionPolicy`, `AccessControl`  
**Key functions:** `check_file_access()`, `check_command()`, `load_policy()`  
**When to modify:**
- Adding new permission types
- Changing access control logic
- Modifying permission policies

**Related modules:** `audit.rs`, `config.rs`, `security.rs`  
**Lines of code:** ~600

---

### audit.rs
**Purpose:** Persistent audit logging (every tool call, file access, network request)  
**Key types:** `AuditLog`, `AuditEntry`, `AuditEvent`  
**Key functions:** `log_event()`, `query_logs()`, `export_logs()`  
**When to modify:**
- Adding new event types
- Changing log storage
- Modifying log queries

**Related modules:** `permissions.rs`, `config.rs`  
**Lines of code:** ~400

---

### security.rs
**Purpose:** Additional security boundaries, sandboxing, threat detection  
**Key types:** `SecurityPolicy`, `ThreatLevel`, `SecurityEvent`  
**Key functions:** `check_threat()`, `sandbox_execution()`, `report_incident()`  
**When to modify:**
- Adding security checks
- Changing sandbox behavior
- Adding threat detection rules

**Related modules:** `permissions.rs`, `audit.rs`  
**Lines of code:** ~350

---

### config.rs
**Purpose:** Configuration loading, validation, provider management  
**Key types:** `Config`, `ProviderConfig`, `MCPConfig`, `LSPConfig`  
**Key functions:** `load()`, `validate()`, `merge_defaults()`  
**When to modify:**
- Adding config option
- Adding provider support
- Changing validation rules

**Related modules:** `permissions.rs`, `security.rs`  
**Lines of code:** ~700

---

## Events Module

### events.rs
**Purpose:** Event bus and handler dispatch  
**Key types:** `Event`, `EventHandler`, `EventBus`  
**Key functions:** `dispatch()`, `subscribe()`, `emit()`  
**When to modify:**
- Adding new event types
- Changing event flow
- Modifying handlers

**Related modules:** `app.rs`, `ui.rs`  
**Lines of code:** ~400

---

## UX & Rendering Modules

### markdown.rs
**Purpose:** Markdown rendering and formatting  
**Key types:** `Markdown`, `Block`, `Inline`  
**Key functions:** `render()`, `parse()`, `to_text()`  
**When to modify:**
- Changing markdown rendering
- Adding markdown extensions
- Modifying formatting rules

**Related modules:** `ui.rs`  
**Lines of code:** ~300

---

### status_bar.rs
**Purpose:** Status bar component (model, tokens, cost, context budget)  
**Key types:** `StatusBar`, `StatusInfo`  
**Key functions:** `render()`, `update()`  
**When to modify:**
- Adding status indicators
- Changing status display
- Modifying updates

**Related modules:** `ui.rs`  
**Lines of code:** ~200

---

## Integration Modules

### git.rs
**Purpose:** Git operations (branch, commit, PR, blame, diff)  
**Key types:** `GitClient`, `CommitInfo`, `BranchInfo`  
**Key functions:** `commit()`, `create_branch()`, `create_pr()`  
**When to modify:**
- Adding git operations
- Changing commit handling
- Modifying PR creation

**Related modules:** `tools.rs`, `web.rs`  
**Lines of code:** ~500

---

### web.rs
**Purpose:** Web search integration, markdown conversion, HTTP requests  
**Key types:** `WebClient`, `SearchResult`, `WebPage`  
**Key functions:** `search()`, `fetch_and_convert()`, `parse_html()`  
**When to modify:**
- Adding search engines
- Changing response conversion
- Modifying HTTP handling

**Related modules:** `config.rs`, `permissions.rs`  
**Lines of code:** ~400

---

### formatters.rs
**Purpose:** Auto-formatter integration (rustfmt, prettier, etc.)  
**Key types:** `Formatter`, `FormatterConfig`  
**Key functions:** `format()`, `discover_formatters()`  
**When to modify:**
- Adding new formatter support
- Changing format detection
- Modifying formatter execution

**Related modules:** `tools.rs`, `config.rs`  
**Lines of code:** ~300

---

### context.rs
**Purpose:** Context management (@file syntax, pinning, budget tracking)  
**Key types:** `Context`, `ContextWindow`, `ContextBudget`  
**Key functions:** `load_file()`, `pin()`, `calculate_tokens()`  
**When to modify:**
- Changing context loading
- Modifying budget calculation
- Adding pinning features

**Related modules:** `llm.rs`, `config.rs`  
**Lines of code:** ~400

---

### rules.rs
**Purpose:** Steering rules / AGENTS.md loading for context injection  
**Key types:** `Rule`, `RuleSet`, `RuleContext`  
**Key functions:** `load_rules()`, `apply_rules()`, `inject_context()`  
**When to modify:**
- Changing rule parsing
- Adding rule types
- Modifying context injection

**Related modules:** `context.rs`, `llm.rs`  
**Lines of code:** ~300

---

## Utility Modules

### json_utils.rs
**Purpose:** JSON path utilities, parsing, querying  
**Key types:** `JsonPath`, `JsonQuery`, `QueryResult`  
**Key functions:** `query()`, `parse_path()`, `extract()`  
**When to modify:**
- Adding JSON utilities
- Changing path syntax
- Modifying queries

**Related modules:** `tools.rs`, `config.rs`  
**Lines of code:** ~200

---

### jsonrpc.rs
**Purpose:** JSON-RPC protocol primitives, request/response handling  
**Key types:** `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`  
**Key functions:** `serialize()`, `deserialize()`, `build_request()`  
**When to modify:**
- Changing JSON-RPC version
- Adding protocol methods
- Modifying error handling

**Related modules:** `mcp.rs`, `acp.rs`  
**Lines of code:** ~250

---

### clipboard.rs
**Purpose:** Clipboard integration (copy/paste)  
**Key types:** `Clipboard`  
**Key functions:** `copy()`, `paste()`  
**When to modify:**
- Changing clipboard provider
- Adding clipboard features
- Modifying platform support

**Related modules:** `ui.rs`, `app.rs`  
**Lines of code:** ~150

---

## Desktop Integration Modules

### desktop/mod.rs
**Purpose:** Desktop environment detection and integration  
**Key types:** `Desktop`, `DesktopEnv`, `DesktopFeatures`  
**Key functions:** `detect_environment()`, `get_features()`  
**When to modify:**
- Supporting new desktop environments
- Adding desktop detection
- Modifying feature discovery

**Related modules:** `desktop/detection.rs`, `desktop/notifications.rs`, `desktop/file_picker.rs`  
**Lines of code:** ~200

---

### desktop/detection.rs
**Purpose:** Desktop environment detection (Cinnamon, MATE, GNOME, Plasma)  
**Key types:** `DesktopDetector`, `DesktopType`, `ThemeInfo`  
**Key functions:** `detect()`, `get_theme()`, `is_available()`  
**When to modify:**
- Adding desktop support
- Changing detection logic
- Modifying theme extraction

**Related modules:** `desktop/mod.rs`  
**Lines of code:** ~300

---

### desktop/notifications.rs
**Purpose:** System notifications (DBus + notify-send fallback)  
**Key types:** `NotificationClient`, `Notification`, `NotificationLevel`  
**Key functions:** `send()`, `init()`, `close()`  
**When to modify:**
- Adding notification backends
- Changing notification format
- Modifying platform support

**Related modules:** `desktop/mod.rs`  
**Lines of code:** ~300

---

### desktop/file_picker.rs
**Purpose:** Native file pickers (Nemo, Zenity, KDialog, etc.)  
**Key types:** `FilePicker`, `FilePickerBackend`, `FilePickerResult`  
**Key functions:** `pick_file()`, `pick_directory()`, `detect_backend()`  
**When to modify:**
- Adding picker backends
- Changing picker behavior
- Modifying platform support

**Related modules:** `desktop/mod.rs`  
**Lines of code:** ~400

---

## MCP Showcase & Mission Control Modules

### mcp_showcase/mod.rs
**Purpose:** MCP server browser and management UI  
**Key types:** `MCPShowcase`, `ServerBrowser`, `ServerList`  
**Key functions:** `show()`, `toggle_server()`, `refresh()`  
**When to modify:**
- Changing showcase behavior
- Adding server management features
- Modifying UI

**Related modules:** `mcp_showcase/tui.rs`, `mcp.rs`, `ui.rs`  
**Lines of code:** ~300

---

### mcp_showcase/tui.rs
**Purpose:** MCP Showcase terminal UI (Ratatui component)  
**Key types:** `ShowcaseTUI`, `ServerWidget`, `ListState`  
**Key functions:** `render()`, `handle_input()`, `update()`  
**When to modify:**
- Changing showcase appearance
- Adding interactive features
- Modifying keybinds

**Related modules:** `mcp_showcase/mod.rs`, `ui.rs`  
**Lines of code:** ~400

---

### mission_control/mod.rs
**Purpose:** Mission control interface for multi-agent coordination  
**Key types:** `MissionControl`, `AgentControl`, `TaskMonitor`  
**Key functions:** `monitor()`, `coordinate()`, `report_status()`  
**When to modify:**
- Changing agent coordination
- Adding monitoring features
- Modifying control logic

**Related modules:** `mission_control/tui.rs`, `llm.rs`, `events.rs`  
**Lines of code:** ~300

---

### mission_control/tui.rs
**Purpose:** Mission control terminal UI (Ratatui component)  
**Key types:** `MissionControlTUI`, `AgentWidget`, `StatusDisplay`  
**Key functions:** `render()`, `handle_input()`, `update()`  
**When to modify:**
- Changing mission control appearance
- Adding interactive features
- Modifying keybinds

**Related modules:** `mission_control/mod.rs`, `ui.rs`  
**Lines of code:** ~400

---

## Sessions Module

### sessions.rs
**Purpose:** Session persistence, save/load/list/fork operations  
**Key types:** `Session`, `SessionMetadata`, `SessionStore`  
**Key functions:** `save()`, `load()`, `list()`, `fork()`  
**When to modify:**
- Changing session storage format
- Adding session features
- Modifying persistence

**Related modules:** `app.rs`, `config.rs`  
**Lines of code:** ~500

---

## Quick Navigation by Use Case

**"I want to add a new CLI command"**
→ `main.rs` (argument parsing) → `app.rs` (state handling) → `events.rs` (dispatch)

**"I want to integrate a new tool"**
→ `tools.rs` (schema) → `tool_executor.rs` (execution) → `mcp.rs` or `custom_tools.rs` (discovery)

**"I want to support a new language"**
→ `lsp.rs` (language server protocol) → `config.rs` (configuration)

**"I want to modify permissions or auditing"**
→ `permissions.rs` (access control) → `audit.rs` (logging)

**"I want to add a UI feature"**
→ `ui.rs` (rendering) → `app.rs` (state) → `events.rs` (input handling)

**"I want to improve search"**
→ `rag.rs` (semantic search) → `context.rs` (context management)

**"I want to support a new desktop"**
→ `desktop/detection.rs` (detection) → `desktop/notifications.rs` or `desktop/file_picker.rs` (features)

---

**For more context, see:**
- **docs/DEVELOPMENT.md** — How to extend each subsystem
- **docs/ARCHITECTURE.md** — How modules interact
- **AGENTS.md** — Coding standards and patterns
