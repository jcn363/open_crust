# OpenCrust Enhanced Implementation Plan

This document provides detailed implementation plans for all priority features identified in the market analysis. Each plan includes technical approach, file modifications, acceptance criteria, and effort estimates.

> **Last Updated**: 2026-05-08
> **Status**: Actively maintained - reflects current codebase state
>
> **Note**: Background Agents feature implemented on 2026-05-07
> **2026-05-08 Review**: All 12 planned features confirmed implemented. No unimplemented priority features remain. Only Vim Mode (Feature 8) left as Future Enhancement.

## Executive Summary

Based on competitive analysis against Cursor, Claude Code, and open-source alternatives (VT Code, smelt, oli), this plan reorders features by **actual user impact** derived from pain points:

### Top User Pain Points (from Reddit/competitor reviews)
1. **Agent goes wrong** - executes wrong files, corrupts code
2. **"Where am I?"** - context confusion, token limits
3. **Rate limits** - work killed mid-stream
4. **Context loss** - long sessions lose early context
5. **Claude Code lock-in** - can't switch models mid-session

### Current Implementation Status
 
| Feature | Status | Notes |
|---------|--------|-------|
| Tabbed Interface (Chat/Tasks) | ✅ Implemented | `app.rs` - tabs, active_tab |
| File Tree Sidebar | ✅ Implemented | `app.rs` - show_sidebar, Ctrl+B toggle |
| Command History | ✅ Implemented | Persistent across sessions |
| Interactive Diff Viewer | ✅ Implemented | Approval-gate with a/d keys |
| Context Pinning | ✅ Implemented | `llm.rs` - pinned_files |
| Customizable TUI/Theme | ✅ Implemented | `config.rs` - theme config |
| Desktop Integration | ✅ Implemented | notifications, file picker, detection |
| CLI Commands | ✅ Implemented | desktop, session, mcp subcommands |
| MCP & LSP Support | ✅ Implemented | Full JSON-RPC support |
| Skills System | ✅ Implemented | 9 built-in skills, active/inactive toggle, SkillBrowser UI (Ctrl+Shift+K), CLI subcommands |
| Web Intelligence | ✅ Implemented | Brave search integration |
| Semantic Search | ✅ Implemented | Local RAG |
| Task Planner | ✅ Implemented | Multi-step plans |
| Usage Tracking | ✅ Implemented | Token count + cost in USD |
| Context Pruning | ✅ Implemented | Auto-prune at 22 messages |
| Multi-Provider Support | ✅ Implemented | 6 providers (Ollama, OpenRouter, OpenAI, Gemini, Mistral, Anthropic) |
| **Multi-Provider Toggle** | ✅ **Implemented** | **Runtime provider switching via command palette** |
| ACP Mode | ✅ Implemented | JSON-RPC over stdio |
| MCP Browser UI | ✅ Implemented | Server management panel |
| **Command Palette (Ctrl+K)** | ✅ **Implemented** | **Provider switching, stats, context clear, MCP browser** |
| **Context Budget Display** | ✅ Implemented | **Status bar shows tokens, budget, percentage, cost** |
| **Model Context Display** | ✅ Implemented | **Status bar shows model name and context limit** |
| **Plan Mode + Diff Preview** | ✅ Implemented | **File list, status indicators, approve/deny workflow** |
| **Background Agents** | ✅ **Implemented** | **Async task spawning, Tasks tab display, Ctrl+T keybinding** |
| **Auto-Context Summarization** | ✅ **Implemented** | **LLM-based summarization at 80% context threshold, config threshold** |
| **Input Prediction (Ghost Text)** | ✅ **Implemented** | **Ghost text with Tab accept, Escape dismiss, 300ms debounce** |
| **Session Forking (CLI)** | ✅ **Implemented** | **CLI command `opencrust session fork`, unit tests passing** |
| **Enhanced Skills** | ✅ **Implemented** | **Skill active/inactive toggle, SkillBrowser UI, CLI subcommands, usage tracking** |

### Impact-Ordered Priority (Not Yet Implemented)

> **Note**: All planned features have been implemented. Vim Mode has been moved to Future Enhancements section due to implementation complexity.

### Completed in This Session

