# Plan: Extract Mode Handlers from event_loop/mod.rs

## Current State
- `event_loop/mod.rs`: 977 lines
- 10 mode handlers inline in the main event loop
- Tightly coupled with `App` state and various managers

## Target State
- `event_loop/mod.rs`: ~300 lines (main loop + dispatch)
- `event_loop/modes/` directory with 10 mode handler modules
- `event_loop/modes/mod.rs` - unified dispatch
- `event_loop/modes/tests.rs` - extracted tests (if any)
- Clean builds, all tests pass

---

## Mode Handlers to Extract

| Mode | Lines | Complexity | Dependencies |
|------|-------|------------|--------------|
| Normal | ~186 | High | App, clipboard, skill_manager, plugin_manager, config |
| Insert | ~215 | High | App, clipboard, custom_commands, formatters, share |
| Review | ~85 | Medium | App, config |
| Servers | ~52 | Medium | App, config, mcp_browser_items |
| SkillBrowser | ~54 | Medium | App, skill_manager |
| PluginBrowser | ~48 | Medium | App, plugin_manager |
| CommandPalette | ~71 | Medium | App, config |
| Help | ~5 | Low | App |
| McpShowcase | ~19 | Low | App, mcp_showcase_ui |
| MissionControl | ~9 | Low | App, mission_control_ui, orchestrator_tasks |

---

## Phase 1: Create Directory Structure & Types

### 1.1 Create modes directory
```bash
mkdir -p src/event_loop/modes
```

### 1.2 Create shared types module
**File:** `src/event_loop/modes/types.rs`
- `ModeHandler` trait with `handle_key(&mut App, KeyEvent, &mut Context) -> ModeAction`
- `ModeAction` enum: `Continue`, `ExitMode`, `SwitchMode(Mode)`, `Quit`
- `HandlerContext` struct holding shared dependencies:
  - `skill_manager: Arc<Mutex<SkillManager>>`
  - `plugin_manager: Arc<Mutex<PluginManager>>`
  - `clipboard: &mut ClipboardManager`
  - `config: &Config`
  - `llm_client: &LlmClient`
  - `orchestrator_tasks: Option<Arc<Mutex<TaskState>>>`

### 1.3 Create mode module declarations
**File:** `src/event_loop/modes/mod.rs`
- Declare all mode modules
- Export `ModeHandler` trait and `HandlerContext`
- Provide `dispatch_mode(app, key, context)` function

---

## Phase 2: Extract Individual Mode Handlers

### 2.1 Normal Mode (`normal.rs`)
- Extract lines 200-386
- Handle: navigation, sidebar, mode switching, vim toggle, background tasks, formatting
- Dependencies: clipboard, skill_manager, plugin_manager, config, llm_client

### 2.2 Insert Mode (`insert.rs`)
- Extract lines 387-602
- Handle: input editing, vim keys, slash commands, custom commands, format, share
- Dependencies: clipboard, custom_commands, formatters, share, llm_client

### 2.3 Review Mode (`review.rs`)
- Extract lines 603-688
- Handle: diff navigation, approve/deny, execute, cancel
- Dependencies: App (proposed_changes, plan_review_index)

### 2.4 Servers Mode (`servers.rs`)
- Extract lines 689-741
- Handle: MCP server browser, install
- Dependencies: App, config

### 2.5 SkillBrowser Mode (`skill_browser.rs`)
- Extract lines 742-796
- Handle: skill list navigation, toggle activation
- Dependencies: App, skill_manager

### 2.6 PluginBrowser Mode (`plugin_browser.rs`)
- Extract lines 797-845
- Handle: plugin list navigation, toggle enable/disable
- Dependencies: App, plugin_manager

### 2.7 CommandPalette Mode (`command_palette.rs`)
- Extract lines 846-917
- Handle: provider switch, model switch, clear context, MCP browser
- Dependencies: App, config

### 2.8 Help Mode (`help.rs`)
- Extract lines 918-923
- Handle: exit on Esc/q
- Dependencies: App

### 2.9 McpShowcase Mode (`mcp_showcase.rs`)
- Extract lines 924-943
- Handle: delegate to mcp_showcase_ui
- Dependencies: App, mcp_showcase_ui

### 2.10 MissionControl Mode (`mission_control.rs`)
- Extract lines 944-953
- Handle: delegate to mission_control_ui, refresh tasks
- Dependencies: App, mission_control_ui, orchestrator_tasks

---

## Phase 3: Wire Up Dispatch

### 3.1 Update `event_loop/modes/mod.rs`
- Implement `dispatch_mode()` that matches on `app.mode` and calls appropriate handler
- Pass `HandlerContext` with all required dependencies

### 3.2 Update `event_loop/mod.rs`
- Remove all inline mode match arms (lines 200-953)
- Add `mod modes;` declaration
- Call `modes::dispatch_mode(&mut app, key, &mut context)` in main loop
- Create `HandlerContext` with all dependencies before loop

---

## Phase 4: Test Separation

### 4.1 Check for existing tests
- Search for `#[cfg(test)]` in event_loop/mod.rs
- Search for mode-specific tests

### 4.2 Create `event_loop/modes/tests.rs`
- Extract any inline tests
- Add integration tests for mode transitions
- Add unit tests for individual handlers

### 4.3 Update `event_loop/modes/mod.rs`
- Add `#[cfg(test)] mod tests;`

---

## Phase 5: Verification

### 5.1 Build checks
```bash
cargo check
cargo clippy -- -D warnings
cargo fmt -- --check
```

### 5.2 Test checks
```bash
cargo test
cargo test --lib event_loop
```

### 5.3 Integration test
- Run the TUI manually to verify all modes work
- Test mode transitions: Normal ↔ Insert, Normal → Help, Normal → CommandPalette, etc.

---

## Phase 6: Commit & Push

```bash
git add -A
git commit -m "refactor: extract mode handlers from event_loop/mod.rs

- Created event_loop/modes/ with 10 mode handler modules
- Normal, Insert, Review, Servers, SkillBrowser, PluginBrowser,
  CommandPalette, Help, McpShowcase, MissionControl
- Unified dispatch via ModeHandler trait
- All 372 tests pass, clippy clean, fmt clean"
git push
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Tight coupling with App | Pass `&mut App` to handlers; use `HandlerContext` for external deps |
| Async in handlers (skill/plugin toggle) | Keep tokio::spawn in handlers; pass managers via context |
| Mode transitions | Return `ModeAction::SwitchMode(Mode)` from handlers |
| Clipboard access | Include in `HandlerContext` |
| Config mutation | Include `&mut Config` in context or use `App.config` |

---

## Dependencies Between Phases

```
Phase 1 (types, mod.rs) 
    → Phase 2 (10 mode files, can be done in parallel)
    → Phase 3 (wire dispatch)
    → Phase 4 (tests)
    → Phase 5 (verify)
    → Phase 6 (commit)
```

---

## Estimated Effort

| Phase | Files | Est. Lines | Time |
|-------|-------|------------|------|
| 1 | 2 | ~100 | 30 min |
| 2 | 10 | ~700 | 2-3 hrs |
| 3 | 2 | ~50 | 30 min |
| 4 | 1 | ~100 | 30 min |
| 5 | - | - | 30 min |
| 6 | - | - | 10 min |
| **Total** | **15** | **~950** | **4-5 hrs** |

---

## Notes

- No backward compatibility required
- All tests must be in separate `tests.rs` files
- Use `cargo check` incrementally during Phase 2 to catch errors early
- The `Normal` and `Insert` modes are the most complex - tackle them first
- Consider extracting shared navigation logic (Up/Down with scroll) to a helper