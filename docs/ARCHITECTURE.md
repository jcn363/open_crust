# OpenCrust Architecture Deep Dive

This document explains how OpenCrust works internally: data flows, core systems, concurrency model, module interactions, and design decisions.

**For 10,000-foot overview:** See **README.md**.  
**For module reference:** See **docs/MODULES.md**.  
**For extending:** See **docs/DEVELOPMENT.md**.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         OpenCrust TUI Application                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Terminal User Interface (Ratatui)                            │  │
│  │ • Chat Tab: Message history + input                          │  │
│  │ • Tasks Tab: Active tasks, progress tracking                 │  │
│  │ • Plan Tab: Proposed changes, approval/edit                  │  │
│  │ • File Tree: Workspace file browser                          │  │
│  │ • Status Bar: Token usage, session info                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                           ▲                                          │
│                    Events (KeyCode, Mouse)                          │
│                           │                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Event Loop (main.rs)                                         │  │
│  │ • Poll user input (non-blocking)                             │  │
│  │ • Route to current Tab handler                               │  │
│  │ • Dispatch tool completions, LLM responses                   │  │
│  │ • Render UI                                                  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                           ▲                                          │
│                    State mutations                                   │
│                           │                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Application State (app.rs)                                   │  │
│  │ • Current tab + internal state per tab                       │  │
│  │ • Chat history, active tasks, file tree                      │  │
│  │ • Session metadata (token usage, costs)                      │  │
│  │ • Undo/redo stack                                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                           ▲                                          │
│                    Requests / Responses                             │
│                           │                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Core Services Layer                                          │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ • LLM Client (llm.rs) — Manages agentic loop                 │  │
│  │ • Tool Executor (tools.rs, tool_executor.rs)                 │  │
│  │ • RAG System (rag.rs) — Semantic search                      │  │
│  │ • Skills Manager (skills.rs) — Skill loading                 │  │
│  │ • Planner (planner.rs) — Plan generation                     │  │
│  └────────────────────────────────────────────────────────────┤  │
│                           ▲                                          │
│           Tool calls, Context building, Model invocations           │
│                           │                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ External Integration Layer                                   │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ • MCP Servers (mcp.rs) — JSON-RPC over stdio/network        │  │
│  │ • LSP Servers (lsp.rs) — Language features                  │  │
│  │ • Custom Tools (.opencrust/tools/) — Scripts                │  │
│  │ • LLM Providers — 11 supported models                        │  │
│  └────────────────────────────────────────────────────────────┤  │
│                           ▲                                          │
│              HTTP/SSH/stdio/JSON-RPC/Process calls                  │
│                           │                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ System Resources & Security                                  │  │
│  ├────────────────────────────────────────────────────────────┤  │
│  │ • Filesystem (with permission policies)                      │  │
│  │ • Network (with gating rules)                                │  │
│  │ • Process execution (with audit trail)                       │  │
│  │ • Configuration (config.json)                                │  │
│  │ • Session state (persistent storage)                         │  │
│  └────────────────────────────────────────────────────────────┤  │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Core Data Flow: From User Input to Response

### Phase 1: User Input → Event
```
User types message in Chat tab
  ↓
Terminal emits KeyCode event (via crossterm)
  ↓
Event loop captures it (events.rs)
  ↓
Route to Tab handler (ui.rs)
```

### Phase 2: User Request → LLM Input
```
User presses Enter (message ready)
  ↓
app.rs: Store message in chat history
  ↓
llm.rs: Build context (conversation, skills, config)
  ↓
Include system prompt + user message + tool definitions
  ↓
Call LLM provider (openai, ollama, etc. via llm.rs)
```

### Phase 3: LLM Response → Tool Execution
```
LLM responds with:
  - Text content (displayed in Chat tab)
  - Tool calls (function_calls, e.g., file_read, tool_execute)
  
Tool executor handles each call:
  ↓
For tool_read: Check permissions → fs::read_file → return content
  ↓
For tool_execute: Check permissions + audit → subprocess::run → capture output
  ↓
For tool_mcp_call: Serialize JSON-RPC → send to MCP server → parse response
  ↓
For tool_code_plan: Parse markdown → store in app state → render in Plan tab
```

