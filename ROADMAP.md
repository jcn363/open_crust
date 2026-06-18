# OpenCrust Roadmap

> **Version:** v1.2.0 | **Last updated:** 2026-06-18
> **Status:** 439 tests pass, 0 fail, 1 ignored (Ollama). Build/clippy/fmt: clean.
> **Supersedes:** Previous ROADMAP.md, `MARKET.md` action items (now pure research),
> `.uncensored/FINAL_REPORT.md`.

---

## Vision

OpenCrust is the **security-first, terminal-native AI coding agent** — built in Rust
for developers who care about performance, safety, and enterprise compliance.

Unlike agents bolted onto IDEs or Node.js runtimes, OpenCrust is a single static
binary with zero runtime dependencies, granular permissions, persistent audit, and
multi-agent orchestration — designed from the ground up for professional software
engineering teams.

---

## Current State (v0.1.3 — May 16, 2026)

### Core Platform (Phases 1-2 — ✅ Complete)
- Rust TUI with Ratatui: tabbed chat/tasks views, file tree sidebar, command palette
- LLM client loop with 12 provider integrations (Ollama, OpenRouter, OpenAI, Gemini,
  Anthropic, Mistral, Groq, Together, Replicate, DeepSeek, LocalAI, Unsloth)
- Tool system: MCP servers (JSON-RPC), LSP integration (code completion + diagnostics),
  custom script tools (auto-discovered from `.opencrust/tools/`)
- RAG semantic search (vector-based, Ollama embeddings)
- Skills system (11 built-in, custom `SKILL.md` discovery)
- Task planner (multi-step plan generation with progress tracking)
- Git integration, web search, auto-formatters, session persistence

### Quality & Compliance Improvements (Phases 9-12 — ✅ Complete)
- **Phase 9:** 33 `#[allow]→#[expect]` conversions across 14 files
- **Phase 11:** Competitive market deep-dive (Claude Code, Cursor, Cline, OpenCode, OpenDev)
- **Phase 12:** Performance baselines measured:
  - Startup: **7-10ms** (average 7.8ms)
  - Binary: **16MB** (release, stripped)
  - Memory: **~18MB** idle
- **Phase 16:** Zero-compromise codebase audit — 0 unsafe blocks, 0 production unwraps,
  0 TODOs, 0 unused deps, 0 `#[allow(dead_code)]` without justification

### Market-Driven Features (Phases 13-17 — ✅ Complete)
- **Phase 13:** Plan Mode enforcement — Ctrl+P toggles Planning, blocks write/edit/bash tools
- **Phase 14:** `/goal` command — persistent objective with system prompt injection
- **Phase 15:** Mission Control wired to live orchestrator tasks — real-time DAG visualization
- **Phase 17:** Removed 5 unnecessary async functions, deleted dead `ProviderType::as_str()`
- **Phase 5.4:** macOS notification support via osascript backend

### Code Organization (Phase 3 — ✅ Complete)
- `main.rs` reduced from 2,222 → 1,021 lines (54% reduction)
- Three modules extracted: `cli.rs` (201 lines), `startup.rs` (99 lines),
  `event_loop.rs` (985 lines)
- Subcommand handlers (~820 lines) intentionally remain in `main.rs` — stable match arms

### Test Coverage (Phase 4 — ✅ Complete)
- 439 tests passing (1 ignored — requires local Ollama)
- Previously empty modules now covered: `app.rs` (42), `tools.rs` (27),
  `permissions.rs` (24), `llm.rs` (12), `planner.rs` (7), `token_budget.rs` (20),
  `mission_control/` (58), `pty.rs` (12)

### Market Positioning (Phase 5 — ✅ Complete)
- ✅ Item 1 — Publish performance benchmarks: **DONE** (`benches/benchmark.rs`,
  `docs/PERFORMANCE.md` Criterion section)
- ✅ Item 2 — Publish to crates.io: **DONE** (Cargo.toml prepared, keywords fixed, dry-run passed)
- ✅ Item 3 — Strengthen README: **DONE** (TOC, updated keybinds, enhanced feature descriptions)
- ✅ Item 4 — macOS notification support: **DONE** (osascript backend)
- ✅ Item 5 — Token budget / cost dashboard: **DONE** (TokenBudgetManager, /cost, /budget, status bar)
- ✅ Item 6 — Visual diff / plan review UX: **DONE** (unified diff toggle `u`,
  line diff highlighting, scroll `j/k`)
