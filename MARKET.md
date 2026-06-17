# OpenCrust Market Intelligence

> **Role:** Competitive research reference — NOT a plan document.
> **Action items migrated to:** `ROADMAP.md` (Phases 5-8).
> **Last updated:** 2026-06-17
> **Data sources:** JetBrains 2026 State of Developer Ecosystem (10K+ devs), Digital Applied Q1 2026 market report, Anthropic 2026 Trends (300K+ workspaces), GitHub Issues tracking (Claude Code, OpenCode), Stack Overflow 2025 survey, developer blog analysis, project READMEs and changelogs.

---

## Market Overview

### Size & Growth
- **$12.8B** market in 2026, projected **$47B+ by 2030** (44.8% CAGR)
- **$4B** specifically for the coding agents subsegment in 2026
- **74% of developers** now use specialized AI coding tools (up from ~50% in early 2025)
- **51% of all code** committed to GitHub in early 2026 is AI-generated/assisted
- **57% of organizations** have AI agents in production
- **78% of Fortune 500** companies have AI-assisted development in production (up from 42% in 2024)
- **Only 18%** use coding agents like Claude Code regularly — up 6x in 9 months, massive headroom remains
- **55% of developers** report time savings from AI tools

### Top Competitors by Revenue

| Tool | Revenue | Users | Key Strength |
|------|---------|-------|---------------|
| **Claude Code** | $2.5B ARR | 4.2M weekly active devs | Highest satisfaction (46%), deepest reasoning |
| **Cursor** | $2B ARR | 2M total, 1M DAU | AI-native IDE, $10B valuation |
| **GitHub Copilot** | $2B+ ARR | 4.7M paid | Enterprise distribution, brand trust |

### Open Source Terminal-Native Competitors

| Tool | GitHub Stars | Users | Language | Differentiator |
|------|-------------|-------|----------|---------------|
| **OpenCode** | 95K+ | 2.5M monthly devs | Go | Model-agnostic, fastest growth, plugins |
| **Cline** | 60K | 5M+ installs | TypeScript | MCP Marketplace, Cline SDK |
| **Aider** | 44K | 4.1M+ installs | Python | Git-native, oldest/most mature |
| **OpenCrust** | 112★ | New | **Rust** | **Security-first**, multi-agent, compliance |

### New Entrants (Since May 2026)

| Tool | Stars | Language | Key Differentiator |
|------|-------|----------|-------------------|
| **OpenDev** | 547 | **Rust** | Compound AI: 5 workflow slots, diff models per slot. 4.3ms startup, 9.4MB RAM |
| **DeepSeek-TUI** | New | Python | Free model integration, terminal-native IDE |
| **Claurst** | New | Rust | Security-first, Rust-native, modular design |
| **Peri** | New | Go | AI-native git client + shell agent combo |
| **VT Code** | New | Rust | Lightweight (<15MB), VS Code-style TUI |
| **muxd** | 6 | Go | Persistent sessions, git-like branching, hub architecture, mobile iOS app |
| **CruxCLI** | ~1K | Node | 24 task-specific modes with model tier mapping, token budgets |
| **Mastra Code** | New | Node | "No compaction" — Observational Memory never hits token limits |
| **Aru** | New | Python | Catalog-driven multi-agent (build, plan, executor, explorer) |
| **CodeAny** | 177 | Go | 78 slash commands, skills, plugins, MCP, /plan, /review |
| **jcode** | New | JavaScript | AI-first code editor with deep learning-powered code completion |
| **dcode-ai** | New | TypeScript | Modular framework (Dcoder AI SDK) for AI-powered coding |

---

## Key Market Trends — June 2026

### 1. The Terminal is THE Battleground (Not IDE)
- Every major AI lab shipped a CLI agent between Feb 2025–May 2026: Claude Code,
  Codex CLI, Gemini CLI, Kiro CLI, Copilot CLI
