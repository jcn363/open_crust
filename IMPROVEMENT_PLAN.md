# OpenCrust Improvement Plan

Based on analysis of DeepSpeed, OpenCode (anomalyco/opencode), and vLLM best practices.

## 1. Configuration Management ✅ (Removed)

**Status:** VllmConfig centralized configuration was implemented and later removed. Configuration is now handled by a single `Config` struct in `src/config.rs` with provider configs, model aliases, and subagent configuration, loaded at startup and validated. No further work planned.

## 2. Dependency Injection / Service Locator ❌ (Removed)

**Status:** `ServiceRegistry` was implemented and later removed. Managers are passed directly via `Arc<Mutex<T>>` in `main.rs`, which is sufficient for the current architecture.

## 3. Enhanced CI/CD Pipeline ✅ (Implementing)

**Current State:** Basic CI with Ubuntu/macOS, stable/beta Rust. Pre-commit hooks, DCO signing, and conventional commits enforced.

**Target:** Comprehensive pipeline matching DeepSpeed's multi-platform approach.

**Changes:**
- ✅ Basic CI with fmt, clippy, test on Ubuntu/macOS/Windows
- ✅ Rust nightly testing (Ubuntu only)
- ✅ Pre-commit hooks with rustfmt, clippy, trailing-whitespace, EOF fixer, DCO
- ✅ Conventional commit validation
- ❌ Scheduled nightly runs (not configured)
- ❌ Release automation with version management
- ❌ PR size checks

## 4. Pre-commit Hooks & DCO Signing ✅

**Status:** Fully implemented. `.pre-commit-config.yaml` with rustfmt, clippy, trailing-whitespace, end-of-file-fixer, YAML/TOML/JSON validation, merge conflict check, DCO sign-off, and commit-msg hook for conventional commit format.

## 5. Testing Infrastructure ◐ (Partial)

**Current State:** Inline unit tests (416 passing). No integration tests, basic benchmarks.

**Target:** Multi-layer testing strategy.

**Changes:**
- ✅ Unit tests in `#[cfg(test)]` modules (416 tests, 0 failures)
- ✅ Criterion benchmarks
- ❌ `tests/` integration tests directory
- ❌ Property-based testing with `proptest`
- ❌ Snapshot testing with `insta`
- ❌ Test coverage reporting

## 6. Documentation ✅ (Good)

**Current State:** AGENTS.md, README.md, docs/ with MODULES.md, ARCHITECTURE.md, DEVELOPMENT.md, TESTING.md, SECURITY.md, CONFIGURATION.md, etc.

**Target:** Comprehensive documentation for AI assistants and humans.

**Changes:**
- ✅ AGENTS.md with module map, conventions, workflows
- ✅ docs/ARCHITECTURE.md with component layout
- ✅ docs/MODULES.md with detailed module descriptions
- ✅ docs/CONFIGURATION.md with config reference
- ✅ docs/DEVELOPMENT.md with extension patterns
- ❌ RFC template for major changes

## 7. Provider-Based Architecture ◐ (Partial)

**Current State:** Desktop integration uses trait-based providers (detection, notifications, file_picker). Other integrations are hardcoded.

**Target:** Extensible provider system for tools, desktop, notifications, file pickers.

**Changes:**
- ✅ Desktop environment detection with `DesktopEnvironment` enum
- ✅ System notifications via `notify-send` / `cinnamon-sendto`
- ✅ Native file picker via `zenity` / `cinnamon-file-dialog`
- ❌ Generic `Provider` trait for each integration type
- ❌ External plugin providers

## 8. Version Management ◐ (Partial)

**Status:** Version in `Cargo.toml` and `version.txt`. Build script reads from version.txt. Release script exists. Post-release auto-bump not configured.

## 9. Multi-Agent Orchestration Enhancement ✅ (Implemented)

**Status:** Subagent system in `src/orchestrator/` with task management, session persistence, context management, ACP support, and supervisor feedback loop.

## 10. Code Quality & Anti-Patterns ◐ (Partial)

**Current State:** Zero warnings, zero clippy errors, 0 TODOs, 416 tests passing. Some `#[allow(dead_code)]` on test-only dead code.

**Changes:**
- ✅ All clippy warnings fixed (zero tolerance, `#![deny(warnings)]`)
- ✅ Dead code removed (VllmConfig 869 lines, ServiceRegistry 300+ lines)
- ✅ Standardized error types with `thiserror` and `anyhow`
- ◐ `unwrap()`/`expect()` usage — minimal, mostly in tests
- ◐ `#[allow(dead_code)]` on some fields/methods used in tests but dead in production

---

## Implementation Priority

### Phase 1: Foundation ✅
1. ~~Centralized configuration (VllmConfig)~~ → Removed, using simple Config
2. ~~Service Locator / DI pattern~~ → Removed, direct passing sufficient
3. Version management system → Partial
4. Pre-commit hooks & DCO → ✅ Complete

### Phase 2: CI/CD & Testing ◐
1. Enhanced CI/CD pipeline → Partial
2. Testing infrastructure → Partial
3. Benchmark improvements → Basic criterion setup done

### Phase 3: Architecture & Extensibility ◐
1. Provider-based architecture → Partial (desktop only)
2. Multi-agent orchestration enhancements → ✅ Complete
3. Documentation overhaul → ✅ Complete

### Phase 4: Polish ◐
1. Code quality fixes → ✅ Nearly complete
2. Anti-pattern removal → ✅ Done (main modules cleaned)
3. Final verification and release prep → Pending
