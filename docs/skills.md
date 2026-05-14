# OpenCrust Skills Guide

Skills provide specialized instructions and workflows for specific tasks. They teach OpenCrust how to behave — like following coding standards, using specific frameworks, or applying security best practices.

## Overview

- **Skills** are defined in `SKILL.md` files
- Stored in `.opencrust/skills/` directories (project-local or global)
- OpenCrust automatically discovers and loads skills at startup
- Skills are passed to the LLM as context for relevant tasks

## Discovery Paths

OpenCrust searches for skills in this order:

1. **Local project**: `./.opencrust/skills/`
2. **Local project**: `./.claude/skills/`
3. **Local project**: `./.agents/skills/`
4. **Global**: `~/.config/opencrust/skills/`
5. **Global**: `~/.claude/skills/`
6. **Global**: `~/.agents/skills/`

---

## Included Skills

### rust-expert (Priority: High)

Specialized Rust development assistant for cargo, crates, and best practices.

**When to use:**
- Cargo commands and build optimization
- Crate selection and comparison
- Rust best practices and patterns
- Unsafe code review
- Performance optimization

**Key behaviors:**
- Use `cargo-bloat`, `flamegraph`, `criterion` for optimization
- Search crates.io directly, prefer well-maintained (>1000 stars, recent commits)
- Verify safety invariants in unsafe code
- Run `cargo test --lib --doc` for comprehensive coverage

---

### security-auditor (Priority: High)

Security-focused code review and vulnerability detection.

**When to use:**
- Reviewing code for security vulnerabilities
- Checking dependency vulnerabilities
- Audit compliance verification
- Permission and access control review

**Key behaviors:**
- Leverage OpenCrust's `permissions.rs` and `audit.rs`
- Check for OWASP Top 10 vulnerabilities
- Verify input sanitization
- Review authentication/authorization patterns

---

### summarize (Priority: Medium)

Use when you need to summarize text, videos, or web pages. Supports summarizing URLs, local files, and YouTube links. ships with specialized skills: summarize (brew install summarize)

**When to use:**
- Summarizing web articles, documents, or YouTube videos
- Extracting key points from lengthy content
- Getting quick overviews of online resources

**Key behaviors:**
- Use `summarize` CLI tool with appropriate flags
- Support for URL, local file, and YouTube inputs
- Multiple AI provider support (OpenAI, Anthropic, xAI, Google)
- Configurable models and extraction options

---

### git-workflow (Priority: Medium)

Git branch management, commit conventions, and PR workflows.

**When to use:**
- Branch creation and management
- Commit message formatting
- Rebasing and merge strategies
- PR preparation and review

**Key behaviors:**
- Follow conventional commits format
- Use meaningful branch names (`feature/`, `fix/`, `refactor/`)
- Ensure clean commit history before PR
- Verify CI passes before merging

---

### code-refactorer (Priority: Medium)

Pattern-based code transformations and technical debt identification.

**When to use:**
- Large-scale refactoring
- Pattern application (builder, factory, etc.)
- Technical debt analysis
- Code smell detection

**Key behaviors:**
- Identify code smells (long methods, duplicate code, tight coupling)
- Apply appropriate design patterns
- Preserve functionality during refactoring
- Write tests before refactoring

---

### test-generator (Priority: Medium)

Unit test creation, coverage analysis, and property-based testing.

**When to use:**
- Writing new tests
- Improving test coverage
- Property-based testing
- Integration test setup

**Key behaviors:**
- Use `#[cfg(test)]` modules
- Prefer `proptest` or `quickcheck` for property-based tests
- Aim for meaningful assertions, not just `assert!(true)`
- Test edge cases and error conditions

---

### docs-generator (Priority: Medium)

README creation, API documentation, and CHANGELOG generation.

**When to use:**
- Creating project documentation
- Writing API docs
- Generating changelogs
- Documenting public APIs

**Key behaviors:**
- Follow standard Rust doc conventions (`///` for items)
- Include usage examples in docs
- Generate CHANGELOG from git history
- Keep README concise but complete

---

### sequentialthinking (Priority: Medium)

Specialized assistant for sequential thinking and step-by-step problem solving.

**When to use:**
- Complex problems requiring step-by-step reasoning
- Mathematical problem solving
- Algorithm design and implementation
- Debugging processes
- Decision making with multiple factors
- Planning multi-step tasks or projects

**Key behaviors:**
- Break down problems into smaller, manageable steps
- Ensure logical flow between steps
- State assumptions clearly at each step
- Validate intermediate results before proceeding
- Document reasoning for each step
- Use structured frameworks for problem solving
- Show work and explain the "why" behind actions

---

### criticalthinking (Priority: Medium)

Specialized assistant for critical thinking, analysis, and evaluation.

**When to use:**
- Evaluating claims, arguments, or proposals
- Analyzing complex issues or problems
- Making reasoned judgments or decisions
- Reviewing designs, implementations, or solutions
- Identifying biases, fallacies, or weaknesses in reasoning
- Synthesizing information from multiple sources