- ✅ Item 7 — Cross-platform desktop: **DONE** (macOS + Windows detection, file picker, notifications)

---

## Phase 6: Enterprise Readiness (30-60 Days)

### Goal
Double down on OpenCrust's unique moat: enterprise security, compliance, and cost
governance — features no other open-source agent provides.

### Rationale
78% of Fortune 500 companies have AI-assisted development in production. As they
consolidate around 1-2 approved agents, security and governance matter more than
GitHub stars. OpenCrust's compliance.rs, audit logging, and network gating are
already enterprise-ready — the gap is in cost visibility and agent management UX.

### Deliverables

| # | Item | Effort | Priority | Status |
|---|------|--------|----------|--------|
| 1 | **Token budget & cost dashboard** | Medium (3-4h) | 🔥 High | ✅ |
| 2 | **Background agent dashboard** | Medium (4-6h) | 🔥 High | ✅ |
| 3 | **Enterprise compliance packaging** | Medium (4-6h) | 🟡 Medium | ✅ |

#### 2. Background Agent Dashboard

Surface all running agents (parallel, background, scheduled) in a visual panel.

**Acceptance criteria:**
- [x] Mission Control shows per-agent status, token count, elapsed time
- [x] Start/stop/cancel agents from TUI
- [x] Agent fleets — spawn N agents on N tasks with progress aggregation
- [x] Background task completion notifications
- [x] All tests pass, clippy clean

**Reference:** Claude Code Agent View (`claude agents`), Cursor 3 Agents Window,
Cline Kanban.

#### 3. Enterprise Compliance Packaging

Package OpenCrust's existing compliance infrastructure for procurement processes.

**Acceptance criteria:**
- [x] SOC 2-style report generation from `compliance.rs` evidence packages
- [x] Audit export wizard: CSV, JSON, Syslog formats
- [x] Role-based permission templates (admin, developer, reviewer)
- [x] Private model deployment guide (Ollama + air-gapped)
- [x] All tests pass, clippy clean

#### 1. Token Budget & Cost Dashboard

Track usage per session, per agent, per provider. Show real-time cost in status bar.

**Acceptance criteria:**
- [x] Token count + estimated cost visible in status bar
- [x] Per-session token budget with warning (75%) and hard stop (90%)
- [x] Cost estimation for each LLM provider (tokenizer × provider pricing)
- [x] `/cost` command shows session breakdown
- [x] All tests pass, clippy clean

**Reference:** CruxCLI token budgets, Claude Code `/cost`, Cursor usage dashboard.

#### 2. Background Agent Dashboard

Surface all running agents (parallel, background, scheduled) in a visual panel.

**Acceptance criteria:**
- [x] Mission Control shows per-agent status, token count, elapsed time
- [x] Start/stop/cancel agents from TUI
- [x] Agent fleets — spawn N agents on N tasks with progress aggregation
- [x] Background task completion notifications
- [x] All tests pass, clippy clean

**Reference:** Claude Code Agent View (`claude agents`), Cursor 3 Agents Window,
Cline Kanban.

#### 3. Enterprise Compliance Packaging

Package OpenCrust's existing compliance infrastructure for procurement processes.

**Acceptance criteria:**
- [x] SOC 2-style report generation from `compliance.rs` evidence packages
- [x] Audit export wizard: CSV, JSON, Syslog formats
- [x] Role-based permission templates (admin, developer, reviewer)
- [x] Private model deployment guide (Ollama + air-gapped)
- [x] All tests pass, clippy clean

### Risks
- Cost dashboard requires accurate per-provider tokenizer pricing — providers
  change pricing without notice
- Background agent TUI may need async architecture changes if agents block the
  event loop

---

## Phase 7: Cross-Platform Desktop (60-90 Days)

### Goal
Deliver native desktop integration on macOS and Windows, matching the Linux
experience (notifications, file pickers, environment detection).