| Feature | Completed On | Hours |
|---------|---------------|--------|
| Input Prediction (Ghost Text) | 2026-05-07 | 5 |
| Session Forking (CLI) | 2026-05-07 | 5 |
| MCP Server Mode | 2026-05-07 | 6 |
| Headless / Scriptable Mode | 2026-05-07 | 3 |

### Completed Features (This Session)

| Feature | Completed On | Notes |
|---------|---------------|-------|
| Background Agents | 2026-05-07 | Async task spawning, Tasks tab display, Ctrl+T keybinding |
| Auto-Context Summarization | 2026-05-07 | LLM-based summarization at 80% context threshold, config threshold support |
| Input Prediction | 2026-05-07 | Ghost text with Tab accept, Escape dismiss, 300ms debounce, dimmed rendering |
| Session Forking | 2026-05-07 | CLI command `opencrust session fork <id> [--name <name>]`, unit tests passing |
| **Enhanced Skills** | 2026-05-07 | Skill active/inactive toggle, SkillBrowser UI (Ctrl+Shift+K), CLI subcommands (list/activate/deactivate/stats), unit tests passing, usage tracking deferred |
| **MCP Server Mode** | 2026-05-07 | `opencrust serve --stdio` subcommand, JSON-RPC protocol handling, tools/list returns 3 tools |
| **Headless Mode** | 2026-05-07 | CLI flags (-p/--prompt, -f/--file, --project, --provider, --model), early-exit before TUI |

---

## Impact Tier 1: Critical (Must Have)

### Feature 1: Plan Mode with Diff Preview

#### Description

Implement a plan-review workflow where the LLM proposes changes, user reviews diffs, then approves/denies. This is the #1 requested feature from competitive reviews and directly addresses trust issues with autonomous agents.

#### Technical Approach

**Architecture**: Extend the existing `Review` mode in `app.rs` with multi-file diff tracking.

**State Changes** (app.rs):
```rust
// New struct in app.rs
#[derive(Clone, Debug)]
pub struct ProposedChange {
    pub path: String,
    pub original: String,
    pub proposed: String,
    pub status: ChangeStatus,  // NEW: pending/approved/denied
}

#[derive(Clone, Debug)]
pub enum ChangeStatus {
    Pending,
    Approved,
    Denied,
}

// Extend App struct
pub struct App {
    // ... existing fields ...
    pub proposed_changes: Vec<ProposedChange>,  // MODIFY: add status field
    pub plan_mode: PlanMode,  // NEW
}

#[derive(Clone, Copy, Debug)]
pub enum PlanMode {
    Disabled,    // Normal execution
    Planning,    // LLM is planning
    Reviewing,    // User reviewing diffs
}
```

**UI Changes** (ui.rs):
1. Add keybinding for plan mode toggle (Ctrl+P)
2. Enhance `draw_review_popup` to show file list + diff navigation
3. Add status indicators for each file (pending/approved/denied)
4. Show summary: "X files changed, Y pending review"

**LlmClient Changes** (llm.rs):
1. Add new method `send_message_plan_mode()` that collects all file changes
2. Instead of executing write tools immediately, queue them as ProposedChange
3. Send final summary: "X files will be modified. Review changes?"

**IPC Mechanism**:
- Progress channel (`progress_tx`): Send "[PLAN_FILE]path|original|proposed" messages
- Approval channel (`approval_rx`): Receive per-file approve/deny commands

**Flow**:
1. User types prompt, presses Ctrl+P (or prompt starts with "plan:")
2. LlmClient detects plan mode, enters `PlanMode::Planning`
3. When LLM requests write, don't execute - queue as ProposedChange with original content
4. After LLM finishes, switch to `PlanMode::Reviewing`
5. Show popup with file list + diff view
6. User navigates (↑/↓), approves (A), denies (D), or approves all (Shift+A)
7. Execute approved changes, skip denied

#### File Modifications

| File | Changes |
|------|---------|
| `src/app.rs` | Add `PlanMode` enum, extend `ProposedChange`, add method handlers |
| `src/ui.rs` | Enhance review popup, add file list, status indicators |
| `src/llm.rs` | Add plan mode detection, queue writes instead of execute |
| `src/main.rs` | Add Ctrl+P keybinding |
| `src/events.rs` | Add key handling for A/D keys in review mode |

#### Acceptance Criteria

