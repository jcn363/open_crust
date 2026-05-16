# OpenCrust Roadmap

> Consolidated plan generated 2026-05-16.
> Replaces: `.uncensored/FINAL_REPORT.md`, `MARKET.md` action items (superseded).
> Status: ✅ Phase 1-17 complete. 152 tests pass, 0 fail. Build/clippy/fmt: clean.

---

## ✅ Completed Work (v0.1.3)

### Core Platform (Phases 1-8)
- Rust TUI with Ratatui (terminal UI, chat/tasks views, file tree sidebar)
- LLM client loop with 11 provider integrations
- Tool system: MCP servers, LSP integration, custom script tools
- RAG (semantic search), skills system (11 built-in), task planner
- Security: granular permissions, network gating, audit logging, compliance
- Desktop: Cinnamon/GNOME/Plasma detection, notifications, file pickers
- Utilities: git integration, web search, formatters, sessions, JSON utils

### Quality & Compliance (Phases 9-12)
- **#[allow] → #[expect] migration** — 33 instances across 14 files (Phase 9)
- **Market analysis** — competitive deep-dive vs Claude Code, Cursor, Cline, OpenCode, OpenDev (Phase 11)
- **Performance benchmarks** — startup 7-10ms, binary 16MB, memory ~18MB (Phase 12)
- **Zero-compromise audit** — 0 unsafe, 0 prod unwraps, 0 TODOs, 0 unused deps (Phase 16)

### Market-Driven Features (Phases 13-17)
- **Plan Mode enforcement** — Ctrl+P toggles Planning mode, blocks write/edit/bash tools (Phase 13)
- **`/goal` command** — persistent objective with system prompt injection (Phase 14)
- **Mission Control wiring** — live orchestrator tasks bridge, real-time DAG visualization (Phase 15)
- **Codebase cleanup** — removed 5 unnecessary async functions, deleted dead `ProviderType::as_str()` duplicate (Phase 17)

---

## Phase 3: Code Organization (Pending)

### Goal
Refactor `main.rs` (2,222 lines) into manageable modules.

### Rationale
- `main.rs` is **2.5x larger** than the next largest module
- Makes onboarding harder, testing harder, and parallel development impossible
- Referenced as highest-priority technical debt in every audit (Phase 12, 16, FINAL_REPORT)

### Approach
```
src/
├── main.rs              # Entry point only (~80 lines: CLI dispatch + init)
├── app.rs               # App state, PlanMode enum (keep, trim)
├── cli.rs               # NEW — CLI subcommand parsing and dispatch
├── startup.rs           # NEW — Manager initialization, config loading
└── event_loop.rs        # NEW — Main event loop, key handling dispatch
```

### Estimated Effort
- **Preparation**: 1h — audit all `main.rs` dependencies, identify extraction boundaries
- **Extraction**: 2-3h — create `cli.rs`, `startup.rs`, `event_loop.rs`, wire up
- **Verification**: 1h — test all CLI subcommands, keybindings, UI states
- **Total**: ~4-5 hours

### Risks
- `main.rs` currently has deeply interwoven initialization + event loop logic
- Some global/shared state (`Manager` struct) needs careful split
- All CLI subcommands (`desktop detect`, `session list`, etc.) must continue working

### Acceptance Criteria
- [ ] `main.rs` < 200 lines
- [ ] `cargo build` clean
- [ ] `cargo clippy -D warnings` clean
- [ ] `cargo test` — all 152 tests pass
- [ ] All CLI subcommands functional
- [ ] Ctrl+P plan mode toggle works
- [ ] Ctrl+G mission control opens

---

## Phase 4: Test Coverage (Pending)

### Goal
Expand test coverage from current levels to >40% of non-trivial modules.

