# Contributing to OpenCrust

Welcome! OpenCrust is built by the community. This guide will help you get started contributing.

## Prerequisites

- **Rust 1.75+** — Install from [rustup.rs](https://rustup.rs)
- **Git** — For version control and submitting PRs
- **Cargo** — Comes with Rust
- **Python 3.8+** — For pre-commit hooks
- **Time** — Start with 30 min for setup, 1-2 hours for a first contribution

## Local Development Setup

### 1. Clone the Repository

```bash
git clone https://github.com/opencrust/opencrust.git
cd opencrust
```

### 2. Build the Project

```bash
# Debug build (fast, unoptimized)
cargo build

# Release build (slow, optimized for production)
cargo build --release
```

If the build fails, check:
- Rust version: `rustc --version` (should be 1.75+)
- Dependencies installed: `cargo check` shows detailed errors
- See **docs/TROUBLESHOOTING.md** for common issues

### 3. Install Pre-commit Hooks

```bash
# Install pre-commit
pip install pre-commit

# Install the git hooks
pre-commit install

# Install commit-msg hook for DCO sign-off
pre-commit install --hook-type commit-msg
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run only lib tests (fast)
cargo test --lib
```

### 4. Format & Lint

```bash
# Check formatting
cargo fmt -- --check

# Format code
cargo fmt

# Lint with strict warnings (enforced)
cargo clippy -- -D warnings
```

All of these must pass before submitting a PR.

---

## Development Workflow

### 1. Create a Feature Branch

Follow the naming convention from **AGENTS.md**:

```bash
git checkout -b feature/my-feature
# or
git checkout -b fix/issue-123
# or
git checkout -b docs/improve-readme
```

### 2. Make Your Changes

- Edit files in `src/`
- Keep changes focused (one feature per PR)
- Reference **docs/MODULES.md** to understand which file to modify
- Reference **docs/DEVELOPMENT.md** for extension patterns

### 3. Test Your Changes

```bash
# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Build release (takes longer but catches more issues)
cargo build --release

# Build .deb package (requires cargo-deb)
cargo deb
```

If any of these fail, your PR will be rejected. Fix them locally first.

### 5. Commit Your Changes

Follow the phase-based convention from **AGENTS.md**:

```bash
# Feature work (within a phase)
git add src/my_file.rs
git commit -s -m "phase [N]: brief description of change"

# Refactoring or fixes
git commit -s -m "refactor: extract function for clarity"
git commit -s -m "fix: handle nil pointer in tool execution"
git commit -s -m "docs: expand CONFIGURATION.md with examples"

# Small docs fixes
git commit -s -m "docs: typo in README"
```

**Rules:**
- Imperative mood ("add feature" not "added feature")
- One logical concern per commit
- Reference issue numbers if applicable: "fix #123: ..."
- **All commits must be signed off with DCO** (`git commit -s`)
- Commit messages must follow conventional commit format: `<type>(<scope>): <description>`
- See past commits for examples: `git log --oneline | head -20`

### Commit Message Format

All commits must follow the conventional commit format:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks
- `build`: Build system changes
- `ci`: CI/CD changes
- `revert`: Reverting a previous commit

**Example:**
```
feat(config): add centralized configuration management

Implements VllmConfig pattern for unified configuration management.
Follows vLLM Phase 1 approach with single configuration object.

Signed-off-by: John Doe <john.doe@example.com>
```

### 5. Push and Create a Pull Request

```bash
git push origin feature/my-feature
```

Then open a PR on GitHub. Include:
- **What changed:** Brief summary (1-2 sentences)
- **Why it changed:** Motivation, context, or issue reference
- **Testing:** How you verified the change works
- **Checklist:** Did you run tests? Update docs?

---

## PR Submission Checklist

Before pushing your PR, verify:

- [ ] `cargo test` passes (all tests)
- [ ] `cargo fmt -- --check` passes (formatting correct)
- [ ] `cargo clippy -- -D warnings` passes (no warnings)
- [ ] New code includes unit tests (if non-trivial logic)
- [ ] Documentation updated (if user-facing change)
- [ ] Commit message follows convention
- [ ] All commits are signed off with DCO (`git commit -s`)
- [ ] Branch is up to date with main: `git pull origin main`
- [ ] No unintended changes included

### Example: Complete Workflow

```bash
# Start feature branch
git checkout -b feature/add-new-tool
cd /home/user/Desktop/opencrust

# Make changes
vim src/custom_tools.rs
vim docs/DEVELOPMENT.md

# Test thoroughly
cargo test
cargo fmt -- --check
cargo clippy -- -D warnings

# Commit (with DCO sign-off)
git add src/custom_tools.rs docs/DEVELOPMENT.md
git commit -s -m "feat(config): auto-discover tools from .opencrust/tools/ directory"

# Push
git push origin feature/add-new-tool

# Create PR on GitHub
# Include: what changed, why, how tested, checklist
```

---

## CI/CD Pipeline

When you submit a PR, GitHub Actions runs:

1. **Formatting check:** `cargo fmt -- --check`
2. **Clippy linting:** `cargo clippy -- -D warnings`
3. **Unit tests:** `cargo test --lib`
4. **Integration tests:** `cargo test --test *`
5. **Documentation build:** `cargo doc --no-deps`

All must pass. If any fail, click "Details" to see the error and fix locally.

---

## Getting Help

- **Questions about contributing?** Open a GitHub Discussion
- **Bug or feature request?** Open an Issue with details
- **Want to chat?** Join the Discord/community chat
- **Need code review help?** Ask in the PR comments

### Important Docs for Contributors

- **AGENTS.md** — Coding standards, naming conventions, error handling
- **docs/MODULES.md** — Where each part of the code lives
- **docs/DEVELOPMENT.md** — How to add features (tools, skills, commands, etc.)
- **docs/TROUBLESHOOTING.md** — Common issues and solutions

---

## Project Structure

OpenCrust is organized by concern. For detailed breakdown, see **docs/MODULES.md**.

**Quick reference:**

```
src/
├── main.rs              # Entry point & CLI
├── app.rs               # Application state
├── ui.rs                # Terminal UI (Ratatui)
├── llm.rs               # LLM client & tool loop
├── tools.rs             # Tool definitions
├── mcp.rs               # MCP server integration
├── skills.rs            # Skill system
├── config.rs            # Configuration
├── permissions.rs       # Security & access control
└── ...                  # 30+ more modules
```

When adding a feature:
1. Check **docs/MODULES.md** to find the right file
2. Check **docs/DEVELOPMENT.md** for the pattern
3. Add code following **AGENTS.md** conventions
4. Write tests
5. Update **docs/** if user-facing

---

## Common Mistakes to Avoid

1. **Large unfocused PRs** — Keep changes minimal and related
2. **No tests** — Add tests for non-trivial logic (see **docs/TESTING.md**)
3. **Ignoring linter warnings** — Run clippy and fix warnings before pushing
4. **Changing style while fixing bugs** — Keep style and logic changes separate
5. **Not updating docs** — If you change user-facing behavior, update **docs/**
6. **Forgetting to rebase** — Keep your branch up to date: `git pull origin main --rebase`

---

## Code Review Process

Maintainers will review your PR within 1-3 days. They may:

- **Request changes:** Address feedback and push new commits (don't force-push)
- **Ask questions:** Respond in PR comments
- **Approve:** Your PR gets squashed and merged!

### Tips for Fast Approval

- ✅ Focused PR (one feature/fix)
- ✅ Clear commit messages
- ✅ Tests passing
- ✅ Documentation updated
- ✅ Responsive to feedback

---

## Code of Conduct

- Be respectful and inclusive
- Give and receive feedback gracefully
- No discrimination or harassment
- Help others learn

---

## Recognition

Contributors are recognized in:

- **Git history** — Your commits live forever
- **CHANGELOG.md** — Major contributors listed per release
- **GitHub contributors page** — Automatic

Thank you for contributing to OpenCrust! 🦀
