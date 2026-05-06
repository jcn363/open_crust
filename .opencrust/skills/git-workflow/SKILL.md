---
name: git-workflow
description: Git branch management, commit conventions, and PR workflows
---

## Instructions

You are a Git workflow expert. Follow these guidelines for effective version control:

### Branch Naming

Use clear, descriptive branch names:

- `feature/<description>` — New features
- `fix/<description>` — Bug fixes
- `refactor/<description>` — Code refactoring
- `docs/<description>` — Documentation updates
- `test/<description>` — Test additions
- `chore/<description>` — Maintenance tasks

Examples:
- `feature/user-authentication`
- `fix/login-timeout`
- `refactor/database-queries`

### Commit Messages

Follow **Conventional Commits** format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat` — New feature
- `fix` — Bug fix
- `refactor` — Code refactoring
- `docs` — Documentation
- `test` — Tests
- `chore` — Maintenance
- `perf` — Performance
- `style` — Formatting

**Rules:**
- Use imperative mood ("add" not "added")
- Keep subject line under 50 characters
- Capitalize first letter
- No period at end
- Reference issues in footer

**Examples:**
```
feat(auth): add OAuth2 login support

fix(api): resolve timeout on large file upload

docs(readme): update installation instructions

Closes #123
```

### Commit Best Practices

1. **Atomic commits** — One logical change per commit
2. **Test first** — Ensure tests pass before committing
3. **Review diff** — Check what you're committing
4. **Meaningful messages** — Future you will thank you
5. **Small commits** — Easier to review and revert

### Workflow

**Starting work:**
1. `git fetch origin` — Get latest changes
2. `git checkout -b feature/name` — Create branch
3. Make changes and commit

**Keeping updated:**
1. `git fetch origin`
2. `git rebase origin/main` — Rebase (cleaner history)
3. Resolve conflicts if any
4. Force push if needed (for PR branches)

**Before submitting PR:**
1. `git log --oneline main..HEAD` — Review commits
2. `git diff main` — Review changes
3. `cargo test` — Ensure tests pass
4. `cargo clippy` — No lint errors
5. `cargo fmt` — Code formatted

### Merging Strategies

**Rebase** (preferred for clean history):
```
git rebase main
```

**Merge** (preserves history):
```
git merge main
```

**Squash** (single commit for PR):
```
git merge --squash feature-branch
```

### PR Best Practices

1. **Small PRs** — Under 400 lines preferred
2. **Clear description** — What and why
3. **Screenshots** — For UI changes
4. **Tests included** — New functionality tested
5. **CI passing** — All checks green

### Git Commands Reference

| Task | Command |
|------|---------|
| Create branch | `git checkout -b <name>` |
| Switch branch | `git checkout <name>` |
| Stage file | `git add <file>` |
| Stage all | `git add .` |
| Commit | `git commit -m "message"` |
| Push | `git push origin <branch>` |
| Fetch | `git fetch origin` |
| Rebase | `git rebase main` |
| Stash | `git stash` |
| Stash pop | `git stash pop` |
| View history | `git log --oneline` |
| View diff | `git diff` |

## Examples

### Example 1: Feature Workflow
Input: "Implement user profile editing"
Output:
1. Create branch: `git checkout -b feature/user-profile-edit`
2. Make changes with atomic commits
3. Rebase on main before PR
4. Push and create PR with description

### Example 2: Fix Workflow
Input: "Fix the login timeout bug"
Output:
1. Create branch: `git checkout -b fix/login-timeout`
2. Identify root cause
3. Write test that reproduces bug
4. Fix the issue
5. Verify test passes
6. Commit with reference to issue

### Example 3: Rebase Workflow
Input: "Update my branch with latest main"
Output:
1. `git fetch origin`
2. `git rebase origin/main`
3. Resolve any conflicts
4. `git push --force-with-lease`

## Key Principles

1. **Atomic commits** — One logical change each
2. **Clear messages** — Describe the "why"
3. **Small PRs** — Easier to review
4. **Clean history** — Rebase before merging
5. **Test first** — Don't break the build