### Current State
| Module | Test Lines | Assessment |
|--------|-----------|------------|
| `skills.rs` | 19 tests | ✅ Good |
| `security.rs` | 17 tests | ✅ Good |
| `rag.rs` | 11 tests | ✅ Adequate |
| `mcp_showcase/tui.rs` | 9 tests | ✅ Good |
| `audit.rs` | 8 tests | ⚠️ Adequate |
| `models.rs` | 7 tests | ⚠️ Light |
| `orchestrator/task.rs` | 7 tests | ⚠️ Light |
| `orchestrator/agent_pool.rs` | 5 tests | ⚠️ Light |
| `orchestrator/coordinator.rs` | 4 tests | ⚠️ Light |
| `sessions.rs` | 4 tests | ⚠️ Light |
| `desktop/file_picker.rs` | 3 tests | ⚠️ Light |
| `markdown.rs` | 3 tests | ⚠️ Light |
| `json_utils.rs` | 3 tests | ⚠️ Light |
| `llm.rs` | 0 tests | ❌ Missing |
| `app.rs` | 0 tests | ❌ Missing |
| `planner.rs` | 0 tests | ❌ Missing |
| `tools.rs` | 0 tests | ❌ Missing |
| `permissions.rs` | 0 tests | ❌ Missing |
| `ui.rs` | 0 tests | ❌ Missing |

### Priority Modules (No Tests)
1. **`llm.rs`** — Core LLM loop, tool execution, plan/goal mode logic — highest risk
2. **`app.rs`** — Application state, PlanMode, mode transitions
3. **`planner.rs`** — Task plan creation and step management
4. **`tools.rs`** — Tool schema definitions and routing
5. **`permissions.rs`** — Security permission checking

### Approach
- **Unit tests first**: pure logic modules (`planner.rs`, `json_utils.rs`, `config.rs`)
- **Integration for critical**: `llm.rs` tool filtering in plan mode, `app.rs` mode switching
- **Property-based**: `permissions.rs` — verify all access patterns are correct
- **Snapshot**: `tools.rs` — ensure tool schemas don't silently change

### Estimated Effort
- Phase 4a: Planner + JsonUtils + Config tests (~2h)
- Phase 4b: Permissions + Tools tests (~2h)
- Phase 4c: App state + PlanMode tests (~2h)
- Phase 4d: LLM loop integration tests (~3h)
- **Total**: ~9 hours across 4 sub-phases

### Acceptance Criteria
- [ ] Every public module has at least basic unit tests
- [ ] `llm.rs` has tests for plan mode tool filtering
- [ ] `app.rs` has tests for PlanMode state transitions
- [ ] `permissions.rs` has property-based tests
- [ ] `cargo test` — all tests pass
- [ ] No test-only `#[allow]` violations

---

## Phase 5: Market Positioning (0-60 Days)

### Goal
Convert OpenCrust's technical advantages into community presence and adoption.

### Action Items (from MARKET.md analysis)

| # | Action | Effort | Priority | Status |
|---|--------|--------|----------|--------|
| 1 | **Publish performance benchmarks** | Low (1-2h) | 🔥 High | ⏳ Not started |
| 2 | **Publish to crates.io** | Low (1h) | 🔥 High | ⏳ Not started |
| 3 | **Strengthen README** | Medium (3-4h) | 🔥 High | ⏳ Not started |
| 4 | **macOS notification support** | Medium (2-3h) | 🟡 Medium | ⏳ Not started |
| 5 | **Token budget/cost dashboard** | Medium (3-4h) | 🟡 Medium | ⏳ Not started |
| 6 | **Visual diff / plan review UX** | Medium (3-4h) | 🟡 Medium | ⏳ Not started |
| 7 | **Cross-platform desktop** | High (1-2w) | 🔵 Low | ⏳ Not started |

### Key Insight
OpenDev proved "publish Rust benchmarks → massive growth" works. OpenCrust has similar performance characteristics (7-10ms startup, 16MB binary, 18MB memory) and should publish immediately.

### Quick Wins (This Week)
1. Restructure README with hero benchmarks, demo GIF, feature comparison table
2. `cargo publish` to crates.io
3. Add macOS notification backend
4. Publish benchmark comparison blog post / GitHub discussion

---

## Appendix: Cross-References

| Document | Staleness | Action Taken |
|----------|-----------|-------------|
| `.uncensored/FINAL_REPORT.md` | Very stale (May 12, duplicate content, stale metrics) | Superseded by this ROADMAP.md. Archived. |
| `MARKET.md` | Partially stale (action items superseded) | Keep as market reference. Action items migrated here. |
| `.uncensored/state.json` | Fresh | Update pending items to reference this ROADMAP. |
| `docs/ARCHITECTURE.md` | Fresh | No changes needed. |

---

*Next update trigger: when any Phase 3-5 item is completed or market conditions shift significantly.*
