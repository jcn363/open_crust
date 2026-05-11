# Practical Examples & Walkthroughs

Copy-paste-ready tutorials for common tasks. Each example is self-contained.

**For reference docs:** See **docs/CONFIGURATION.md** and **docs/DEVELOPMENT.md**.  
**For deep context:** See **docs/ARCHITECTURE.md**.

---

## Example 1: Create Your First Custom Tool

**Goal:** Write a linter script that OpenCrust can use.

### Step 1: Create the script

```bash
mkdir -p ~/.opencrust/tools
cat > ~/.opencrust/tools/check_code << 'EOF'
#!/bin/bash
# name: check_code
# description: Run clippy and format check on Rust code

set -e

if [ $# -eq 0 ]; then
  echo "Usage: check_code <file_or_directory>" >&2
  exit 1
fi

TARGET="$1"

echo "=== Checking formatting ==="
cargo fmt -- --check "$TARGET" 2>&1 || {
  echo "❌ Format check failed"
  exit 1
}

echo "=== Running clippy ==="
cargo clippy -- -D warnings 2>&1 || {
  echo "❌ Clippy check failed"
  exit 1
}

echo "✅ All checks passed!"
EOF
chmod +x ~/.opencrust/tools/check_code
```

### Step 2: Test manually

```bash
# Verify it works
~/.opencrust/tools/check_code src/main.rs

# Should output:
# === Checking formatting ===
# ✅ All checks passed!
```

### Step 3: Use in OpenCrust

```bash
# Start OpenCrust
opencrust

# In chat, type:
# "Use check_code to lint src/main.rs"

# OpenCrust will:
# 1. Find check_code in .opencrust/tools/
# 2. Check permission
# 3. Execute it
# 4. Return output to you
```

### Step 4: Have LLM use it automatically

```
User: "Review my code for style issues"

OpenCrust/LLM will:
1. Suggest running check_code
2. Execute it
3. Show results
4. Suggest fixes based on clippy output
```

---

## Example 2: Set Up Ollama for Local LLM

**Goal:** Run an open-source LLM locally (no API key needed, private).

### Step 1: Install Ollama

**macOS:**
```bash
brew install ollama
```

**Linux (Ubuntu/Debian):**
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