### Phase 4: Tool Results → LLM Again
```
Collect all tool results
  ↓
Format as "tool result: [output]" in chat context
  ↓
Send back to LLM: "Here are tool results, continue"
  ↓
LLM processes and either:
  - Returns more text (agent's response)
  - Calls more tools (agentic loop)
  - Ends conversation (no more tool calls)
```

### Phase 5: Response → UI Rendering
```
Render response in Chat tab as it streams in
  ↓
Parse tool output (diffs, code blocks, tables)
  ↓
Update status bar: token count, cost estimate
  ↓
Store message in session state
  ↓
Next event loop: Render updated UI
```

---

## Core Systems

### 1. The Agentic Loop (llm.rs)

**How it works:**

```rust
loop {
  // 1. Build context with conversation + tools
  let context = build_context(&config, &skills, &history);
  
  // 2. Call LLM
  let response = call_llm(&config.provider, &context)?;
  
  // 3. Extract tool calls from response
  let tool_calls = parse_tool_calls(&response);
  
  if tool_calls.is_empty() {
    // No more tools → conversation done
    break;
  }
  
  // 4. Execute each tool
  for call in tool_calls {
    let result = execute_tool(&call)?;
    history.push(ToolResult { tool_name: call.name, result });
  }
  
  // 5. Loop: Send results back to LLM with updated context
}
```

**Key insight:** The LLM never directly accesses files or runs tools. OpenCrust is the executor. This enables:
- Permission enforcement (before tool execution)
- Audit logging (what was run, by whom)
- Sandboxing (script execution in restricted env)
- Cost control (track token usage)

### 2. Tool System (tools.rs, tool_executor.rs)

**Tool types:**

| Type | Implementation | Examples |
|------|-----------------|----------|
| **Built-in** | Rust code in tool_executor.rs | file_read, file_write, file_delete, dir_list, code_search |
| **Custom** | Executable scripts in .opencrust/tools/ | my_linter, deploy_script, data_transform |
| **MCP** | JSON-RPC servers defined in config | github, postgres, slack, confluence |
| **LSP** | Language-specific servers | rust-analyzer, typescript-language-server, pyright |

**Tool execution path:**

```
1. User/LLM requests tool_execute("my_script", args)
  ↓
2. Permissions system checks:
   - Is tool in allowed file patterns?
   - Does current session have execute permission?
   - Is tool in audit allowlist?
  ↓
3. If denied: Return permission error
  ↓
4. If allowed: Execute
   - Spawn subprocess with limited env
   - Capture stdout/stderr
   - Record in audit log (what ran, when, by whom, exit code)
   - Return output or error
```

### 3. Permissions & Audit System

**Permissions model:**

```json
{
  "permissions": {
    "file_patterns": ["src/**", "docs/**", ".opencrust/**"],
    "deny_patterns": [".env", "*.key"],
    "network": {
      "enabled": true,
      "hosts": ["api.openai.com", "github.com"]
    }
  }
}
```

**Audit trail (audit.json):**

```json
{
  "timestamp": "2026-05-08T14:30:00Z",
  "session_id": "sess_abc123",
  "event_type": "tool_executed",
  "tool_name": "file_read",
  "path": "src/main.rs",
  "user": "alice",
  "result": "success",
  "output_lines": 50
}
```

**Security boundaries:**

- **File I/O:** Can only read/write files matching patterns in config
- **Execution:** Custom tools run as separate processes (no direct memory access)
- **Network:** Only configured hosts allowed
- **Secrets:** Never logged or displayed in UI

### 4. LLM Provider Abstraction

**How providers work:**

All 11 providers (Ollama, OpenRouter, OpenAI, Gemini, etc.) use same interface:

