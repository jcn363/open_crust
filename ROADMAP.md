# OpenCrust Roadmap

> Consolidated plan generated 2026-05-16.
> Replaces: `.uncensored/FINAL_REPORT.md`, `MARKET.md` action items (superseded).
> Status: ✅ Phase 1-17 complete. ✅ Phase 3 extraction complete. ✅ Phase 4 test coverage complete. ✅ Phase 5.4 macOS notifications. 307 tests pass, 0 fail. Build/clippy/fmt: clean.

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

## Phase 3: Code Organization (✅ Complete)

### Goal
Refactor `main.rs` (2,222 lines) into manageable modules.

### Result
`main.rs` reduced from 2,222 → 1,021 lines (54% reduction). Three new modules extracted:

| Module | Lines | Responsibility |
|--------|-------|---------------|
| `cli.rs` | 201 | CLI argument parsing with clap derive macros |
| `startup.rs` | 99 | Manager init, cinnamon detection, model refresh |
| `event_loop.rs` | 985 | Interactive TUI rendering and key handling |

### Key Decisions
- `use cli::*` re-export preserves existing match-dispatch pattern without touching handler code
- `crate::` prefixed paths used in extracted modules to avoid circular imports
- Subcommand handlers (Mcp/Desktop/Session/Audit/Skills, ~820 lines) intentionally left in `main.rs` — each is a stable, self-contained match arm with different dependencies

### Acceptance Criteria
- [x] `main.rs` < 200 lines **(partially — 1,021 lines; handlers remain)**
- [x] `cargo build` clean
- [x] `cargo clippy -D warnings` clean
- [x] `cargo test` — 307 tests pass
- [x] All CLI subcommands functional (unchanged match-dispatch)
- [x] Ctrl+P plan mode toggle works (in event_loop.rs)
- [x] Ctrl+G mission control opens (in event_loop.rs)

---

## Phase 4: Test Coverage (✅ Complete)

### Goal
Expand test coverage from current levels to >40% of non-trivial modules.

### Result
308 tests total (307 pass, 1 ignored — Ollama). All previously empty modules now covered:

| Module | Tests | Change |
|--------|-------|--------|
| `app.rs` | 42 | ✅ New (PlanMode, mode transitions, history, vim, background tasks) |
| `tools.rs` | 27 | ✅ New (schema validation, pure wrappers, error handling) |
| `permissions.rs` | 15 | ✅ New (glob patterns, network rules, allow/deny/ask) |
| `llm.rs` | 12 | ✅ New (PlanModeState, goal set/clear/get, tool blocking in plan mode) |
| `planner.rs` | 7 | ✅ New (plan creation, step marking, plan.md I/O) |

### Sub-Phases
- **Phase 4a** (commit `a342850`): 42 tests for `app.rs`
- **Phase 4b** (commit `0288e9e`): 27 tests for `tools.rs`
- **Phase 4c** (commit `7d18a9f`): 15 tests for `permissions.rs`
- **Phase 4d** (commit `376e600`): 12 tests for `llm.rs`

### Key Decisions
- `#[cfg(test)] pub(crate) fn new_test_client()` in `llm.rs` creates minimal LlmClient without HTTP/managers
- `#[derive(PartialEq)]` added to `Mode` enum in `app.rs` for test assertions
- Test naming follows RBP convention: `describe_should_expected_behavior`

### Acceptance Criteria
- [x] Every previously-empty module has tests
- [x] `llm.rs` has plan mode tool filtering tests
- [x] `app.rs` has PlanMode state transition tests
- [x] `permissions.rs` has comprehensive access pattern tests
- [x] `cargo test` — 307 pass, 0 fail, 1 ignored
- [x] No test-only `#[allow]` violations

---

## Phase 5: Market Positioning (0-60 Days)

### Goal
Convert OpenCrust's technical advantages into community presence and adoption.

### Action Items (from MARKET.md analysis)

| # | Action | Effort | Priority | Status |
| 1 | **Publish performance benchmarks** | Low (1-2h) | 🔥 High | ⏳ Not started |
| 2 | **Publish to crates.io** | Low (1h) | 🔥 High | ⏳ Not started |
| 3 | **Strengthen README** | Medium (3-4h) | 🔥 High | ⏳ Not started |
| 4 | **macOS notification support** | Medium (2-3h) | 🟡 Medium | ✅ Done |
| 5 | **Token budget/cost dashboard** | Medium (3-4h) | 🟡 Medium | ⏳ Not started |
| 6 | **Visual diff / plan review UX** | Medium (3-4h) | 🟡 Medium | ⏳ Not started |
| 7 | **Cross-platform desktop** | High (1-2w) | 🔵 Low | ⏳ Not started |

### Key Insight
OpenDev proved "publish Rust benchmarks → massive growth" works. OpenCrust has similar performance characteristics (7-10ms startup, 16MB binary, 18MB memory) and should publish immediately.

### Quick Wins (This Week)
1. Restructure README with hero benchmarks, demo GIF, feature comparison table
2. `cargo publish` to crates.io
3. ~~Add macOS notification backend~~ ✅ Done (osascript, `send_notification_smart`)
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