**Windows:**
- Download from [ollama.ai](https://ollama.ai/download)
- Run installer

### Step 2: Start Ollama

```bash
# Start in background
ollama serve &

# Should print:
# 2026-05-08T14:00:00Z info server.go:... listening on 127.0.0.1:11434
```

### Step 3: Download a model

```bash
# Download Mistral 7B (small, fast, good quality)
ollama pull mistral

# Download takes 5–10 minutes first time
# Shows progress: [=======>    ] 65%

# After done, you can now use it
```

### Step 4: Configure OpenCrust

```bash
cat >> ~/.config/opencrust/config.json << 'EOF'
{
  "default_provider": "ollama",
  "default_model": "mistral",
  "providers": {
    "ollama": {
      "base_url": "http://localhost:11434"
    }
  }
}
EOF
```

### Step 5: Test it

```bash
# Start OpenCrust
opencrust

# In chat, type: "Hello, what's your name?"

# Should respond within 1–5 seconds (depending on hardware)
# Output: "I'm Mistral, an open-source language model..."
```

### Available Models

```bash
# Lightweight (good for laptops)
ollama pull neural-chat    # 4.7B, fast

# Balanced (recommended)
ollama pull mistral        # 7B, good quality + speed

# Large (need GPU)
ollama pull neural-chat:13b  # 13B, better quality
ollama pull openhermes      # 34B, very capable

# List available locally
ollama list
```

### Troubleshooting

**"Connection refused"**
```bash
# Ollama not running, start it:
ollama serve &
```

**"Model not found"**
```bash
# Pull it first:
ollama pull mistral
```

**"Out of memory"**
```bash
# Your system doesn't have enough VRAM
# Solution: Use a smaller model
ollama pull neural-chat  # 4.7B instead of 7B
```

---

## Example 3: Add GitHub Integration via MCP

**Goal:** Let OpenCrust search GitHub, read repos, and create PRs.

### Step 1: Create GitHub Token

1. Go to [github.com/settings/tokens](https://github.com/settings/tokens)
2. Click "Generate new token" → "Personal access tokens"
3. Select scopes: `repo`, `read:org`, `user`
4. Generate and copy token
5. **Keep this secret!**

### Step 2: Add to Config

```bash
cat >> ~/.config/opencrust/config.json << 'EOF'
{
  "mcp": {
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "ghp_YOUR_TOKEN_HERE"
      },
      "enabled": true
    }
  }
}
EOF

# Make config private
chmod 600 ~/.config/opencrust/config.json
```

### Step 3: Test It

```bash
# Restart OpenCrust
opencrust

# In chat, type:
# "Search GitHub for open issues in torvalds/linux with label 'bug'"

# OpenCrust will:
# 1. Call GitHub MCP server
# 2. Execute search
# 3. Return top 10 results
```

### Example Queries

```
"Show me open PRs in my organization"
"Create an issue titled 'Fix crash on startup' in my repo"
"Search GitHub for 'Rust benchmarking tools'"
"List recent commits to my main branch"
```

### Rotating Token (If Compromised)

```bash
# 1. Revoke old token
# GitHub → Settings → Developer settings → Personal access tokens → Delete

# 2. Create new token (same process)

# 3. Update config
sed -i 's/ghp_OLD_TOKEN/ghp_NEW_TOKEN/' ~/.config/opencrust/config.json

# 4. Restart OpenCrust
```

---

## Example 4: Debug a Failing Tool

**Goal:** Understand why a custom tool isn't working.

### Scenario

You created `~/.opencrust/tools/deploy`:

```bash
#!/bin/bash
# name: deploy
# description: Deploy to production

# (some deployment logic)
```

But it's failing and you don't know why.

### Step 1: Test Tool Manually

```bash
# Run the tool directly
./.opencrust/tools/deploy prod

# Note any error messages
# If it works here but fails in OpenCrust, see step 2
```

### Step 2: Check OpenCrust Logs

```bash
# Look at audit log
tail -20 ~/.config/opencrust/audit.json

# Find your tool execution
grep '"tool_name":"deploy"' ~/.config/opencrust/audit.json | tail -1

# Example output:
{
  "timestamp": "2026-05-08T14:30:00Z",
  "event_type": "tool_executed",
  "tool_name": "deploy",
  "result": "permission_denied",
  "reason": "Path 'prod' matches deny_patterns: '*.prod.*'"
}
```

### Step 3: Fix Common Issues

**Permission Denied:**
```json
{
  "permissions": {
    "file_patterns": ["src/**", "infra/**"],
    "deny_patterns": []  // Remove the deny pattern
  }
}
```

**Tool Not Found:**
```bash
# Check it exists and is executable
ls -la ~/.opencrust/tools/deploy

# Should show: -rwxr-xr-x (executable bit)
# If not: chmod +x ~/.opencrust/tools/deploy
```

**Tool Times Out:**
```json
{
  "timeout_seconds": 60  // Increase timeout
}
```

**Tool Needs Environment Variable:**
```bash
# Check what env vars the tool needs
grep -E 'export|ENV' ~/.opencrust/tools/deploy

# Add to config:
{
  "tools": {
    "deploy": {
      "env": {
        "DEPLOY_KEY": "...",
        "ENVIRONMENT": "prod"
      }
    }
  }
}
```

### Step 4: Enable Verbose Logging

```bash
# Run with debug output
RUST_LOG=debug opencrust -p "Use deploy prod"

# Will show:
# [DEBUG] Executing tool: deploy
# [DEBUG] Tool args: ["prod"]
# [DEBUG] Tool output: (stdout)
# [DEBUG] Tool stderr: (stderr)
```

---

## Example 5: Create a Custom Skill

**Goal:** Add project-specific guidance that influences LLM behavior.

### Scenario

Your project uses specific patterns you want the LLM to follow.

### Step 1: Create Skill File

```bash
mkdir -p ~/.opencrust/skills/my_project_rules
cat > ~/.opencrust/skills/my_project_rules/SKILL.md << 'EOF'
---
name: my_project_rules
description: Follow My Project's coding standards and patterns
priority: high
---

## Coding Standards

When writing code for this project, follow these rules:

### Error Handling

Always use explicit error types:
```rust
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
  // Use ConfigError, not generic Error
}
```

### Testing

Every function must have a unit test:
```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_load_config() {
    // Test implementation
  }
}
```

### Comments

- No comments above obvious code
- Comments only for "why", not "what"
- Use rustdoc for public APIs

## Project Patterns

- Use Result<T> for fallible operations
- No unwrap() in production code
- Prefer pattern matching over if/else
- Follow RAII (Resource Acquisition Is Initialization)

## Common Tasks

When asked to [add a feature], follow this checklist:
1. Find similar features in the codebase
2. Follow existing patterns
3. Add tests for the new code
4. Run `cargo test` to verify
5. Run `cargo clippy` to check style
6. Submit only passing changes

EOF
```

### Step 2: Verify Skill is Loaded

```bash
# Start OpenCrust
opencrust

# Check skill browser (Ctrl+Shift+K)
# Should see "my_project_rules" in list

# If not listed, check:
# - File at: ~/.opencrust/skills/my_project_rules/SKILL.md
# - YAML frontmatter valid
# - Not in deny list in config
```

### Step 3: Use It

```
User: "Add a new function to parse config files"

LLM will now:
1. Search for similar config parsing in codebase
2. Follow your error handling patterns
3. Include unit tests automatically
4. Suggest using Result<T> instead of unwrap()
```

### Making It Work Better

**Add code examples to your skill:**

```markdown
## Example: Config Parsing

```rust
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
  let content = std::fs::read_to_string(path)?;
  let config = serde_json::from_str(&content)?;
  Ok(config)
}
```
```