```rust
pub trait LlmProvider {
  async fn call(&self, request: LlmRequest) -> Result<LlmResponse>;
}

pub struct LlmRequest {
  pub model: String,
  pub messages: Vec<Message>,
  pub tools: Vec<Tool>,
  pub temperature: f32,
}

pub struct LlmResponse {
  pub content: String,
  pub tool_calls: Vec<ToolCall>,
  pub tokens_used: TokenUsage,
}
```

**Provider routing (in llm.rs):**

```rust
match &config.provider {
  Provider::Ollama(cfg) => OllamaClient::call(cfg, request).await,
  Provider::OpenAI(cfg) => OpenAiClient::call(cfg, request).await,
  Provider::Gemini(cfg) => GeminiClient::call(cfg, request).await,
  // ... 8 more providers
}
```

**Why this matters:**
- Easy to add new providers (implement trait)
- Configuration-driven (no code changes to switch)
- Token tracking per provider (cost estimation)
- Fallback chains possible (try provider A, fall back to B)

### 5. RAG & Semantic Search (rag.rs)

**Architecture:**

```
Codebase files
  ↓
Split into chunks (code blocks, paragraphs)
  ↓
Generate embeddings (via Ollama embedding model or API)
  ↓
Store in vector DB (in-memory + disk cache)
  ↓
User query: "Find function that handles auth"
  ↓
Embed query (same model)
  ↓
Similarity search: Find top-K nearest chunks
  ↓
Return relevant code snippets to LLM context
```

**Used by:**
- `semantic_search` tool (find related code)
- Context builder (automatically include relevant files)
- Planner (understand codebase structure)

---

## Module Interaction Map

```
┌─────────────────────────────────────────────────────────────────┐
│ main.rs                                                           │
│ • Entry point                                                     │
│ • CLI argument parsing                                            │
│ • Route to subcommand handlers                                    │
└─────────────────────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────────────────────┐
│ app.rs                                                            │
│ • Central state machine (tabs, history, sessions)                │
│ • Undo/redo stack                                                 │
│ • Delegates to tab-specific handlers                             │
└─────────────────────────────────────────────────────────────────┘
    ↙         ↓         ↘
   UI        LLM       Sessions
   ↓         ↓           ↓
┌──────┐ ┌─────────┐ ┌───────────┐
│ui.rs │ │llm.rs   │ │sessions.rs│
└──────┘ └─────────┘ └───────────┘
   ↓         ↓           ↓
Renders    Calls        Persists
 events    tools        history
   ↑         ↓           ↑
   └─────────┼───────────┘
             ↓
      ┌────────────────┐
      │ tools.rs       │
      │ tool_executor  │
      └────────────────┘
             ↓
    ┌────────┼────────┐
    ↓        ↓        ↓
  MCP      LSP    Custom
 (mcp.rs) (lsp.rs) tools
```

**Key observations:**
- **ui.rs** never calls llm.rs directly (goes through app.rs)
- **tools.rs** never accesses app.rs state (stateless)
- **llm.rs** uses tools.rs but tools are "functions" to LLM (LLM decides what to call)
- **permissions.rs** gates everything (before any I/O)

---

## Concurrency Model

### Multi-tasking (Async/Await with Tokio)

**Long-running operations spawn tasks:**

```rust
// Example: LLM call + tool execution happening in background
tokio::spawn(async {
  match call_llm_and_execute_tools(&state).await {
    Ok(response) => {
      // Send response to UI channel
      tx.send(UiMessage::ResponseReady(response)).await;
    }
    Err(e) => {
      tx.send(UiMessage::Error(e.to_string())).await;
    }
  }
});

// UI still responsive while task runs
// New events processed while LLM thinks
```

**Channels for communication:**

```
UI Thread              Background Tasks
     ↓                      ↑
  [channel tx]  ←→   [channel rx]
     ↑                      ↓
Poll events        Send results/errors
```

**Why async?**
- UI never blocks waiting for LLM
- Multiple concurrent operations (multiple MCP servers, streaming)
- Natural fit for network I/O (LLM API calls, SSH, HTTP)

### Thread Safety