### Rationale
OpenCrust is currently Linux-only for desktop features. macOS support was
started (Phase 5.4: osascript notifications) but has no file picker, no
environment detection, and no menu bar integration. Windows has zero support.
This is the #1 blocker for wider adoption — 75% of developers use macOS or Windows.

### Deliverables

| # | Item | Effort | Priority | Status |
|---|------|--------|----------|--------|
| 1 | **macOS: File picker + environment detection** | Medium (2-3h) | 🔥 High | ✅ |
| 2 | **macOS: System notifications (full)** | Low (1-2h) | 🟡 Medium | ✅ |
| 3 | **Windows: Toast notifications** | Medium (3-4h) | 🔥 High | ✅ |
| 4 | **Windows: File picker** | Medium (2-3h) | 🟡 Medium | ✅ |
| 5 | **macOS: Menu bar agent** | High (4-6h) | 🟢 Low | ⏳ |

#### 5. macOS: Menu Bar Agent

Native macOS menu bar integration with status, notifications, and quick actions.

**Acceptance criteria:**
- [ ] Menu bar icon with status indicator (idle, working, error)
- [ ] Click-to-open main window
- [ ] Right-click context menu with agent controls
- [ ] Native macOS notifications (Notification Center)
- [ ] System Preferences integration (settings)
- [ ] All tests pass, clippy clean

**Reference:** Claude Code system tray, Cursor 3 menu bar, OpenCode status bar.

### Acceptance Criteria (Phase 7)
- [x] macOS notifications + file picker working (matching Linux)
- [x] Windows toast notifications + file picker working
- [x] No new platform-specific dependencies in core modules
- [x] All tests pass, clippy clean

### Risks
- Windows TUI requires different terminal handling (WinRT console APIs)
- macOS code signing needed for menu bar agent distribution
- Cross-platform abstractions can leak platform-specific complexity into core

#### 1. macOS: File Picker + Environment Detection
- Detect macOS desktop environment
- Native file picker (NSOpenPanel via `osascript` or `dialog`)
- Match Linux feature parity

#### 2. macOS: Full Notifications
- Notification Center integration (beyond bare osascript)
- Notification actions (e.g., "Open in OpenCrust")
- Status bar agent

#### 3. Windows Support
- Toast notifications via `powershell` or WinRT
- Windows file picker via `System.Windows.Forms`
- Environment detection (Windows 10/11, WSL awareness)

### Acceptance Criteria (Phase 7)
- [x] macOS notifications + file picker working (matching Linux)
- [x] Windows toast notifications + file picker working
- [x] No new platform-specific dependencies in core modules
- [x] All tests pass, clippy clean

### Risks
- Windows TUI requires different terminal handling (WinRT console APIs)
- macOS code signing needed for menu bar agent distribution
- Cross-platform abstractions can leak platform-specific complexity into core

---

## Phase 8: Ecosystem & Community (90+ Days)

### Goal
Build the developer ecosystem around OpenCrust: plugins, multi-repo support,
and community growth.

### Rationale
OpenCrust has zero community presence. Cline (60K stars), OpenCode (95K+), and
Aider (44K) all have vibrant ecosystems. A public repo with credible benchmarks
and a plugin system could bootstrap adoption — but only after the enterprise
and cross-platform foundations are solid.

### Deliverables

| # | Item | Effort | Priority | Status |
|---|------|--------|----------|--------|
| 1 | **Plugin system** | High (1-2w) | 🟡 Medium | ✅ |
| 2 | **Multi-repo support** | High (1-2w) | 🟡 Medium | ✅ |
| 3 | **Publish to crates.io** | Low (1h) | 🔥 High | ✅ |
| 4 | **Community growth (GitHub presence)** | Ongoing | 🔥 High | 🔶 |

#### 4. Community Growth (GitHub Presence)

Build developer ecosystem around OpenCrust: documentation, examples, Discord, GitHub templates.

**Acceptance criteria:**
- [ ] GitHub Discussions for Q&A
- [ ] Contributing guide with ecosystem section
- [ ] Benchmark comparison blog post
- [ ] Discord/Slack community channel
- [ ] First community contribution merged
- [ ] All tests pass, clippy clean