**Key behaviors:**
- Question assumptions and examine underlying premises
- Evaluate evidence for quality, relevance, and sufficiency
- Consider alternative explanations or solutions
- Identify cognitive biases, logical fallacies, and rhetorical devices
- Assess the validity and soundness of reasoning processes
- Synthesize information from multiple sources
- Weigh evidence objectively and acknowledge uncertainty
- State conclusions clearly with appropriate qualifications

---

### rust-best-practices (Priority: High)

Enforce Apollo GraphQL's Rust Best Practices Handbook for code review, linting, performance, error handling, testing, and type safety.

**When to use:**
- Reviewing Rust code for best practices compliance
- Checking borrowing, ownership, and cloning patterns
- Validating error handling (thiserror/anyhow, Result vs panic)
- Performance optimization (iterators, zero-cost abstractions, benchmarking)
- Testing strategy (naming, one assertion per test, snapshot testing)
- Type-state pattern application
- Clippy lint enforcement
- Documentation and comment standards

**Key behaviors:**
- Flag all `.unwrap()`/`.expect()` in production code
- Enforce `&T` over `.clone()`, `&str` over `String` in parameters
- Recommend iterator combinators over manual loops
- Suggest `thiserror` for libraries, `anyhow` for binaries
- Apply type-state pattern for stateful objects
- Require `///` doc comments for public APIs, `//` for *why* explanations

---

### rust-ai-ide-rules (Priority: Medium)

Global rules for Rust AI IDE development ensuring best practices in code structure, development processes, performance, and security.

**When to use:**
- Rust code organization and modular design
- Development practices and task decomposition
- Code and test separation strategies
- Performance optimization and user experience
- Code scanning and security vulnerability detection

**Key behaviors:**
- Prioritize modular design with single responsibility modules
- Use composition over inheritance and dependency injection
- Break tasks into sequential steps for progressive verification
- Implement incremental changes with targeted testing
- Strictly separate tests from production code
- Focus on load time reduction and code splitting
- Integrate Snyk for vulnerability scanning and remediation
- Follow modification sequence: dependencies → imports → types → quality → validation

---

### global-dev-rules (Priority: Medium)

Global development rules and best practices for maintaining high-quality, secure, and performant code across all projects.

**When to use:**
- Code and test separation strategies
- Code organization and modular design
- Development practices and task execution
- Performance optimization techniques
- Security scanning and code quality assurance

**Key behaviors:**
- Enforce absolute segregation between test and production code
- Modularize code with single responsibilities using traits for abstraction
- Decompose tasks incrementally with frequent validation
- Focus on perceived performance metrics (FCP, TTI)
- Mandate Snyk Code and SCA scanning for vulnerabilities
- Follow error resolution sequence: dependencies → imports → types → quality → validation
- Use selective imports, lazy loading, and asset optimization
- Remediate security issues promptly and re-scan after fixes

---

## Creating Custom Skills

### Skill File Structure

```markdown
---
name: my_skill
description: Does something useful
---

## Instructions

The agent should follow these steps to accomplish the task:

1. First step...
2. Second step...
3. Third step...

## Examples

### Example 1: Task Description
Input: "Do X"
Output: "Result Y"

### Example 2: Another Task
Input: "Do Z"
Output: "Result W"
```

### Skill Metadata

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique skill identifier (kebab-case) |
| `description` | Yes | Brief description of what the skill does |

### Best Practices

1. **Be specific** — Clear instructions, not vague guidelines
2. **Provide examples** — Show input/output pairs
3. **Focus on one domain** — Don't try to cover everything
4. **Use the agent's tools** — Reference OpenCrust's built-in capabilities

---

## Skill Loading

Skills are loaded at startup and made available to the LLM via the `<available_skills>` XML block. The LLM automatically selects relevant skills based on the task.

### Verification

To verify skills are loaded:
1. Start OpenCrust
2. Check startup logs for "Loaded skill: X"
3. Ask "What skills are available?" to see the list

---

## Troubleshooting

### Skill Not Found

1. Verify the `SKILL.md` file exists in a discovery path
2. Check the YAML frontmatter is valid
3. Ensure the file is named exactly `SKILL.md` (case-sensitive)

### Skill Not Applied
3. Ensure the file is named exactly `SKILL.md` (case-sensitive)

---

## Active/Inactive Toggle

Skills can be toggled active or inactive. Only active skills are sent to the LLM for context.

### Using the SkillBrowser UI (Ctrl+Shift+K)

1. Press `Ctrl+Shift+K` to open the SkillBrowser
2. Use `↑`/`↓` to navigate the skill list
3. Press `Enter` to toggle a skill's active status
4. Press `Esc` or `q` to close the browser

The SkillBrowser shows:
- Skill name and status (ACTIVE/INACTIVE)
- Description
- Usage statistics (count and average latency)

### Using CLI Commands

```bash
# List all skills with their status
opencrust skills list

# Activate a skill
opencrust skills activate rust-expert

# Deactivate a skill
opencrust skills deactivate security-auditor

# Show statistics for a specific skill
opencrust skills stats rust-expert

# Show statistics for all skills
opencrust skills stats
```

---

## Skill Usage Tracking

> **Note**: Usage tracking is planned but not yet integrated. When implemented, OpenCrust will track:
> - Number of times each skill is used
> - Average latency for skill execution
> - Statistics viewable via `opencrust skills stats`

---

*Last updated: 2026-05-14*