All shared state uses Arc + Mutex or Tokio channels:

```rust
// Shared session state
let session = Arc::new(Mutex::new(SessionState::new()));

// Background task reads session
let session_clone = Arc::clone(&session);
tokio::spawn(async move {
  let state = session_clone.lock().await;
  println!("Current model: {}", state.current_model);
});
```

---

## Event Loop Timing

**Per frame (typically 16-33ms):**

```
1. Poll crossterm for keyboard/mouse input (non-blocking)
2. Process input event in current tab handler (instant)
3. Dispatch any ready async results from channels
4. Update app state based on new messages
5. Render UI using current app state
6. Sleep remainder of frame (or return if next event waiting)
```

**Typical latency:**
- Key press → appears on screen: **< 50ms** (usually 16-33ms per frame)
- LLM call (background): Doesn't block UI, shows "thinking..." indicator
- Tool execution: Shown in Tasks tab with real-time updates

---

## State Mutations & Undo/Redo

**Immutable-first design:**

```rust
// Every mutation creates new state
pub struct App {
  pub chat_history: Vec<Message>,  // Append-only
  pub undo_stack: Vec<AppState>,   // Save before mutation
  pub redo_stack: Vec<AppState>,   // Save redo states
}

// To mutate: Save current, change, push to undo stack
pub fn handle_delete_message(&mut self, id: usize) {
  self.undo_stack.push(self.clone());  // Save current state
  self.chat_history.retain(|m| m.id != id);  // Mutate
  self.redo_stack.clear();  // Clear redo (new branch)
}
```

**Undo/Redo implementation:**
- Ctrl+Z: Pop from undo_stack, push current to redo_stack
- Ctrl+Shift+Z: Pop from redo_stack, push current to undo_stack
- Limited to last 50 states (prevent memory bloat)

---

## Configuration Loading

**Startup sequence:**

```
1. Load default config (hardcoded in code)
2. Read config.json if exists (~/.config/open_crust/config.json)
3. Merge: JSON overrides defaults
4. Validate: Check required fields, types
5. Load MCP servers from config
6. Load LSP servers from config
7. Load custom tools from .opencrust/tools/
8. Load skills from .opencrust/skills/
9. Ready: Start event loop
```

**Config hierarchy:**
```
Default built-in config
  ↓ (overridden by)
~/.config/open_crust/config.json
  ↓ (overridden by)
Environment variables (e.g., OPENCRUST_MODEL)
  ↓ (overridden by)
CLI flags (e.g., --model gpt-4)
```

---

## Session Persistence

**What's saved:**

```json
{
  "id": "sess_abc123",
  "created_at": "2026-05-08T10:00:00Z",
  "name": "My Project",
  "messages": [
    { "role": "user", "content": "..." },
    { "role": "assistant", "content": "..." }
  ],
  "metadata": {
    "tokens_used": 45230,
    "estimated_cost": 1.23,
    "current_model": "gpt-4"
  }
}
```

**Location:** `~/.config/open_crust/sessions/<session_id>.json`

**Loading on startup:**
- List recent sessions in sidebar
- Click to restore: Load messages into chat history
- All context preserved (model, token count, etc.)

---

## Error Handling Strategy

**Three-tier error handling:**

