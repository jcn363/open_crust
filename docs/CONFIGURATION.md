# Configuration Reference

Complete guide to configuring OpenCrust: all options, all LLM providers, MCP servers, LSP servers, keybindings, and TUI customization.

**Location:** `~/.config/opencrust/config.json`  
**For security:** See **docs/SECURITY.md**.  
**For extending:** See **docs/DEVELOPMENT.md**.  
**For troubleshooting:** See **docs/TROUBLESHOOTING.md**.

---

## Quick Start: Default Configuration

OpenCrust works with no config file. Create one to customize:

```bash
mkdir -p ~/.config/opencrust
cat > ~/.config/opencrust/config.json << 'EOF'
{
  "default_model": "gpt-4",
  "default_provider": "openai",
  "providers": {
    "openai": {
      "api_key": "sk-YOUR_KEY_HERE"
    }
  }
}
EOF
chmod 600 ~/.config/opencrust/config.json
```

---

## Configuration Hierarchy

Options are resolved in this order (first match wins):

1. **CLI flags** — `opencrust --model gpt-4-turbo`
2. **Environment variables** — `OPENCRUST_MODEL=gpt-4-turbo`
3. **config.json** — JSON file contents
4. **Defaults** — Built-in hardcoded values

**Example:**

```bash
# Built-in default: gpt-3.5-turbo
# Set in config: "default_model": "gpt-4"
# Set environment: export OPENCRUST_MODEL=claude-3-opus
# CLI flag: opencrust --model gemini-pro

# Result: Uses gemini-pro (CLI wins)
```

---

## Global Settings

### Root Level

```json
{
  "default_model": "gpt-4",                    // Default LLM model
  "default_provider": "openai",                // Default provider
  "context_budget": 8000,                      // Max tokens for context
  "max_retries": 3,                            // Tool retry attempts
  "timeout_seconds": 30,                       // Default timeout
  "auto_save_session": true,                   // Save session on exit
  "offline_mode": false,                       // Disable network
  "theme": "dark"                              // TUI theme (dark/light)
}
```

### Explanation

| Setting | Type | Default | Notes |
|---------|------|---------|-------|
| `default_model` | string | `"gpt-4"` | Model to use if not specified |
| `default_provider` | string | `"openai"` | Provider to use if not specified |
| `context_budget` | number | `8000` | Max tokens to include in context (reduce for smaller models) |
| `max_retries` | number | `3` | How many times to retry failed tools |
| `timeout_seconds` | number | `30` | How long to wait for LLM/tools before timeout |
| `auto_save_session` | boolean | `true` | Automatically save session on exit |
| `offline_mode` | boolean | `false` | If true, disable all network access |
| `theme` | string | `"dark"` | TUI color scheme |

---

## LLM Providers

### Configuration Structure

```json
{
  "providers": {
    "provider_name": {
      "api_key": "...",        // or env var
      "base_url": "...",       // (optional)
      "organization": "...",   // (optional)
      "models": ["model1", "model2"]
    }
  }
}
```

### 1. OpenAI

```json
{
  "providers": {
    "openai": {
      "api_key": "sk-YOUR_KEY_HERE",
      "organization": "org-xyz"  // Optional
    }
  }
}
```

**Available models:**
- `gpt-4-turbo` (recommended)
- `gpt-4`
- `gpt-3.5-turbo` (fast, cheap)
- `gpt-4-vision` (with image understanding)