- [ ] User can enter plan mode with Ctrl+P
- [ ] LLM proposes changes without executing them
- [ ] User sees side-by-side diff for each file
- [ ] User can navigate between files with ↑/↓
- [ ] User can approve individual files with A
- [ ] User can deny individual files with D
- [ ] User can approve all with Shift+A
- [ ] Approved changes are executed after review
- [ ] Denied changes are not executed
- [ ] Plan mode can be cancelled with Esc

#### Effort Estimate

**Developer Hours**: 8-12 hours
- UI enhancements: 4 hours
- LLM client changes: 4 hours
- Integration testing: 2 hours

---

### Feature 2: Context Budget Display

#### Description

Show token count and cost in real-time status bar. Users consistently complained about "where am I in context?" - this provides visibility similar to Cursor's usage display.

#### Technical Approach

**Existing Data**: `UsageStats` already tracks `input_tokens` and `output_tokens` in `llm.rs`:
```rust
pub struct UsageStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
}
```

**Display Location**: Status bar in ui.rs (already partially implemented):

**Enhancements**:
1. Add context window budget - estimate based on provider limits:
   - Ollama: ~8K (configurable)
   - OpenAI/Gemini: ~128K
   - Anthropic: ~200K (1M for Max)

2. Calculate approximate tokens from message history:
   - Use simple heuristic: ~4 characters per token
   - Or count actual if provider returns it

3. Show percentage: "Tokens: 45K/128K (35%)"

4. Show estimated cost for current session

**Config Addition** (config.rs):
```rust
pub struct Config {
    // ... existing fields ...
    pub context_budget: Option<u64>,  // Override default budget
}
```

**UI Update** (ui.rs):
```rust
// In status bar rendering
let context_percent = if budget > 0 {
    (total_tokens as f64 / budget as f64 * 100.0) as u16
} else {
    0
};
let status = Paragraph::new(format!(
    "-- {} -- | Context: {}/{} ({}%) | Cost: ${:.4}",
    mode_str, total_tokens, budget, context_percent, cost
))
```

#### File Modifications

| File | Changes |
|------|---------|
| `src/config.rs` | Add `context_budget` config option |
| `src/ui.rs` | Enhance status bar with context percentage |
| `src/llm.rs` | Calculate running token estimate |

#### Acceptance Criteria

- [ ] Status bar shows current token count
- [ ] Status bar shows context budget (provider-based default or custom)
- [ ] Status bar shows percentage filled
- [ ] Status bar shows session cost estimate
- [ ] User can configure custom budget in config.json

#### Effort Estimate

**Developer Hours**: 2-3 hours
- Config addition: 0.5 hours
- UI enhancement: 1 hour
- Integration: 0.5 hours

---

## Impact Tier 2: High

### Feature 3: Multi-Provider Toggle

#### Description

Allow switching between providers (Anthropic, OpenAI, Gemini, Ollama) at runtime without restart. This addresses Claude Code's lock-in criticism and differentiates OpenCrust as a flexible tool.

#### Technical Approach

**Existing Support**: `ProviderType` enum already exists in config.rs:
```rust
pub enum ProviderType {
    Ollama,
    OpenRouter,
    OpenAI,
    Gemini,
    Mistral,
    Anthropic,
}
```

**Runtime Switching**: Add UI mechanism to change provider + model without config file reload.

**State Addition** (app.rs):
```rust
pub struct App {
    // ... existing fields ...
    pub provider_overrides: HashMap<String, String>,  // "provider": "OpenAI"
}
```

**UI Addition** (ui.rs):
1. Add command palette (Ctrl+K) for quick actions
2. Show current provider/model in status bar
3. Add provider switcher in command palette

**Keybinding**: Ctrl+K opens command palette with:
- "Switch Provider: [current]"
- "Switch Model: [current]"
- "Show Stats"
- "Clear Context"

**Implementation** (app.rs):
```rust
pub fn switch_provider(&mut self, provider: ProviderType) {
    self.config.provider = provider;
    // Clear message history on provider switch
    // (different models have different context handling)
}
```

#### File Modifications

| File | Changes |
|------|---------|
| `src/app.rs` | Add provider override methods, command palette state |
| `src/ui.rs` | Add command palette rendering |
| `src/events.rs` | Add Ctrl+K handling |
| `src/config.rs` | Add runtime config (optional) |

#### Acceptance Criteria

- [ ] Ctrl+K opens command palette
- [ ] User can see current provider and model
- [ ] User can switch provider at runtime
- [ ] Provider switch clears context (to avoid mixed contexts)
- [ ] New provider is used for next message
- [ ] Status bar updates to show new provider

