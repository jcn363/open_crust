# OpenCrust Improvement Plan

Based on analysis of DeepSpeed, OpenCode (anomalyco/opencode), and vLLM best practices.

## 1. Configuration Management (vLLM VllmConfig Pattern)

**Current State**: Config scattered across multiple files, no centralized configuration object.

**Target**: Single `VllmConfig`-style configuration object passed throughout the application.

**Changes**:
- Create `src/config/vllm_config.rs` with unified `AppConfig` struct
- All subsystems read from this central config
- Configuration validation at startup
- Support for config file (TOML/JSON) + environment variables + CLI args

## 2. Dependency Injection / Service Locator (vLLM Phase 1)

**Current State**: Managers created ad-hoc in `main.rs` and passed around.

**Target**: Central service registry (`src/core/services.rs`) with lazy initialization.

**Changes**:
- Create `ServiceRegistry` with `get_or_init<T>()`
- Register all managers (MCP, LSP, Skills, Tools, Plugins, Permissions, Audit)
- Replace direct `Arc<Mutex<T>>` passing with service lookups
- Enable easy testing with mock services

## 3. Enhanced CI/CD Pipeline (DeepSpeed Pattern)

**Current State**: Basic CI with Ubuntu/macOS, stable/beta Rust.

**Target**: Comprehensive pipeline matching DeepSpeed's multi-platform approach.

**Changes**:
- Add Windows testing
- Add Rust nightly testing
- Add scheduled nightly runs
- Add hardware-specific test categories (if applicable)
- Add code quality gates (fmt, clippy, audit, deny)
- Add release automation with version management
- Add PR size checks and conventional commit enforcement

## 4. Pre-commit Hooks & DCO Signing (DeepSpeed/vLLM)

**Current State**: No pre-commit, no DCO enforcement.

**Target**: Automated formatting, linting, and DCO signing on commit.

**Changes**:
- Add `.pre-commit-config.yaml` with rustfmt, clippy, cargo-check
- Add `commit-msg` hook for DCO sign-off verification
- Add conventional commit message validation
- Document in CONTRIBUTING.md

## 5. Testing Infrastructure (vLLM/DeepSpeed)

**Current State**: Inline unit tests only, no integration tests, basic benchmarks.

**Target**: Multi-layer testing strategy.

**Changes**:
- Create `tests/` directory for integration tests
- Add property-based testing with `proptest`
- Add snapshot testing with `insta`
- Enhance benchmarks with Criterion (statistically rigorous)
- Add test categories: unit, integration, e2e
- Add test coverage reporting

## 6. Documentation (OpenCode AGENTS.md + vLLM Architecture)

**Current State**: Basic README, AGENTS.md exists but could be enhanced.

**Target**: Comprehensive documentation for AI assistants and humans.

**Changes**:
- Enhance AGENTS.md with module map, conventions, workflows
- Add `docs/architecture/` with component diagrams
- Add `docs/contributing/` with detailed guides
- Add API documentation with examples
- Add RFC template for major changes

## 7. Provider-Based Architecture (OpenCode Pattern)

**Current State**: Hardcoded integrations for MCP, LSP, desktop.

**Target**: Extensible provider system for tools, desktop, notifications, file pickers.

**Changes**:
- Create `Provider` trait for each integration type
- Implement provider registry
- Allow dynamic provider registration
- Support for external plugin providers

## 8. Version Management (DeepSpeed Pattern)

**Current State**: Version hardcoded in Cargo.toml.

**Target**: Single source of truth (`version.txt`) with automated bumping.

**Changes**:
- Add `version.txt` at repo root
- Update Cargo.toml to read from version.txt via build script
- Add release script with validation
- Automate patch version bump post-release

## 9. Multi-Agent Orchestration Enhancement (OpenCode Pattern)

**Current State**: Basic multi-agent support in `startup.rs`.

**Target**: Persistent subagent context, smart context management, self-healing.

**Changes**:
- Implement subagent session persistence
- Add priority-based context sliding window
- Add supervisor feedback loop for error recovery
- Add ACP (Agent Client Protocol) support

## 10. Code Quality & Anti-Patterns

**Current State**: Some `unwrap()` usage, dead code warnings, inconsistent patterns.

**Target**: Zero warnings, idiomatic Rust, consistent patterns.

**Changes**:
- Fix all clippy warnings
- Replace `unwrap()`/`expect()` with proper error handling
- Remove dead code
- Standardize error types with `thiserror`
- Add `#[expect]` with justification where needed

---

## Implementation Priority

### Phase 1: Foundation (Week 1)
1. Centralized configuration (VllmConfig)
2. Service Locator / DI pattern
3. Version management system
4. Pre-commit hooks & DCO

### Phase 2: CI/CD & Testing (Week 2)
1. Enhanced CI/CD pipeline
2. Testing infrastructure
3. Benchmark improvements

### Phase 3: Architecture & Extensibility (Week 3)
1. Provider-based architecture
2. Multi-agent orchestration enhancements
3. Documentation overhaul

### Phase 4: Polish (Week 4)
1. Code quality fixes
2. Anti-pattern removal
3. Final verification and release prep