**Setup:**
1. Get API key from [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
2. Paste into config

### 2. Anthropic (Claude)

```json
{
  "providers": {
    "anthropic": {
      "api_key": "sk-ant-YOUR_KEY_HERE"
    }
  }
}
```

**Available models:**
- `claude-3-opus-20240229` (most capable)
- `claude-3-sonnet-20240229` (balanced)
- `claude-3-haiku-20240307` (fast, cheap)

**Setup:**
1. Get API key from [console.anthropic.com](https://console.anthropic.com/account/keys)
2. Paste into config

### 3. Google Gemini

```json
{
  "providers": {
    "gemini": {
      "api_key": "YOUR_GEMINI_API_KEY"
    }
  }
}
```

**Available models:**
- `gemini-pro` (text only)
- `gemini-pro-vision` (with images)
- `gemini-1.5-pro` (latest)

**Setup:**
1. Get API key from [Google AI Studio](https://makersuite.google.com/app/apikey)
2. Enable Gemini API in Google Cloud
3. Paste into config

### 4. Ollama (Local)

```json
{
  "providers": {
    "ollama": {
      "base_url": "http://localhost:11434"
    }
  }
}
```

**Available models (install with `ollama pull`):**
- `mistral` (7B, fast, good reasoning)
- `neural-chat` (7B, instruction-tuned)
- `llama2` (7B, versatile)
- `openhermes` (34B, very capable)

**Setup (macOS/Linux):**

```bash
# 1. Install Ollama
#    macOS: brew install ollama
#    Linux: Visit ollama.ai

# 2. Start Ollama in background
ollama serve &

# 3. Pull a model
ollama pull mistral

# 4. Configure OpenCrust (already defaults to localhost:11434)
# Test:
curl http://localhost:11434/api/tags
```

**Setup (Windows):**
- Download from [ollama.ai](https://ollama.ai)
- Install and run
- Models stored in `%USERPROFILE%\.ollama\models`

### 5. OpenRouter

```json
{
  "providers": {
    "openrouter": {
      "api_key": "sk-or-YOUR_KEY_HERE"
    }
  }
}
```

**Available models (300+):**
- `anthropic/claude-3-opus` (Claude via OpenRouter)
- `openai/gpt-4-turbo` (GPT-4 via OpenRouter)
- `meta-llama/llama-2-70b-chat` (Llama 2)
- `mistralai/mistral-large` (Mistral)
- See [openrouter.ai/models](https://openrouter.ai/models) for full list

**Setup:**
1. Get API key from [openrouter.ai/keys](https://openrouter.ai/keys)
2. Paste into config
3. Benefit: Access to many models through one API

### 6. Mistral AI

```json
{
  "providers": {
    "mistral": {
      "api_key": "YOUR_MISTRAL_API_KEY"
    }
  }
}
```

**Available models:**
- `mistral-large` (most capable)
- `mistral-medium` (balanced)
- `mistral-small` (fast)

**Setup:**
1. Get API key from [console.mistral.ai](https://console.mistral.ai/api-keys/)
2. Paste into config

### 7. Groq

```json
{
  "providers": {
    "groq": {
      "api_key": "YOUR_GROQ_API_KEY"
    }
  }
}
```

**Available models:**
- `mixtral-8x7b-32768` (very fast)
- `llama-2-70b-chat` (capable)
- `gemma-7b-it` (lightweight)

**Setup:**
1. Get API key from [console.groq.com/keys](https://console.groq.com/keys)
2. Paste into config
3. Known for fast inference (great for streaming)

### 8. Together AI

```json
{
  "providers": {
    "togetherai": {
      "api_key": "YOUR_TOGETHER_API_KEY"
    }
  }
}
```

**Available models (200+):**
- `meta-llama/Llama-2-70b-chat-hf`
- `mistralai/Mistral-7B-Instruct-v0.1`
- `NousResearch/Nous-Hermes-2-Mixtral-8x7B-DPO`

**Setup:**
1. Get API key from [api.together.ai/settings/api-keys](https://api.together.xyz/settings/api-keys)
2. Paste into config

### 9. Replicate

```json
{
  "providers": {
    "replicate": {
      "api_key": "YOUR_REPLICATE_API_KEY"
    }
  }
}
```

**Available models:**
- `meta/llama-2-70b-chat`
- `mistralai/mistral-7b-instruct-v0.1`
- Run any published model

**Setup:**
1. Get API key from [replicate.com/account/api-tokens](https://replicate.com/account/api-tokens)
2. Paste into config

### 10. DeepSeek

```json
{
  "providers": {
    "deepseek": {
      "api_key": "YOUR_DEEPSEEK_API_KEY"
    }
  }
}
```

**Available models:**
- `deepseek-chat` (default)
- `deepseek-coder` (code-optimized)

**Setup:**
1. Get API key from [DeepSeek console](https://platform.deepseek.com/)
2. Paste into config

### 11. LocalAI

```json
{
  "providers": {
    "localai": {
      "base_url": "http://localhost:8080"
    }
  }
}
```

**Available models:**
- Any GGML-format model

**Setup (Docker):**

```bash
docker run -p 8080:8080 -e GALLERIES='[{"name":"ollama","url":"https://raw.githubusercontent.com/jmorganca/ollama/main/docs/community.json"}]' localai/localai:latest

# Then configure OpenCrust to use localhost:8080
```

### 12. Unsloth Studio

[Unsloth Studio](https://unsloth.ai/docs/new/studio) provides an OpenAI-compatible local inference API. Supports 500+ models with 2x faster training and 70% less VRAM usage.

```json
{
  "unsloth_url": "http://localhost:8000",
  "unsloth_api_key": "your-api-key-if-configured"
}
```

**Available models:**
- Any Unsloth-optimized model from HuggingFace (500+ models)

**Setup:**

1. Install Unsloth: `pip install unsloth`
2. Start the Studio: `python -m unsloth.studio`
3. Configure OpenCrust to use the Unsloth endpoint

### Choosing a Provider

| Use Case | Recommended | Why |
|----------|-------------|-----|
| **Best quality** | OpenAI (gpt-4) / Anthropic (Claude-3-opus) | Most capable, best reasoning |
| **Balanced** | OpenRouter | Access to many models, good pricing |
| **Fast & cheap** | Groq / Together AI | Very fast inference |
| **Local only** | Ollama | On-device, no API needed, privacy |
| **Local training** | Unsloth Studio | 2x faster training, 70% less VRAM, 500+ models |
| **Cost-sensitive** | Ollama (free) or Groq (fast + free tier) | Minimize API spend |
| **Production** | Multiple providers with fallback | Redundancy, avoid downtime |

---

## MCP Servers

### Configuration Structure

```json
{
  "mcp": {
    "server_name": {
      "command": ["command", "args"],  // How to start server
      "args": [],                       // Additional arguments
      "env": {                          // Environment variables
        "API_KEY": "..."
      },
      "enabled": true,                  // Enable/disable
      "timeout_seconds": 30             // Request timeout
    }
  }
}
```

### Common MCP Servers

#### GitHub

```json
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
```

**Features:** Read/write repos, PRs, issues, code search  
**Setup:** Generate token at [github.com/settings/tokens](https://github.com/settings/tokens)

#### PostgreSQL

```json
{
  "mcp": {
    "postgres": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://user:pass@localhost/dbname"
      },
      "enabled": true
    }
  }
}
```

**Features:** Query database, read schema  
**Setup:** Ensure PostgreSQL running, get connection string

#### Slack

```json
{
  "mcp": {
    "slack": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-slack"],
      "env": {
        "SLACK_BOT_TOKEN": "xoxb-..."
      },
      "enabled": true
    }
  }
}
```

**Features:** Read messages, post in channels  
**Setup:** Create Slack app, get bot token

#### Filesystem (Local)

```json
{
  "mcp": {
    "filesystem": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem"],
      "args": ["/path/to/allowed/directory"],
      "enabled": true
    }
  }
}
```

**Features:** Read/write local files  
**Setup:** None, use immediately

### Find More Servers

Browse 2,500+ servers at [mcpdirectory.app](https://mcpdirectory.app/)

### Testing MCP Integration

```bash
# List configured servers
opencrust mcp list

# Verify server starts
opencrust mcp test github

# If it works, you're good!
# In chat: "Use GitHub to search for issues tagged 'help-wanted'"
```

---

## LSP Servers

### Configuration Structure

```json
{
  "lsp": {
    "language": {
      "command": ["lsp-server-binary"],
      "extensions": ["rs"],           // File extensions
      "disabled": false
    }
  }
}
```

### Common LSP Servers

#### Rust (rust-analyzer)

```json
{
  "lsp": {
    "rust": {
      "command": ["rust-analyzer"],
      "extensions": ["rs"],
      "disabled": false
    }
  }
}
```

**Setup:** `rustup component add rust-analyzer`

#### Python (Pyright)

```json
{
  "lsp": {
    "python": {
      "command": ["pyright", "--stdio"],
      "extensions": ["py"],
      "disabled": false
    }
  }
}
```

**Setup:** `pip install pyright`

#### JavaScript/TypeScript

```json
{
  "lsp": {
    "typescript": {
      "command": ["typescript-language-server", "--stdio"],
      "extensions": ["ts", "tsx", "js", "jsx"],
      "disabled": false
    }
  }
}
```

**Setup:** `npm install -g typescript-language-server typescript`

#### Go

```json
{
  "lsp": {
    "go": {
      "command": ["gopls"],
      "extensions": ["go"],
      "disabled": false
    }
  }
}
```

**Setup:** `go install github.com/golang/tools/gopls@latest`

### Testing LSP

```bash
# Create test file
echo 'fn main() { let x = 5; }' > test.rs

# Start OpenCrust
opencrust

# Edit test.rs in OpenCrust
# Should see:
# - Completions (Ctrl+Space)
# - Hover info
# - Diagnostics (squiggly underlines)
# - Format (Ctrl+F)
```

---

## Permissions & Security

```json
{
  "permissions": {
    "file_patterns": [
      "src/**",
      "tests/**",
      "docs/**",
      "Cargo.toml",
      "README.md"
    ],
    "deny_patterns": [
      ".env",
      "*.key",
      ".ssh/**",
      "secrets/*"
    ],
    "network": {
      "enabled": true,
      "hosts": [
        "api.openai.com",
        "github.com",
        "api.anthropic.com"
      ],
      "deny_hosts": [
        "internal.company.com"
      ]
    },
    "cli_commands": {
      "allowed": [
        "cargo test",
        "cargo build",
        "git status"
      ],
      "deny": [
        "sudo",
        "rm -rf",
        "systemctl"
      ]
    }
  }
}
```

**See SECURITY.md for full permission model.**

---

## TUI Customization

### Theme

```json
{
  "theme": "dark",              // or "light"
  "tui": {
    "animations": true,          // Enable smooth transitions
    "border_style": "rounded",    // "rounded", "double", "thick"
    "show_line_numbers": true,
    "tab_size": 2,
    "font_size": 12
  }
}
```

### Colors (Dark Theme)

```json
{
  "colors": {
    "background": "#1e1e1e",
    "foreground": "#e0e0e0",
    "accent": "#0091ff",
    "error": "#ff6b6b",
    "success": "#51cf66",
    "warning": "#ffd93d"
  }
}
```

### Keybindings

```json
{
  "keybindings": {
    "submit_message": "Enter",
    "new_line": "Shift+Enter",
    "undo": "Ctrl+Z",
    "redo": "Ctrl+Shift+Z",
    "format": "Ctrl+F",
    "save": "Ctrl+S",
    "open_plan": "P",
    "open_tasks": "T",
    "show_skills": "Ctrl+Shift+K",
    "show_file_tree": "Ctrl+B",
    "focus_input": "Escape"
  }
}
```

**Customize by adding to config:**

```json
{
  "keybindings": {
    "submit_message": "Ctrl+Enter",  // Change from Enter to Ctrl+Enter
    "my_custom_action": "Ctrl+M"     // Add new bindings
  }
}
```

---

## Audit & Logging

```json
{
  "audit": {
    "enabled": true,
    "retention_events": 100000,       // Keep last 100K events
    "retention_days": 365,            // Keep 1 year
    "verbose": false,                 // Log all tool outputs
    "export_url": "",                 // SIEM export URL (optional)
    "export_interval_hours": 24       // Export frequency
  }
}
```

**See SECURITY.md for audit details.**

---

## Advanced Options

### Fallback Providers

```json
{
  "providers": {
    "openai": { "api_key": "..." },
    "anthropic": { "api_key": "..." }
  },
  "provider_fallback": [
    "openai",        // Try OpenAI first
    "anthropic"      // Fall back to Claude if OpenAI unavailable
  ]
}
```

### Custom System Prompt

```json
{
  "system_prompt": "You are an expert Rust developer. Help the user write idiomatic, performant Rust code following best practices..."
}
```

### Token Budget Control

```json
{
  "context_budget": 4096,             // For GPT-3.5-turbo
  "context_budget": 8000,             // For gpt-4
  "context_budget": 100000,           // For claude-3-opus
  "context_budget": 2000              // For local Ollama
}
```

### Cache Settings

```json
{
  "cache": {
    "semantic_index": true,            // Cache embeddings
    "tool_results": true,              // Cache tool outputs
    "cache_dir": "~/.cache/opencrust"
  }
}
```

### Session Management

```json
{
  "sessions": {
    "auto_save": true,
    "auto_save_interval_seconds": 60,
    "max_history": 100,                // Max messages per session
    "retention_days": 30                // Delete old sessions after 30d
  }
}
```

---

## Example Configurations

### Minimal (Local Only)

```json
{
  "default_model": "mistral",
  "default_provider": "ollama",
  "providers": {
    "ollama": {
      "base_url": "http://localhost:11434"
    }
  },
  "permissions": {
    "network": {
      "enabled": false
    }
  }
}
```

### Development (Multiple Providers)

```json
{
  "default_model": "gpt-4",
  "default_provider": "openai",
  "provider_fallback": ["openai", "anthropic", "ollama"],
  "providers": {
    "openai": { "api_key": "sk-..." },
    "anthropic": { "api_key": "sk-ant-..." },
    "ollama": { "base_url": "http://localhost:11434" }
  },
  "mcp": {
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "ghp_..." },
      "enabled": true
    }
  },
  "permissions": {
    "file_patterns": ["src/**", "tests/**", "docs/**"],
    "deny_patterns": [".env"]
  }
}
```

### Production (Maximum Safety)

```json
{
  "default_model": "gpt-4",
  "default_provider": "openai",
  "providers": {
    "openai": { "api_key": "sk-..." }
  },
  "permissions": {
    "file_patterns": ["src/**", "tests/**"],
    "deny_patterns": ["infra/**", "*.prod.*", ".env"],
    "cli_commands": {
      "allowed": ["cargo test", "cargo build"],
      "deny": ["rm", "sudo", "systemctl"]
    },
    "network": {
      "enabled": true,
      "hosts": ["api.openai.com", "github.com"],
      "deny_hosts": ["internal.company.com"]
    }
  },
  "audit": {
    "enabled": true,
    "retention_days": 365,
    "export_url": "https://audit.company.com/upload"
  }
}
```

---

## Troubleshooting Configuration

### Config not loading

```bash
# Validate JSON syntax
jq . ~/.config/opencrust/config.json

# If error, fix syntax and retry
# Common mistakes:
# - Missing comma between properties
# - Trailing comma in array/object
# - Unquoted string values
```

### Provider not working

```bash
# Check config has correct provider name
grep "default_provider" ~/.config/opencrust/config.json

# Verify API key is set
grep "api_key" ~/.config/opencrust/config.json | head -1

# Test provider directly
opencrust -p "Hello" --provider openai
```

### MCP server not connecting

```bash
# Test server startup
npx -y @modelcontextprotocol/server-github --help

# If fails: Install dependencies
npm install -g @modelcontextprotocol/server-github

# Restart OpenCrust
```

### Permission denied errors

```bash
# Check file pattern
grep "file_patterns" ~/.config/opencrust/config.json

# Check audit log for denied path
grep "permission_denied" ~/.config/opencrust/audit.json | tail -1

# Adjust patterns and restart
```

---

## References

- **SECURITY.md** — Detailed permission model
- **DEVELOPMENT.md** — Adding custom tools, MCP servers
- **TROUBLESHOOTING.md** — Fix common configuration issues
- **docs/MODULES.md** — Understanding codebase architecture