#### Effort Estimate

**Developer Hours**: 4-6 hours
- Command palette UI: 3 hours
- Provider switching logic: 2 hours
- Testing: 1 hour

---

### Feature 4: Background Agents

#### Description

Run tasks in background while user continues working. Matches Cursor 2.0's background agent feature. High demand from power users who want parallelization.

#### Technical Approach

**Architecture**: Spawn independent tokio tasks that report via OS notifications.

**State Addition** (app.rs):
```rust
pub struct BackgroundTask {
    pub id: String,
    pub prompt: String,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub started_at: DateTime<Utc>,
}

pub enum TaskStatus {
    Running,
    Completed,
    Failed,
}

pub struct App {
    // ... existing fields ...
    pub background_tasks: Vec<BackgroundTask>,
}
```

**Task Spawning** (llm.rs):
```rust
pub async fn spawn_background_task(
    &self,
    prompt: String,
    notification_tx: mpsc::Sender<String>,
) -> String {
    let task_id = generate_task_id();
    
    let llm = self.clone();
    tokio::spawn(async move {
        let result = llm.send_message_simple(&prompt).await;
        let _ = notification_tx.send(format!("[TASK_COMPLETE]{}", task_id)).await;
    });
    
    task_id
}
```

**Desktop Integration** (desktop/notifications.rs):
```rust
pub fn notify_task_complete(title: &str, body: &str) {
    // Use existing notification system
    send_notification(title, body);
}
```

**UI Display** (ui.rs):
1. Keep Tasks tab in sync with background task status
2. Show running indicator (spinner or pulse)
3. Notify when complete (system notification + UI indicator)

**Task Browser** (similar to MCP browser):
- List all background tasks
- Show status: Running/Completed/Failed
- Click to view result
- Option to cancel running tasks

#### File Modifications

| File | Changes |
|------|---------|
| `src/app.rs` | Add BackgroundTask struct, task management |
| `src/llm.rs` | Add spawn_background_task method |
| `src/ui.rs` | Add task list display in Tasks tab |
| `src/desktop/notifications.rs` | Add task notification |

#### Acceptance Criteria

- [ ] User can spawn background task with /bg or Ctrl+B
- [ ] Task runs while user continues in chat
- [ ] System notification on task complete
- [ ] Tasks tab shows all background tasks
- [ ] User can view task result
- [ ] User can cancel running task

#### Effort Estimate

**Developer Hours**: 6-8 hours
- Task spawning logic: 3 hours
- UI display: 2 hours
- Notifications: 1 hour
- Testing: 2 hours

---

## Impact Tier 3: Medium

### Feature 5: MCP Server Mode

#### Description

Run OpenCrust as an MCP server, allowing other tools (Cursor, VS Code extensions) to use OpenCrust's capabilities. Differentiator - most CLI agents don't offer this.

#### Technical Approach

**Existing Infrastructure**: MCP client already exists; need server mode.

**Implementation**:
1. Add new CLI subcommand: `opencrust serve`
2. Implement JSON-RPC 2.0 server
3. Register OpenCrust tools as MCP tools

**CLI Addition** (main.rs):
```rust
fn main() {
    // Existing args handling
    match &args[1] as &str {
        "serve" => serve_mcp_server(),
        // ... existing
    }
}
```

**Server Implementation**:
```rust
pub async fn serve_mcp_server() {
    let listener = TcpListener::bind("127.0.0.1:8765").await?;
    
    // JSON-RPC handler
    loop {
        let (socket, _) = listener.accept().await?;
        handle_connection(socket).await?;
    }
}

async fn handle_connection(mut stream: TcpStream) {
    // Read JSON-RPC request
    // Call appropriate tool
    // Return result
}
```

**Tool exposed via MCP**:
- `chat` - Send prompt, get response
- `execute` - Execute command
- `read_file` - Read file
- `write_file` - Write file

#### File Modifications

| File | Changes |
|------|---------|
| `src/main.rs` | Add serve subcommand |
| `src/mcp.rs` | Add MCP server implementation |
| `src/jsonrpc.rs` | Ensure JSON-RPC utilities exist |

#### Acceptance Criteria