| Tier | Scope | Behavior |
|------|-------|----------|
| **Critical** | App panic (memory corruption, unwrap in prod) | Crash, save session to recovery file, display error |
| **User-facing** | Tool failure, LLM timeout, permission denied | Show error in Chat, offer retry or fallback |
| **Silent** | Logging-level events (cache miss, retry #3) | Log only, don't interrupt user |

**Error propagation:**

```rust
// Tool fails: Return error message to LLM context
pub fn execute_tool(...) -> Result<ToolResult> {
  match std::fs::read(&path) {
    Ok(content) => Ok(ToolResult::Success(content)),
    Err(e) => Ok(ToolResult::Error(format!("Permission denied: {}", e))),
  }
}

// LLM receives: "Tool failed: Permission denied: ..."
// LLM can then suggest alternatives or ask for clarification
```

---

## Performance Characteristics

### Typical Latencies

| Operation | Latency | Bottleneck |
|-----------|---------|-----------|
| Key press → screen | 16–50ms | Terminal rendering |
| Chat message enter | 100ms | Parse + format |
| LLM call start | 500ms–5s | Provider API (network) |
| Streaming LLM response | Real-time (tokens/sec) | Network bandwidth |
| Tool execution | Varies | Script runtime |
| Semantic search | 50–200ms | Embedding + vector search |
| Session save | 10–50ms | Disk I/O |

### Memory Usage

Typical running: **200–500MB**
- Chat history: ~100MB (1000 messages)
- RAG vector index: ~100MB (if enabled)
- Async runtime: ~50MB
- UI buffers: ~10MB

Can grow to **1GB+** if:
- Very long sessions (10,000+ messages)
- Large codebase indexed (100K+ files)
- Many open files (file tree rendering)

### Startup Time

Typical startup: **1–2 seconds**
- Load config + verify: **100ms**
- Discover custom tools: **10–50ms**
- Load skills: **50–100ms**
- Initialize LLM client: **100–200ms**
- Setup RAG (optional): **500–1000ms**

---

## Key Design Decisions

### 1. **Stateless Tools, Stateful App**

Tools don't hold state; they're pure functions.  
State lives in app.rs.

**Why:** Easy to parallelize, test, and reason about.

### 2. **Permission Checks Before Execution**

Every file I/O and process execution checked against permission policy first.

**Why:** Prevents accidents, enables audit trail, supports sandboxing.

### 3. **Provider Abstraction**

11 different LLM providers, same interface.

**Why:** Users can switch providers without code changes; easy to add new ones.

### 4. **Configuration-Driven**

MCP servers, LSP servers, custom tools, skills — all discovered and configured via config.json + filesystem.

**Why:** No code changes needed to extend; users can customize without rebuilding.

### 5. **Async/Await for Concurrency**

Long operations (LLM calls, tool execution, file I/O) don't block UI.

**Why:** Responsive TUI; users see progress, can interrupt.

### 6. **Immutable-First State**

App state changes are push-to-stack, enabling cheap undo/redo.

**Why:** Users can experiment without fear; rollback is free.

---

## Security Boundaries

```
┌─────────────────────────────────────┐
│ Trusted Code (Rust, open_crust)     │
├─────────────────────────────────────┤
│ • Makes all decisions                │
│ • Enforces permissions               │
│ • Logs all actions                   │
└─────────────────────────────────────┘
          ↕ (JSON/stdio)
┌─────────────────────────────────────┐
│ Untrusted I/O (LLM, MCP, scripts)   │
├─────────────────────────────────────┤
│ • Requests vetted before execution   │
│ • Output parsed, not executed        │
│ • Sandboxed processes                │
└─────────────────────────────────────┘
          ↕ (stdio/HTTP/SSH)
┌─────────────────────────────────────┐
│ External Systems (APIs, servers)    │
├─────────────────────────────────────┤
│ • Network gating (hosts allowlist)   │
│ • API keys managed in config         │
│ • Responses validated                │
└─────────────────────────────────────┘
```

---

## Next Steps

- **Configure your first MCP server:** See **docs/CONFIGURATION.md**
- **Add a custom tool:** See **docs/DEVELOPMENT.md**
- **Understand the permission model:** See **docs/SECURITY.md**
- **Optimize performance:** See **docs/PERFORMANCE.md**
- **Deep-dive a module:** See **docs/MODULES.md**, pick a module, read the source code

---

## References

- **Module reference:** `docs/MODULES.md` (where each system lives)
- **Development guide:** `docs/DEVELOPMENT.md` (how to extend)
- **Security model:** `docs/SECURITY.md` (permissions, audit, sandboxing)
- **Performance tuning:** `docs/PERFORMANCE.md` (profiling, optimization)
- **Coding standards:** `AGENTS.md` (style, error handling, testing)