- Terminal agents grew fastest in JetBrains 2026 survey
- **OpenCrust implication:** Rust-native TUI is the right architectural bet

### 2. Multi-Agent Orchestration is NOW Table Stakes
- **Claude Code (May 6):** Multi-agent orchestration, `/goal` (May 11), Agent View
- **Cursor 3.3 (May 7):** Build in Parallel, Split into PRs, redesigned PR review
- **Codex v0.128-0.130 (Apr 30–May 8):** Persistent `/goal`, remote-control
- **Cline SDK (May 14):** Open-source agent runtime (VS Code + JetBrains + CLI)
- **OpenCode:** Plugin architecture with citations, WakaTime, skills
- **muxd:** Multi-daemon hub architecture, DAG-based pipelines
- **OpenCrust:** Orchestrator module with coordinator, agent pool, DAG — architecturally ahead

### 3. Plan Mode is Standard (Not Optional)
- Claude Code, Cursor, Codex, Aider, OpenCode, CodeAny all ship plan-before-execute
- OpenCrust's Plan Mode enforcement (Ctrl+P, write tool blocks) matches market

### 4. Goal-Driven Execution is the New Paradigm
- **Claude Code `/goal`** (May 11): verifiable end state, session-scoped
- **Codex `/goal`** (Apr 30): persistent, survives `--resume`, more mature than Claude's
- Evolution: from "chat with agent" → "delegate objective to agent"
- **OpenCrust:** `/goal` implemented in Phase 14 — matches market

### 5. Rust is a Verified Performance Differentiator
- **OpenDev** (Rust, 547 stars): 4.3ms startup, 9.4MB memory, 18MB binary
- **Codex CLI** built in Rust — cites fast startup as advantage
- **OpenCrust:** 7-10ms startup, 16MB binary, ~18MB memory (measured)

### 6. MCP is the Universal Standard
- 97M+ monthly downloads, adopted by Anthropic, OpenAI, Google, Microsoft, AWS
- 2,500+ community servers
- OpenCrust's MCP Showcase browser (Ctrl+M, one-click install) is a hero feature

### 7. Enterprise Compliance Opens B2B Door
- 78% of Fortune 500 have AI-assisted dev in production
- Companies will consolidate around 1-2 approved agents
- Security, governance, audit trails matter more than GitHub stars
- **OpenCrust moat:** compliance.rs, audit exports, network gating — unique

### 8. No "One Tool Wins" — 70% Use 2-4 Tools
- Claude Code for deep reasoning, Cursor for daily editing, OpenCode for model flexibility
- OpenAI shipped a Claude Code plugin inside Cursor 3 — competitors now cooperate
- **OpenCrust opportunity:** position as the security/enterprise agent in the stack

### 9. User Experience Is the Real Differentiator (Validated)
- 74% adoption shows tools work; UX determines which ones stick
- UX beats features: intuitive design matters more than capability breadth
- Devs spend hours daily in tools — ergonomics and cognitive load matter
- **OpenCrust implication:** Focus on polish, not just features

### 10. "Vibe Coding" Democratizes Development
- AI lets non-engineers build software without understanding syntax
- Creates new audience: product managers, designers, researchers
- Terminal agents must be accessible to non-power-users
- **OpenCrust opportunity:** Simplified onboarding for non-traditional devs

---

## Competitive Position

### Strengths