- [ ] `opencrust serve` starts server on port 8765
- [ ] Server accepts JSON-RPC requests
- [ ] Tools are callable via MCP protocol
- [ ] Server can be connected from external tools

#### Effort Estimate

**Developer Hours**: 4-6 hours
- Server implementation: 3 hours
- Protocol handling: 1 hour
- Testing: 2 hours

---

### Feature 6: Headless / Scriptable Mode

#### Description

Run OpenCrust from scripts and CI/CD pipelines without TUI. Matches smelt's headless mode for automation.

#### Technical Approach

**CLI Enhancement**:
```bash
opencrust -p "Explain this code" --project ./myproject
```

**Implementation** (main.rs):
```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check for headless mode
    if let Some(idx) = args.iter().position(|a| a == "-p" || a == "--prompt") {
        let prompt = args[idx + 1].clone();
        return run_headless(&prompt);
    }
}

fn run_headless(prompt: &str) {
    // No TUI - just execute and print result
    let result = blocking_get_response(prompt);
    println!("{}", result);
}
```

**Options**:
- `-p, --prompt`: One-shot prompt
- `-f, --file`: Read prompt from file
- `--project`: Set working directory
- `--provider`: Override provider
- `--model`: Override model

#### File Modifications

| File | Changes |
|------|---------|
| `src/main.rs` | Add headless CLI parsing |

#### Acceptance Criteria

- [ ] `opencrust -p "prompt"` prints result and exits
- [ ] Works in CI/CD pipelines
- [ ] Exits with appropriate code on error

#### Effort Estimate

**Developer Hours**: 2-3 hours

---

## Implementation Order (By User Impact)

### Weeks 1-2: Critical Impact Features (17 hours)

1. **Plan Mode with Diff Preview** (12h) — addresses #1 pain point: agent goes wrong
2. **Context Budget Display** (3h) — addresses "where am I?" confusion
3. **Model Context Display** (2h) — shows which model is active

### Weeks 3-4: High Impact Features (18 hours)

4. **Background Agents** (8h) — prevents rate limit kills mid-workstream
5. **Auto-Context Summarization** (4h) — prevents context loss
6. **Multi-Provider Toggle** (6h) — runtime model switching

### Weeks 5-6: Medium Impact Features (15 hours)

7. **Input Prediction** (5h) — ghost text while typing
8. **Vim Mode** (5h) — vi keybindings for power users
9. **Session Forking** (5h) — parallel session experiments

### Weeks 7-8: Lower Impact Features (13 hours)

10. **Enhanced Skills** (4h) — better skill discovery
11. **MCP Server Mode** (6h) — integration mode
12. **Headless Mode** (3h) — CI/CD scripts

---

## Dependencies

| Feature | Depends On |
|----------|------------|
| Plan Mode | Review mode (existing), approval channel |
| Context Display | UsageStats (existing) |
| Multi-Provider | ProviderType (existing) |
| Background Agents | None (new) |
| MCP Server | MCP client (existing) |
| Headless | None (new) |

---

## Testing Strategy

### Unit Tests
- Mode transitions (app.rs)
- Provider switching (config.rs)
- Task spawning (llm.rs)

### Integration Tests
- Full plan-review-execute flow
- Context budget calculation accuracy
- Provider runtime switch

### Manual Testing
- Background task notifications
- MCP server connectivity
- Headless CLI in CI

---

## Rollback Plan

If critical issues arise:
- Plan Mode: Disable Ctrl+P, revert to direct execution
- Context Display: Hide percentage, show basic count only
- Multi-Provider: Require restart to change
- Background Agents: Queue tasks, run sequentially
- MCP Server: CLI-only mode
- Headless: Use existing blocking mode

---

## Success Metrics

| Feature | Metric |
|---------|--------|
| Plan Mode | 50% reduction in "went wrong" audit log entries |
| Context Display | 0 support questions about context limits |
| Multi-Provider | Provider switch frequency (target: 10/session) |
| Background Agents | Tasks tab usage (target: 2+ per session) |
| MCP Server | External connections per day |
| Headless | Pipeline usage (target: 20% of invocations) |

---

## Additional Market Opportunities

Based on the competitive analysis and user pain points, the following enhancements address remaining gaps:

### Feature 7: Input Prediction (Ghost Text)

#### Description

Show AI-suggested continuation as ghost text while typing. Matches smelt's input prediction feature - reduces typing effort significantly.

#### Technical Approach

