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
  "provider": "openai",
  "model": "gpt-4",
  "openai_key": "sk-YOUR_KEY_HERE"
}
EOF
chmod 600 ~/.config/opencrust/config.json
```

---

## Configuration Hierarchy

Options are resolved in this order (first match wins):

1. **CLI flags** — `opencrust --model gpt-4-turbo`
2. **Environment variables** — `OPENCRUST_MODEL=gpt-4-turbo` (maps to the `model` field)
3. **config.json** — JSON file contents
4. **Defaults** — Built-in hardcoded values

**Example:**

```bash
# Built-in default: "openrouter/free" via OpenRouter
# Set in config: "model": "gpt-4", "openai_key": "sk-..."
# Set environment: export OPENCRUST_MODEL=claude-3-opus
# CLI flag: opencrust --model gemini-pro

# Result: Uses gemini-pro (CLI wins)
```

---

## Global Settings

**Note:** The `dirs` crate is used to locate the configuration directory (`~/.config/opencrust`). This is required for all platforms.

**Feature Flags:**
- `macos-menu-bar`: Enables native macOS menu bar integration.

## Global Settings

### Root Level

```json
{
  "provider": "openai",                        // LLM provider (see below)
  "model": "gpt-4",                            // Model name for the provider
  "context_budget": 8000,                      // Max tokens for context
  "summarization_threshold": 0.75,             // Context pressure for auto-summarize
  "auto_summarize": true,                      // Auto-summarize when near token limit
  "allowed_domains": [],                       // Allowed network domains
  "instructions": [],                          // Custom system prompt additions
  "token_budget_max_tokens": 1000000,          // Max tokens per session
  "token_budget_enabled": true,                // Enforce token budget
  "fallback_chain": [],                        // Provider failover order
  "role": "developer",                         // Role-based access (admin/developer/reviewer)
  "compliance_mode": false,                    // Enable compliance logging
  "audit_retention_days": 365,                 // Audit log retention
  "audit_max_size_bytes": 104857600            // Max audit log size (100MB)
}
```

### Explanation

| Setting | Type | Default | Notes |
|---------|------|---------|-------|
| `provider` | string | `"openrouter"` | LLM provider name (see provider list below) |
| `model` | string | `"openrouter/free"` | Model to use with the selected provider |
| `context_budget` | number (optional) | auto | Max tokens for context (provider-dependent) |
| `summarization_threshold` | float (optional) | `0.75` | Context pressure (0.0-1.0) to trigger summarization |
| `auto_summarize` | boolean | `true` | Automatically summarize context when approaching token limit |
| `allowed_domains` | string[] | `[]` | Whitelist of network domains for tool access |
| `instructions` | string[] | `[]` | Custom instructions added to system prompt |
| `token_budget_max_tokens` | number | `1000000` | Hard cap on tokens per session |
| `token_budget_enabled` | boolean | `true` | Enable/disable token budget enforcement |
| `fallback_chain` | string[] | `[]` | Ordered provider names for failover (e.g. `["openai", "anthropic"]`) |
| `role` | string | `"developer"` | RBAC role: `"admin"`, `"developer"`, `"reviewer"` |
| `compliance_mode` | boolean | `false` | Enable structured compliance evidence logging |
| `audit_retention_days` | number | `365` | Days to keep audit log events |
| `audit_max_size_bytes` | number | `104857600` | Max audit log file size (100MB) |

---

## LLM Providers

### Configuration Structure

OpenCrust uses a flat config layout — each provider's API key/URL is a top-level field:

```json
{
  "provider": "openai",
  "model": "gpt-4",
  "openai_key": "sk-...",
  "anthropic_api_key": "sk-ant-..."
}
```

Some providers need a URL field instead of (or in addition to) a key: `ollama_url`,
`localai_url`, `unsloth_url`.

### 1. OpenAI

```json
{
  "provider": "openai",
  "model": "gpt-4",
  "openai_key": "sk-YOUR_KEY_HERE"
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
  "provider": "anthropic",
  "model": "claude-3-opus-20240229",
  "anthropic_api_key": "sk-ant-YOUR_KEY_HERE"
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
  "provider": "gemini",
  "model": "gemini-1.5-pro",
  "gemini_api_key": "YOUR_GEMINI_API_KEY"
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
  "provider": "ollama",
  "model": "mistral",
  "ollama_url": "http://localhost:11434"
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
  "provider": "openrouter",
  "model": "openrouter/free",
  "openrouter_key": "sk-or-YOUR_KEY_HERE"
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
  "provider": "mistral",
  "model": "mistral-large",
  "mistral_api_key": "YOUR_MISTRAL_API_KEY"
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
  "provider": "groq",
  "model": "mixtral-8x7b-32768",
  "groq_api_key": "YOUR_GROQ_API_KEY"
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
  "provider": "togetherai",
  "model": "meta-llama/Llama-2-70b-chat-hf",
  "together_api_key": "YOUR_TOGETHER_API_KEY"
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
  "provider": "replicate",
  "model": "meta/llama-2-70b-chat",
  "replicate_api_key": "YOUR_REPLICATE_API_KEY"
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
  "provider": "deepseek",
  "model": "deepseek-chat",
  "deepseek_api_key": "YOUR_DEEPSEEK_API_KEY"
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
  "provider": "localai",
  "model": "any-ggml-model",
  "localai_url": "http://localhost:8080",
  "localai_api_key": "optional-api-key"
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
  "provider": "unsloth",
  "model": "any-huggingface-model",
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

### 13. Azure OpenAI

```json
{
  "provider": "azure",
  "model": "gpt-4",
  "azure_api_key": "YOUR_AZURE_KEY",
  "azure_endpoint": "https://your-instance.openai.azure.com/"
}
```

### 14. GitHub Copilot

```json
{
  "provider": "github-copilot",
  "model": "gpt-4",
  "copilot_key": "YOUR_COPILOT_TOKEN"
}
```

### 15. Amazon Bedrock

```json
{
  "provider": "bedrock",
  "model": "anthropic.claude-3-sonnet-20240229-v1:0",
  "aws_access_key": "YOUR_ACCESS_KEY",
  "aws_secret_key": "YOUR_SECRET_KEY",
  "aws_region": "us-east-1"
}
```

### 16. Vertex AI

```json
{
  "provider": "vertex",
  "model": "gemini-1.5-pro",
  "vertex_project_id": "YOUR_PROJECT_ID",
  "vertex_location": "us-central1"
}
```

### 17. Perplexity

```json
{
  "provider": "perplexity",
  "model": "pplx-70b-online",
  "perplexity_api_key": "YOUR_KEY"
}
```

### 18. Cohere

```json
{
  "provider": "cohere",
  "model": "command-r-plus",
  "cohere_api_key": "YOUR_KEY"
}
```

### 19. Cerebras

```json
{
  "provider": "cerebras",
  "model": "llama3.1-70b",
  "cerebras_api_key": "YOUR_KEY"
}
```

### 20. Alibaba Cloud

```json
{
  "provider": "alibaba",
  "model": "qwen-max",
  "alibaba_api_key": "YOUR_KEY"
}
```

### 21. Venice AI

```json
{
  "provider": "venice",
  "model": "llama-3.3-70b",
  "venice_api_key": "YOUR_KEY"
}
```

### 22. NVIDIA NIM

```json
{
  "provider": "nvidia",
  "model": "meta/llama-3.1-405b-instruct",
  "nvidia_api_key": "YOUR_KEY"
}
```

### 23. Fireworks AI

```json
{
  "provider": "fireworks",
  "model": "accounts/fireworks/models/llama-v3p1-405b-instruct",
  "fireworks_api_key": "YOUR_KEY"
}
```

### 24. SambaNova

```json
{
  "provider": "sambanova",
  "model": "Meta-Llama-3.1-405B-Instruct",
  "sambanova_api_key": "YOUR_KEY"
}
```

### 25. OctoAI

```json
{
  "provider": "octoai",
  "model": "meta-llama-3.1-405b-instruct",
  "octoai_api_key": "YOUR_KEY"
}
```

### 26. Anyscale

```json
{
  "provider": "anyscale",
  "model": "meta-llama/Llama-3-70B-Instruct",
  "anyscale_api_key": "YOUR_KEY"
}
```

### 27. Lambda Labs

```json
{
  "provider": "lambdalabs",
  "model": "lambda-labs/llama-3.1-405b-instruct",
  "lambdalabs_api_key": "YOUR_KEY"
}
```

### 28. RunPod

```json
{
  "provider": "runpod",
  "model": "meta-llama/Meta-Llama-3.1-405B-Instruct",
  "runpod_api_key": "YOUR_KEY"
}
```

### 29. Modal

```json
{
  "provider": "modal",
  "model": "meta-llama-3.1-405b-instruct",
  "modal_api_key": "YOUR_KEY"
}
```

### 30. Hugging Face

```json
{
  "provider": "huggingface",
  "model": "meta-llama/Llama-3.1-405B-Instruct",
  "huggingface_api_key": "YOUR_KEY"
}
```

### 31. LM Studio

```json
{
  "provider": "lmstudio",
  "model": "meta-llama-3.1-405b-instruct",
  "lmstudio_url": "http://localhost:1234"
}
```

### 32. Text Generation Inference (TGI)

```json
{
  "provider": "tgi",
  "model": "meta-llama/Meta-Llama-3.1-405B-Instruct",
  "tgi_url": "http://localhost:8080"
}
```

### 33. vLLM

```json
{
  "provider": "vllm",
  "model": "meta-llama/Meta-Llama-3.1-405B-Instruct",
  "vllm_url": "http://localhost:8000"
}
```

### 34. Custom OpenAI-Compatible

```json
{
  "provider": "custom-openai",
  "model": "your-model-name",
  "custom_openai_url": "http://your-endpoint/v1",
  "custom_openai_key": "YOUR_KEY"
}
```

### Choosing a Provider

| Use Case | Provider | Config Field | Why |
|----------|----------|-------------|-----|
| **Best quality** | `openai` / `anthropic` | `openai_key` / `anthropic_api_key` | Most capable, best reasoning |
| **Balanced** | `openrouter` | `openrouter_key` | Access to many models, good pricing |
| **Fast & cheap** | `groq` / `togetherai` | `groq_api_key` / `together_api_key` | Very fast inference |
| **Local only** | `ollama` | `ollama_url` | On-device, no API needed, privacy |
| **Local training** | `unsloth` | `unsloth_url` | 2x faster training, 70% less VRAM |
| **Cost-sensitive** | `ollama` (free) / `groq` (free tier) | `ollama_url` / `groq_api_key` | Minimize API spend |
| **Production** | Any with `fallback_chain` | Multiple key fields | Redundancy, avoid downtime |

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
  "permission": {
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
  "theme": {
    "name": "dark",              // or "light"
    "colors": {
      "background": "#1e1e1e",
      "foreground": "#e0e0e0",
      "accent": "#0091ff",
      "error": "#ff6b6b",
      "success": "#51cf66",
      "warning": "#ffd93d"
    }
  },
  "tui": {
    "animations": true,          // Enable smooth transitions
    "border_style": "rounded",    // "rounded", "double", "thick"
    "show_line_numbers": true,
    "tab_size": 2,
    "font_size": 12
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
  "audit_retention_days": 365,        // Keep 1 year
  "audit_max_size_bytes": 104857600,  // Max audit log size (100MB)
  "compliance_mode": false,           // Enable structured compliance logging
  "compliance_log_path": null         // Optional path for compliance exports
}
```

**See SECURITY.md for audit details.**

---

## Advanced Options

### Fallback Providers

```json
{
  "openai_key": "...",
  "anthropic_api_key": "...",
  "fallback_chain": [
    "openai",        // Try OpenAI first
    "anthropic"      // Fall back to Claude if OpenAI unavailable
  ]
}
```

### Custom System Prompt

```json
{
  "instructions": [
    "You are an expert Rust developer.",
    "Help the user write idiomatic, performant Rust code following best practices..."
  ]
}
```

### Token Budget Control

```json
{
  "token_budget_max_tokens": 1000000,  // Max tokens per session
  "token_budget_enabled": true,        // Enforce budget
  "context_budget": 8000               // Context window per request (provider-dependent)
}
```

### Cache Settings

```json
{
  "model_auto_refresh": {
    "enabled": true,
    "interval_hours": 24
  }
}
```

### Session Management

Session persistence is automatic. Configure via CLI:
- `opencrust session save --name "name"` — save current session
- `opencrust session list` — list saved sessions
- `opencrust session show <id>` — show session details
- `opencrust session delete <id>` — delete a session

---

## Example Configurations

### Minimal (Local Only)

```json
{
  "provider": "ollama",
  "model": "mistral",
  "ollama_url": "http://localhost:11434",
  "allowed_domains": []
}
```

### Development (Multiple Providers)

```json
{
  "provider": "openai",
  "model": "gpt-4",
  "openai_key": "sk-...",
  "anthropic_api_key": "sk-ant-...",
  "ollama_url": "http://localhost:11434",
  "fallback_chain": ["openai", "anthropic", "ollama"],
  "mcp": {
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "ghp_..." },
      "enabled": true
    }
  },
  "permission": {
    "file_patterns": ["src/**", "tests/**", "docs/**"],
    "deny_patterns": [".env"]
  }
}
```

### Production (Maximum Safety)

```json
{
  "provider": "openai",
  "model": "gpt-4",
  "openai_key": "sk-...",
  "permission": {
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
  "audit_retention_days": 365,
  "compliance_mode": true,
  "compliance_log_path": "/var/log/opencrust/compliance.json",
  "role": "admin"
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
grep '"provider"' ~/.config/opencrust/config.json

# Verify API key is set (field name depends on provider)
grep -E '(openai_key|anthropic_api_key|gemini_api_key|groq_api_key|together_api_key|replicate_api_key|deepseek_api_key|localai_api_key|unsloth_api_key|openrouter_key)' ~/.config/opencrust/config.json | head -1

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
# Check file pattern (under "permission" key)
grep -A 10 '"permission"' ~/.config/opencrust/config.json | grep file_patterns

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