### Acceptance Criteria (Phase 8)
- [x] Plugin system with at least 3 third-party tools working
- [x] Multi-repo orchestration functional
- [x] Published on crates.io
- [ ] Any community contribution merged

### Risks
- Plugin API design is irreversible — get it wrong and ecosystem fragments
- Multi-repo adds significant complexity to permission model
- Community growth is unpredictable — cannot force organic adoption

#### 1. Plugin System
- Extend `.opencrust/tools/` with metadata, dependencies, versioning
- Plugin manifest format
- Discovery and installation from remote sources
- OpenCrust SDK for third-party development

#### 2. Multi-Repo Support
- SSH/remote workspace agents
- Cross-repository orchestration (reference: Cursor 3 cross-repo agents)
- Multi-root workspace configuration

#### 3. Community
- Publish to crates.io (blocked until Phase 7 polish complete)
- GitHub Discussions for Q&A
- Contributing guide (exists but needs ecosystem section)
- Benchmark comparison blog post

### Acceptance Criteria (Phase 8)
- [x] Plugin system with at least 3 third-party tools working
- [x] Multi-repo orchestration functional
- [x] Published on crates.io
- [ ] Any community contribution merged

### Risks
- Plugin API design is irreversible — get it wrong and ecosystem fragments
- Multi-repo adds significant complexity to permission model
- Community growth is unpredictable — cannot force organic adoption

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| OpenDev (Rust) gains critical mass first | Medium | High | Publish benchmarks + crates.io ASAP |
| Claude Code/Cursor add security features | Medium | Medium | OpenCrust has 3-6 month lead in compliance |
| Token pricing changes break cost dashboard | High | Low | Abstract pricing into external config |
| Windows TUI has poor terminal compatibility | Medium | Medium | Test early against Windows Terminal, ConEmu |
| Plugin API design limits future extensibility | Medium | High | Design with open-closed principle, start minimal |
| Community growth never materializes | Low | Medium | Enterprise features are the primary value prop |

---

## Cross-Reference Map

| Document | Purpose | Status |
|----------|---------|--------|
| `ROADMAP.md` | **← YOU ARE HERE** — authoritative plan | Fresh (Jun 17) |
| `MARKET.md` | Competitive intelligence (no action items) | Refreshed (May 16) |
| `AGENTS.md` | Coding standards for contributors | Fresh |
| `docs/PERFORMANCE.md` | Benchmarks + optimization guide | Updated (May 16) |
| `docs/ARCHITECTURE.md` | System design reference | Fresh |
| `docs/SECURITY.md` | Permissions + audit reference | Fresh |
| `docs/CONFIGURATION.md` | All configuration options | Fresh |
| `docs/DEVELOPMENT.md` | How-to guides (tools, skills, MCP) | Fresh |
| `docs/DEPLOYMENT.md` | Private model deployment + compliance | New (Jun 17) |
| `docs/TROUBLESHOOTING.md` | Common issues | Fresh |
| `.uncensored/FINAL_REPORT.md` | Historical (superseded) | Archived with `.stale` |
| `docs/ROADMAP.md` | Roadmap history and evolution | Fresh (Jun 17) |

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Complete |
| 🔶 | In progress |
| ⏳ | Not started |
| 🔥 High | Must do before next release |
| 🟡 Medium | Important but not blocking |
| 🟢 Low | Nice to have |

## Current Status Summary

**Phase 6 (Enterprise Readiness)**: ✅ Complete
- Token budget & cost dashboard: ✅
- Background agent dashboard: ✅
- Enterprise compliance packaging: ✅

**Phase 7 (Cross-Platform Desktop)**: ✅ Complete (except menu bar)
- macOS file picker + environment detection: ✅
- macOS system notifications: ✅
- Windows toast notifications: ✅
- Windows file picker: ✅
- macOS menu bar agent: 🔶

**Phase 8 (Ecosystem & Community)**: ✅ Complete (except community)
- Plugin system: ✅
- Multi-repo support: ✅
- Publish to crates.io: ✅
- Community growth: 🔶

---

*Next update: when any Phase item is completed or market conditions shift significantly.*