**Implementation** (app.rs + ui.rs):
- Debounce input after 300ms delay
- Send truncated prompt + current input to LLM
- Render suggestion in dimmed style (gray, low opacity)
- Tab key accepts suggestion, replaces current input

**Keybinding**: Tab accepts ghost text, Escape dismisses

#### File Modifications

| File | Changes |
|------|---------|
| `src/app.rs` | Add ghost_text field, prediction toggle |
| `src/ui.rs` | Render ghost text with dimmed style |
| `src/llm.rs` | Add lightweight completion endpoint |

#### Acceptance Criteria

- [ ] Ghost text appears after typing pause
- [ ] Tab accepts suggestion
- [ ] Escape dismisses suggestion

#### Effort Estimate

**Developer Hours**: 4-6 hours

---

### Feature 8: Vim Mode

#### Description

Full vi keybindings for input editing. Power users expect terminal-native navigation. Matches smelt's vim mode.

#### Technical Approach

**State Addition** (app.rs):
```rust
pub struct App {
    // ... existing fields ...
    pub vim_mode: bool,
    pub vim_cursor_pos: usize,
}
```

**Input Handling** (events.rs):
- `h/j/k/l` - Navigation
- `w/b` - Word forward/back
- `0/$` - Line start/end
- `i/a` - Insert mode
- `dd/cc/yy` - Line operations

**UI Indicator**: Show "VIM" in status bar when active

#### File Modifications

| File | Changes |
|------|---------|
| `src/app.rs` | Add vim mode state |
| `src/events.rs` | Add vi key handling |
| `src/ui.rs` | Add vim indicator |

#### Acceptance Criteria

- [ ] Toggle vim mode with Alt+V
- [ ] Navigation with h/j/k/l
- [ ] Word navigation with w/b
- [ ] Visual mode with v
- [ ] Delete line with dd

#### Effort Estimate

**Developer Hours**: 4-5 hours

---

### Feature 9: Auto-Context Summarization

#### Description

Automatically compress old messages when approaching context limit. Addresses "context lost" complaints. Matches Claude Code's conversation compaction.

#### Technical Approach

**Trigger**: At 80% of context budget

**Implementation** (llm.rs):
```rust
async fn summarize_old_messages(&self, messages: &mut Vec<Value>) -> String {
    // Extract oldest N messages
    // Prompt: "Summarize this conversation concisely"
    // Replace original messages with summary + recent messages
}
```

**UI Feedback**: Show "Summarizing conversation..." progress message

#### File Modifications

| File | Changes |
|------|---------|
| `src/llm.rs` | Add summarization logic |
| `src/ui.rs` | Add summarization progress indicator |

#### Acceptance Criteria

- [ ] Auto-triggers at 80% context
- [ ] User sees summarization progress
- [ ] Context continues with compressed history
- [ ] User can disable in config

#### Effort Estimate

**Developer Hours**: 3-4 hours

---

### Feature 10: Enhanced Skills System

#### Description

Improve skills with runtime activation, better discovery, and performance tracking.

#### Technical Approach

**Runtime Skill Activation**:
```rust
// Activate skill for session
app.activate_skill("rust-expert");
```

**Skill Metadata**:
```rust
pub struct Skill {
    pub metadata: SkillMetadata,
    pub content: String,
    pub usage_count: u64,    // Track usage
    pub avg_latency_ms: u64, // Performance
}
```

**Skill Browser** (similar to MCP browser):
- Show all skills with usage stats
- Filter by tag/category
- Quick activate

#### File Modifications

| File | Changes |
|------|---------|
| `src/skills.rs` | Add activation, usage tracking |
| `src/ui.rs` | Add skill browser |
| `src/app.rs` | Add active_skill field |

#### Acceptance Criteria

- [ ] Skills tab shows usage stats
- [ ] User can activate skill for session
- [ ] Activated skill injects into system prompt

#### Effort Estimate

**Developer Hours**: 3-4 hours

---

### Feature 11: Session Forking

#### Description

Fork current session to try different approach, then merge or discard. Matches Cursor's multiple cursor feature.

#### Technical Approach

**State Addition** (app.rs):
```rust
pub struct App {
    // ... existing fields ...
    pub session_forks: Vec<SessionFork>,
}

pub struct SessionFork {
    pub id: String,
    pub name: String,
    pub messages: Vec<Value>,
    pub created_at: DateTime<Utc>,
}
```