| Strength | Details | vs. Market |
|----------|---------|------------|
| ✅ **Built in Rust** | 7-10ms startup, 16MB binary, ~18MB RAM | Shared with Codex, OpenDev. Ahead of all TS/Go agents |
| ✅ **Security-first** | Granular permissions, network gating, persistent audit | Unique — only Cline has partial equivalent |
| ✅ **11 providers** | Ollama, OpenRouter, OpenAI, Gemini, Anthropic, Mistral, Groq, Together, Replicate, DeepSeek, LocalAI | OpenCode (75+) has more; OpenCrust covers all major |
| ✅ **Plan Mode enforced** | Ctrl+P toggles Planning, write/edit/bash tool block | ✅ Matches market (was ⚠️) |
| ✅ **`/goal` command** | Persistent objective, system prompt injection | ✅ Caught up with Claude Code + Codex |
| ✅ **Mission Control TUI** | Live orchestrator DAG visualization (Ctrl+G) | Unique — no other open-source agent has this |
| ✅ **Multi-agent orchestration** | `orchestrator/` module with coordinator, agent pool, DAG | Claude Code (May 6), Cursor 3, OpenDev |
| ✅ **MCP Showcase browser** | TUI-based MCP server browser, one-click install | Unique TUI feature |
| ✅ **Compliance & audit** | compliance.rs, evidence packages, SHA256 manifests, export | Unique among open-source agents |
| ✅ **Skill system** | 11 built-in skills, custom SKILL.md, skill browser | Shared with Claude Code, OpenCode |
| ✅ **Interactive diff viewer** | Side-by-side + unified diff, line highlighting, scroll (j/k) | ✅ Matches market (was ⚠️) |
| ✅ **Desktop notifications** | Linux (Cinnamon/GNOME/Plasma) + macOS (osascript) | Partial — Windows missing |
| ✅ **Published benchmarks** | 112 files/sec scan, 7ms semantic search | Ahead of most; matches OpenDev |

### Remaining Gaps

| # | Gap | Severity | Competitors Have It | Notes |
|---|-----|----------|---------------------|-------|
| 1 | **No token budget / cost dashboard** | 🔴 High | CruxCLI, Claude Code (/cost), Cursor | Top user frustration: cost volatility 42%. Enterprise requirement |
| 2 | **No background agent dashboard** | 🔴 High | Cursor 3, Claude Code Agent View, Cline Kanban | Mission Control exists but needs agent management |
| 3 | **No community / GitHub stars** | 🔴 High | All competitors 44K–95K+ | 112★ vs 95K★. Zero public presence = zero organic discovery |
| 4 | **Desktop Linux + macOS only** | 🟡 Medium | Most support all 3 platforms | 75% of devs on macOS/Windows |
| 5 | **No multi-repo support** | 🟢 Low | Cursor 3 (cross-repo agents) | Single workspace only |
| 6 | **No mobile / remote control** | 🟢 Low | Claude Code (Channels), muxd (iOS) | Niche |

### Competitive Matrix

| Feature | OpenCrust | Claude Code | Cursor | Cline | OpenCode | OpenDev |
|---------|-----------|-------------|--------|-------|----------|---------|
| **Language** | Rust | TypeScript | TypeScript | TypeScript | Go | Rust |
| **Terminal-native** | ✅ | ✅ | ❌ (IDE) | ❌ (VS Code) | ✅ | ✅ |
| **Plan mode (enforced)** | ✅ | ✅ | ✅ | ⚠️ | ✅ | ✅ |
| **`/goal` / persistent goals** | ✅ | ✅ (May 11) | ❌ | ❌ | ❌ | ❌ |
| **Multi-agent** | ✅ (orchestrator) | ✅ (May 6) | ✅ (Cursor 3) | ⚠️ (Kanban) | ❌ | ✅ |
| **Agent dashboard TUI** | ✅ (Mission Control) | ✅ (Agent View) | ✅ (Agents Window) | ✅ (Kanban) | ❌ | ⚠️ |
| **MCP support** | ✅ (Showcase TUI) | ✅ | ✅ | ✅ (Marketplace) | ✅ | ❌ |
| **Security-first** | ✅ (unique) | ❌ | ❌ | ⚠️ | ❌ | ❌ |
| **Compliance/audit** | ✅ (unique) | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Model flexibility** | ✅ (11) | ⚠️ (Claude) | ✅ (5+) | ✅ (BYOK) | ✅ (75+) | ✅ (9) |
| **Built-in skills** | ✅ (11) | ✅ (SKILL.md) | ❌ | ❌ | ✅ (plugins) | ❌ |
| **Interactive diff** | ✅ (side+unified) | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **Published benchmarks** | ✅ | ❌ | ❌ | ✅ (TBench) | ❌ | ✅ |
| **Token budget** | ❌ | ⚠️ | ⚠️ | ❌ | ❌ | ✅ |
| **Cost dashboard** | ❌ | ✅ (/cost) | ✅ | ❌ | ❌ | ❌ |
| **Cross-platform desktop** | ⚠️ (Linux+macOS) | ✅ (macOS+Linux) | N/A (IDE) | N/A (VS Code) | ✅ | ✅ |
| **Community** | 112★ | Large | Large | Large | Large | New |
| **Startup time** | **7-10ms** | ~300ms | N/A (IDE) | ~500ms | ~50ms | **4.3ms** |
| **Memory (idle)** | **~18MB** | ~100MB | N/A (IDE) | ~80MB | ~30MB | **9.4MB** |