---

## Example 6: Multi-Model Workflow

**Goal:** Use different models for different tasks (fast model for quick answers, powerful model for complex work).

### Configuration

```bash
cat > ~/.config/opencrust/config.json << 'EOF'
{
  "default_model": "gpt-3.5-turbo",
  "default_provider": "openai",
  "providers": {
    "openai": {
      "api_key": "sk-..."
    },
    "anthropic": {
      "api_key": "sk-ant-..."
    },
    "ollama": {
      "base_url": "http://localhost:11434"
    }
  }
}
EOF
```

### Usage

```
# Quick question (fast model)
User: "What's the capital of France?"
OpenCrust: Uses gpt-3.5-turbo (fast)
Response: "Paris" (instant)

# Complex problem (switch model)
User: "Switch to gpt-4"
User: "Design an authentication system for a SaaS app"
OpenCrust: Uses gpt-4 (powerful)
Response: [detailed architecture with diagram]

# Local and private
User: "Switch to ollama"
User: "Analyze this sensitive data"
OpenCrust: Uses Mistral locally (no API calls)
Response: [fast, private]
```

### Implementing Locally

```bash
# Have multiple models ready
ollama pull mistral       # Fast
ollama pull llama2        # Balanced
ollama pull neural-chat:13b  # Capable

# In chat:
# "I'll use mistral for quick answers, switch to llama2 when I need better quality"
```

---

## Example 7: Customize Keybindings

**Goal:** Change keyboard shortcuts to match your workflow.

### Default Keybindings

| Action | Key |
|--------|-----|
| Submit message | Enter |
| New line | Shift+Enter |
| Undo | Ctrl+Z |
| Redo | Ctrl+Shift+Z |
| Format | Ctrl+F |
| Open Plan tab | P |
| Show Skills | Ctrl+Shift+K |
| Show File Tree | Ctrl+B |

### Customize

```bash
cat >> ~/.config/opencrust/config.json << 'EOF'
{
  "keybindings": {
    "submit_message": "Ctrl+Enter",      // Changed from Enter
    "new_line": "Enter",                 // Changed from Shift+Enter
    "format": "Ctrl+Shift+F",            // Changed from Ctrl+F
    "show_plan": "P",                    // Already P
    "run_tests": "Ctrl+T"                // New custom binding
  }
}
EOF
```

### Test It

```bash
# Start OpenCrust
opencrust

# Now:
# - Enter creates new line
# - Ctrl+Enter submits (instead of Enter)
# - Ctrl+Shift+F formats
```

---

## Example 8: Permission Lockdown (Production)

**Goal:** Maximum security for sensitive projects.

### Ultra-Restrictive Config

```json
{
  "permissions": {
    "file_patterns": [
      "src/**",
      "tests/**"
    ],
    "deny_patterns": [
      "infra/**",
      ".env*",
      "credentials/*",
      "*.prod.*",
      ".git/**"
    ],
    "cli_commands": {
      "allowed": [
        "cargo test",
        "cargo build",
        "git status",
        "git log"
      ],
      "deny": [
        "rm", "sudo", "git push", "git delete-branch",
        "docker", "systemctl", "aws", "terraform"
      ]
    },
    "network": {
      "enabled": true,
      "hosts": ["api.openai.com", "github.com"],
      "deny_hosts": ["internal.company.com", "prod-db.internal"]
    }
  },
  "audit": {
    "enabled": true,
    "retention_days": 365,
    "export_url": "https://audit.company.com/upload"
  }
}
```

### What This Allows

✅ Read/write source code
✅ Run tests
✅ Check git status
✅ Call OpenAI API
✅ Access public GitHub
✅ Use public MCP servers

### What This Blocks

❌ Delete files (no `rm`)
❌ Modify infrastructure
❌ Push to git
❌ Access production databases
❌ Run arbitrary commands via sudo
❌ Access private networks

---

## Troubleshooting Examples

### "Tool execution failed"

```bash
# Check audit log
grep '"result":"' ~/.config/opencrust/audit.json | tail -1

# Result: "result": "permission_denied"
# Solution: Check file_patterns in config.json

# Result: "result": "timeout"
# Solution: Increase timeout_seconds in config
```

### "LLM not responding"

```bash
# Test API directly
curl -X POST https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_KEY" \
  -d '{"model":"gpt-4","messages":[...]}'

# If timeout: Network issue or API down
# Check status page: https://status.openai.com
```

### "Can't find my_tool"

```bash
# Verify tool exists and is executable
ls -la ~/.opencrust/tools/my_tool

# Fix: chmod +x ~/.opencrust/tools/my_tool

# Verify it works
~/.opencrust/tools/my_tool test_arg

# Restart OpenCrust (caches tool list on startup)
```

---

## References

- **docs/DEVELOPMENT.md** — Detailed how-to guides
- **docs/CONFIGURATION.md** — All configuration options
- **CONTRIBUTING.md** — Contribution workflow
- **docs/MODULES.md** — Find code to extend