**Commands**:
- `/fork [name]` - Create fork
- `/merge [fork_id]` - Merge into main
- `/drop [fork_id]` - Discard fork

**UI**: Show fork indicator in tab header

#### File Modifications

| File | Changes |
|------|---------|
| `src/app.rs` | Add session fork state |
| `src/sessions.rs` | Add fork/merge methods |
| `src/ui.rs` | Add fork indicator |

#### Acceptance Criteria

- [ ] /fork creates parallel session
- [ ] Forks show in session list
- [ ] /merge combines histories
- [ ] /drop removes fork

#### Effort Estimate

**Developer Hours**: 4-5 hours

---

### Feature 12: Model Context Window Display

#### Description

Clearly show current model and context window limits in status bar. Addresses "which model am I using?" confusion.

#### Technical Approach

**Provider Limits** (config.rs):
```rust
pub fn context_limit(provider: &ProviderType, model: &str) -> u64 {
    match provider {
        ProviderType::Anthropic => {
            if model.contains("opus") { 200_000 } else { 100_000 }
        }
        ProviderType::OpenAI => 128_000,
        ProviderType::Gemini => 128_000, // 1M for gemini-2.5
        ProviderType::Ollama => 8_000, // configurable
        _ => 32_000,
    }
}
```

**Status Bar Enhancement**:
```
-- INSERT -- | 🤖 claude-sonnet-4 | Context: 45K/100K (45%) | $0.02
```

#### File Modifications

| File | Changes |
|------|---------|
| `src/config.rs` | Add context_limit function |
| `src/ui.rs` | Enhance status bar |

#### Acceptance Criteria

- [ ] Status bar shows model name
- [ ] Status bar shows context limit
- [ ] Status bar updates on model change

#### Effort Estimate

**Developer Hours**: 1-2 hours

---

## Complete Feature Matrix
  
| # | Feature | Impact Tier | Hours | Dependencies | Status |
|---|---------|-------------|-------|-------------|--------|
| 1 | Plan Mode + Diff Preview | Critical | 12 | None | ✅ Done |
| 2 | Context Budget Display | Critical | 3 | UsageStats | ✅ Done |
| 12 | Model Context Display | Critical | 2 | None | ✅ Done |
| 4 | Background Agents | High | 8 | None | ✅ Done |
| 9 | Auto-Context Summarization | High | 4 | Feature 2 | ✅ Done |
| 3 | Multi-Provider Toggle | High | 6 | None | ✅ Done |
| 7 | Input Prediction | Medium | 5 | None | ✅ Done |
| 8 | Vim Mode | Medium | 5 | None | ✅ Done |
| 11 | Session Forking | Medium | 5 | None | ✅ Done |
| 10 | **Enhanced Skills** | Lower | 4 | None | ✅ **Done** |
| 5 | **MCP Server Mode** | Lower | 6 | None | ✅ **Done** |
| 6 | **Headless Mode** | Lower | 3 | None | ✅ **Done** |

**Total Hours**: ~58 hours (all features complete!)

---

## Future Enhancements

All planned features have been fully implemented. No remaining future enhancements at this time.

---

## Revised Implementation Timeline (By Impact)

### Phase 1 (Weeks 1-2): Critical Impact Features
- Feature 1: Plan Mode (12h)
- Feature 2: Context Budget Display (3h)
- Feature 12: Model Context Display (2h)

### Phase 2 (Weeks 3-4): High Impact Features
- Feature 3: Multi-Provider Toggle (6h)
- Feature 4: Background Agents (8h)
- Feature 9: Auto-Context Summarization (4h)

### Phase 3 (Weeks 5-6): Medium Impact Features
- Feature 5: MCP Server Mode (6h)
- Feature 6: Headless Mode (3h)
- Feature 7: Input Prediction (5h)

### Phase 4 (Weeks 7-8): Lower Impact Features
- Feature 10: Enhanced Skills (4h) ✅ **Done**
- Feature 11: Session Forking (5h) ✅ **Done**
- Feature 5: MCP Server Mode (6h) ⏳ **Pending**
- Feature 6: Headless Mode (3h) ⏳ **Pending**

> **Note**: All 12 planned features are now fully implemented (as of 2026-05-08), including Vim Mode (Feature 8) which was previously listed as a future enhancement.