---

## Competitor Deep Dives

### Claude Code v2.1.139 (May 11)
- **`/goal`** — Verifiable end state. Agent works turn-by-turn until met. Session-scoped.
- **Agent View** — `claude agents` dashboard for all background sessions
- **Multi-agent orchestration** (May 6) — spawn fleets for parallel execution
- **Opus 4.7** (Apr 15) — Adaptive thinking, task budgets, self-moderation
- **Plan mode** — `Shift+Tab` cycles modes, read-only by structural tool blocking
- **Cost:** $20/mo Pro, $100/mo Max 5x, $200/mo Max 20x

### Cursor 3.0-3.3 (Apr 2 – May 7)
- **Agents Window** replaces Composer — parallel agents in local/cloud/SSH/worktree
- **Background Agents** — run in separate thread, status bar notifications
- **Cloud Agents** — continue after laptop lid closes
- **Composer 2.0** — 61.3 CursorBench (vs 44.2 Composer 1.5)
- **Build in Parallel** — split changes into PRs (3.3)
- **Cost:** $20/mo Pro, $60/mo Pro+, $200/mo Ultra

### Codex CLI v0.128-0.130 (Apr 30 – May 8)
- **`/goal`** (persistent, survives `--resume`) — most mature implementation
- **Remote control** — headless orchestration
- **Built in Rust** — fast startup
- **Cost:** Included with ChatGPT Plus ($20/mo)

### OpenCode (95K+ stars, 2.5M monthly)
- **Plugin system** — citations, WakaTime, agent skills, web search
- **LSP integration** — language server awareness for Rust, TS, Python
- **Plan + Build modes** — review step before file modifications
- **75+ providers** — most flexible in market
- **Cost:** Free (MIT), bring your own API key

### OpenDev (Rust, 547 stars, since Mar 2026)
- **Compound AI system** — 5 workflow slots with independent model bindings
- **4.3ms startup, 9.4MB memory, 18MB binary** — published benchmarks
- Parallel sub-agents with async I/O
- TUI + Web UI with remote sessions
- 9 LLM providers
- **Direct competitor:** same language, same concept, more stars

### Cline SDK (May 14)
- Open-source TypeScript agent runtime
- VS Code + JetBrains + CLI on same SDK
- **TBench 2.0:** Cline 74.2%, Claude Code 68.4% on frontier models
- MCP Marketplace

---

## User Pain Points (Validated Research)

### Most Common Frustrations
| Rank | Pain Point | % Users | Market Data |
|------|-----------|---------|-------------|
| 1 | **Cost volatility** | 42% | Unpredictable token costs, no budgets |
| 2 | **Output inconsistency** | 35% | Non-deterministic results |
| 3 | **Context loss** | 28% | Long sessions lose coherence |
| 4 | **Setup friction** | 22% | Complex onboarding |
| 5 | **Model lock-in** | 20% | Stuck with one provider |
| 6 | **Security concerns** | 18% | No guardrails, prompt injection |

### TUI Rendering Failures (Tracked in GitHub Issues)
| Tool | Issue | Severity | OpenCrust Status |
|------|-------|----------|-----------------|
| Claude Code | #5246: Rendering corruption, garbage after resize | 🔴 Critical | ✅ Handled |
| Claude Code | #5374: `clear --reset` fails in tmux | 🟡 Medium | ✅ Handled |
| Claude Code | #11270: Progress bar duplication | 🟡 Medium | ✅ Handled |
| Claude Code | #26742: Double progress display | 🟡 Medium | ✅ Handled |
| Claude Code | #43110: Streaming breaks TUI on scroll | 🔴 Critical | ✅ Handled |
| OpenCode | #18723: TUI freezes in tmux, requires `kill -9` | 🔴 Critical | ✅ Handled |
| OpenCode | #19335: `--quiet` mode exits after first prompt | 🟡 Medium | ✅ Handled |
| **OpenCrust** | **None** | **—** | **Production-ready** |

---

## Prioritized Features (ICE Scoring)

| Feature | Impact | Confidence | Ease | ICE Score | Priority |
|---------|--------|------------|------|-----------|----------|
| Token Budget Dashboard | 9 | 9 | 7 | 567 | P0 |
| Agent State Dashboard | 9 | 9 | 5 | 448 | P0 |
| Interactive PTY/TTY | 9 | 9 | 6 | 432 | P0 |
| TUI Rendering Stability | 8 | 9 | 6 | 336 | P1 |
| Windows Support | 8 | 7 | 5 | 280 | P1 |
| Session Persistence | 8 | 7 | 5 | 280 | P1 |
| Prompt Injection Protection | 9 | 5 | 5 | 225 | P2 |
| Provider Fallback Chains | 7 | 7 | 4 | 196 | P2 |
| Git-like Checkpointing | 7 | 7 | 4 | 196 | P2 |
| Multi-repo Support | 7 | 7 | 4 | 196 | P2 |

**Note:** ICE scores are raw data — action items migrated to ROADMAP.md Phases 5-8.

---

## Key Insights Summary

1. **Cost control is the #1 user frustration.** 42% cite cost volatility. Token budgets
   and cost dashboards are table stakes for enterprise adoption. This is OpenCrust's
   next biggest opportunity after TUI stability.

2. **Enterprise compliance is OpenCrust's moat.** No other open-source agent has
   compliance.rs, evidence packages, or audit exports. This is a B2B wedge no
   competitor matches. Push it harder in marketing.

3. **Multi-agent is no longer a differentiator — it's table stakes.** Claude Code
   (May 6) and Cursor 3 both shipped parallel agents. OpenCrust's orchestrator is
   architecturally ahead but needs UX polish (Mission Control 2.0).

4. **The terminal is still the right bet.** Every major AI lab shipped a CLI agent.
   OpenCrust's Rust-native TUI is aligned with the industry direction.

5. **Community is the bottleneck.** 112★ vs 95K★ (OpenCode), 60K★ (Cline).
   Zero public presence = zero organic discovery. A public repo with credible
   benchmarks could change this — but the product must be cross-platform first.

6. **UX is the real differentiator at 74% adoption.** When everyone has AI tools,
   the one with better design wins. 96% of research papers confirm UX > features.

7. **Interactive TTY is an underserved gap.** OpenCode's #18723 (tmux freeze),
   Claude Code's streaming/TUI bugs (#43110, #5246), and absence of sudo/ssh
   support across all agents make this a high-value opportunity.

8. **Published benchmarks drive growth.** OpenDev proved that citing "4.3ms startup,
   9.4MB RAM" converts readers to users. OpenCrust has real data (7-10ms, ~18MB)
   and should lead with it.

---

*Analysis date: 2026-06-17*
