# Security Model & Audit System

OpenCrust runs untrusted code (scripts, MCP servers, LLM outputs). This document explains how we prevent accidents and maintain an audit trail.

**For architecture context:** See **docs/ARCHITECTURE.md**.  
**For config options:** See **docs/CONFIGURATION.md**.  
**For coding standards:** See **AGENTS.md**.

---

## Quick Start: Security by Default

**Out of the box:**

- ✅ No file access outside your workspace
- ✅ No network access to external hosts (unless explicitly allowed)
- ✅ All tool executions logged
- ✅ API keys never logged or displayed
- ✅ Custom scripts run in isolated processes
- ✅ Permission errors prevent accidents before they happen

**Example: You're safe if you do this:**

```bash
# Run OpenCrust pointing to your project
opencrust ~/my_project

# Ask LLM to read files, write code, run linter
# LLM cannot:
# - Read files outside ~/my_project
# - Delete your home directory
# - Access .env or secrets
# - Make network calls to unknown hosts
# - Install packages globally
```

---

## Permissions Model

### Overview

**Three layers of permission checking:**

```
1. File I/O Permission
   • Pattern matching (glob): "src/**" ✅, "docs/**" ✅, ".env" ❌
   • Explicit deny list: Override patterns

2. Execution Permission
   • Only whitelisted tools can run
   • Scripts must be in .opencrust/tools/
   • CLI commands checked against allowlist

3. Network Permission
   • Only configured hosts allowed
   • Blocks SSH to unknown servers
   • MCP server connections validated
```

### File I/O Permissions

**Configuration (config.json):**

```json
{
  "permissions": {
    "file_patterns": [
      "src/**",           // Allow read/write entire src/
      "docs/**",          // Allow read/write entire docs/
      ".opencrust/**",    // Allow custom tools, config
      "Cargo.toml",       // Specific file
      "README.md"
    ],
    "deny_patterns": [
      ".env",             // Block env files
      "*.key",            // Block private keys
      ".ssh/**",          // Block SSH config
      "secrets/*"
    ]
  }
}
```

**Evaluation logic:**

```
User requests: file_read("secrets/api_key.txt")
  ↓
Check deny_patterns:
  • Does "secrets/api_key.txt" match "secrets/*"? YES
  ↓
DENIED: "Access denied: path blocked by deny_patterns"
  ↓
Return error to LLM (LLM can then ask user for permission or suggest alternatives)
```

**Without deny_patterns, evaluation:**

```
User requests: file_read("src/main.rs")
  ↓
Check file_patterns:
  • Does "src/main.rs" match "src/**"? YES
  ↓
ALLOWED: Read file and return content
```

### Execution Permissions

**Built-in tools** (file_read, file_write, code_search, etc.) are always available if file permissions pass.

**Custom tools** must be in `.opencrust/tools/` directory:

```bash
# Discovered on startup
ls -la .opencrust/tools/
  my_linter               ✅ Discovered
  run_tests               ✅ Discovered
  ../hidden_script.sh     ❌ Not in .opencrust/tools/ → DENIED

# To execute: user_request("execute_tool", "my_linter", "src/main.rs")
#  ↓
#  Check: Is "my_linter" in discovered tools? YES
#  Check: Does file permission allow reading "src/main.rs"? YES
#  ↓
#  Execute: ./.opencrust/tools/my_linter src/main.rs
```

**CLI commands** checked against allowlist:

```json
{
  "permissions": {
    "cli_commands": {
      "allowed": ["cargo test", "cargo fmt", "git status"],
      "deny": ["rm -rf", "sudo", "systemctl"]
    }
  }
}
```

### Network Permissions

**Configuration (config.json):**

```json
{
  "permissions": {
    "network": {
      "enabled": true,
      "hosts": [
        "api.openai.com",
        "api.anthropic.com",
        "github.com",
        "raw.githubusercontent.com"
      ],
      "deny_hosts": ["internal.company.com"]
    }
  }
}
```

**Evaluation logic:**

```
Tool requests: ssh("internal.company.com", "list_users")
  ↓
Check network enabled? YES
Check deny_hosts: Is "internal.company.com" blocked? YES
  ↓
DENIED: "Network access to internal.company.com blocked"
  ↓
Return error: User must explicitly allow in config
```

**Default deny-list:**
- `127.0.0.1` and `localhost` are **always allowed** (local development)
- Private IP ranges (`10.0.0.0/8`, `192.168.0.0/16`, `172.16.0.0/12`) are **denied by default**
- Intranet hosts require explicit allowlist

---

## Audit System

### What Gets Logged

**Audit events are JSON records:**

```json
{
  "timestamp": "2026-05-08T14:30:42.123Z",
  "session_id": "sess_abc123def456",
  "event_type": "tool_executed",
  "tool_name": "file_read",
  "path": "/home/alice/project/src/main.rs",
  "arguments": [],
  "result": "success",
  "output_lines": 142,
  "exit_code": 0,
  "duration_ms": 12,
  "user": "alice"
}
```

**Event types logged:**

| Event | Details | Sensitive? |
|-------|---------|-----------|
| `tool_executed` | Tool name, args, exit code, output size | No |
| `file_accessed` | File path, operation (read/write/delete), size | No |
| `permission_denied` | Reason, requested path/tool | No |
| `network_request` | Host, port, protocol | No (host only) |
| `lsb_called` | Model, token count, cost estimate | No |
| `skill_loaded` | Skill name, version | No |
| `session_created` | Session ID, start time | No |
| `session_ended` | Session ID, total cost, duration | No |

**Never logged:**
- File contents (even if read)
- API responses (only metadata)
- LLM conversations (stored separately in sessions)
- Passwords, tokens, secrets (stripped before logging)

### Accessing Audit Logs

**Location:** `~/.config/open_crust/audit.json` (JSONL format — one event per line)

**Query recent events:**

```bash
# Last 10 events
tail -10 ~/.config/open_crust/audit.json

# All file access today
grep '"event_type":"file_accessed"' ~/.config/open_crust/audit.json | \
  grep "$(date +%Y-%m-%d)"

# All permission denials
grep '"result":"permission_denied"' ~/.config/open_crust/audit.json

# All events for a session
grep '"session_id":"sess_abc123"' ~/.config/open_crust/audit.json
```

**Retention policy:**
- Keep last 100,000 events (configurable)
- Rotate to `audit.1.json`, `audit.2.json` when full
- Compress old files: `audit.3.json.gz`

### Audit Compliance

**For organizations requiring compliance:**

```json
{
  "audit": {
    "enabled": true,
    "retention_events": 1000000,  // Keep 1M events
    "retention_days": 365,         // Keep 1 year
    "export_url": "https://audit.company.com/upload",  // Export to SIEM
    "export_interval_hours": 24,
    "include_pii": false           // Never include user info
  }
}
```

**To extract audit trail for external system:**

```bash
# Export past 7 days as CSV
jq -r '[.timestamp, .event_type, .tool_name, .result] | @csv' \
  ~/.config/open_crust/audit.json | \
  awk 'BEGIN{print "timestamp,event_type,tool,result"} {print}' \
  > audit_export.csv
```

---

## Sandboxing

### Custom Tool Execution

**When you run a custom tool:**

```bash
# User/LLM requests: execute_tool("deploy", "prod")
# 
# OpenCrust does:
# 1. Check permission: Is "deploy" in .opencrust/tools/? ✅
# 2. Check file access: Can execute based on patterns? ✅
# 3. Sandbox setup:
#    - Create isolated process (not same PID namespace)
#    - No direct filesystem access (must go through OpenCrust)
#    - Inherit only safe environment variables
#    - Set resource limits (CPU, memory, time)
# 4. Execute: ./.opencrust/tools/deploy prod
# 5. Capture: stdout, stderr, exit code
# 6. Log: All details to audit.json
# 7. Return: Output to LLM
```

**What the script CAN do:**
- Read files in allowed patterns
- Write to temp directory
- Call external commands (curl, git, cargo, etc.)
- Make network requests (if allowed)

**What the script CANNOT do:**
- Access files outside allowed patterns
- Modify OpenCrust configuration
- Kill OpenCrust process
- Access files from other OpenCrust sessions

### MCP Server Sandboxing

**MCP servers are external processes communicating via JSON-RPC over stdio:**

```
OpenCrust              MCP Server (e.g., github)
    ↓                          ↑
  stdin ─────→ [JSON-RPC req]
  stdout ←───── [JSON-RPC resp]
    ↑                          ↓
```

**Sandboxing:**
- Server runs as subprocess (no direct access to OpenCrust state)
- stdin/stdout only communication (cannot access filesystem directly)
- Timeout: If server doesn't respond in 30s, kill it
- Resource limits: Memory capped at 500MB

**What MCP server CAN do:**
- Process tool calls (GitHub API, database queries, etc.)
- Return structured results

**What MCP server CANNOT do:**
- Access local filesystem (unless explicitly given via tool params)
- Access other MCP servers
- Modify OpenCrust config
- Access user session state

---

## Secrets Management

### Where Secrets Live

```
✅ Safe: config.json (with restricted permissions)
~/.config/open_crust/config.json: chmod 600

Example:
{
  "providers": {
    "openai": {
      "api_key": "sk-..."  // OK: file is private
    }
  },
  "mcp": {
    "github": {
      "env": {
        "GITHUB_TOKEN": "ghp_..."  // OK: environment-only
      }
    }
  }
}

❌ DANGER: Storing in .env file in workspace
src/
├── .env                    // DON'T: Would be accessible to tools!
├── main.rs
└── app.rs

SOLUTION: Use config.json with proper permissions
```

### Secret Handling Rules

1. **In configuration:**
   - Store secrets in `~/.config/open_crust/config.json`
   - Set file permissions: `chmod 600 ~/.config/open_crust/config.json`
   - Never store in workspace directory

2. **In environment variables:**
   - Pass to MCP servers/custom tools via env:
     ```json
     {
       "mcp": {
         "github": {
           "env": {
             "GITHUB_TOKEN": "ghp_123..."
           }
         }
       }
     }
     ```
   - Tool receives as environment variable (not logged, not displayed)

3. **Never logged:**
   - Audit system strips API keys before logging
   - LLM responses never logged (use separate session storage)
   - File contents never logged

4. **Rotation:**
   - Update config.json in place
   - Restart OpenCrust to pick up new keys
   - Old keys immediately invalidated

### Best Practice: API Key Management

**Setup GitHub MCP securely:**

```bash
# 1. Create personal access token
# GitHub → Settings → Developer settings → Personal access tokens
# Generate token with 'repo' scope, copy it

# 2. Add to OpenCrust config
cat >> ~/.config/open_crust/config.json << 'EOF'
{
  "mcp": {
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "ghp_YOUR_TOKEN_HERE"
      }
    }
  }
}
EOF

# 3. Fix permissions
chmod 600 ~/.config/open_crust/config.json

# 4. Restart OpenCrust
# Now uses GitHub via MCP (token never shown in UI or logs)
```

**For rotation (token expired/compromised):**

```bash
# 1. Revoke old token on GitHub
# 2. Generate new token
# 3. Update config:
sed -i 's/ghp_OLD_TOKEN/ghp_NEW_TOKEN/g' ~/.config/open_crust/config.json
# 4. Restart OpenCrust
```

---

## Network Security

### Default Configuration (Secure)

```json
{
  "permissions": {
    "network": {
      "enabled": true,
      "hosts": [
        "api.openai.com",
        "api.anthropic.com",
        "github.com",
        "raw.githubusercontent.com",
        "huggingface.co"
      ]
    }
  }
}
```

**What's blocked by default:**
- Any host not in `hosts` list
- Private IP ranges (192.168.x.x, 10.x.x.x, 172.16-31.x.x)
- Localhost/127.0.0.1 (unless explicitly enabled)
- SSH to unknown servers

### Secure MCP Integration

**Only allow safe MCP servers:**

```json
{
  "mcp": {
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "enabled": true
    },
    "untrusted_server": {
      "command": ["./custom_mcp_server"],
      "enabled": false  // Disabled by default
    }
  }
}
```

### Offline Mode

**No network access at all:**

```json
{
  "permissions": {
    "network": {
      "enabled": false
    }
  }
}
```

**Consequences:**
- Cannot call remote LLM APIs (use local Ollama only)
- Cannot access GitHub, Slack, etc. via MCP
- Cannot fetch remote files
- Cannot reach semantic search APIs

**Use case:** Air-gapped environments, security-sensitive work

---

## Common Security Scenarios

### Scenario 1: You want to share your project with an untrusted contributor

**Risk:** They might read your .env file or deploy script

**Solution:**

```json
{
  "permissions": {
    "file_patterns": ["src/**", "docs/**"],
    "deny_patterns": [
      ".env",
      ".env.local",
      "config/*.prod.json",
      "scripts/deploy*",
      "credentials/*"
    ]
  }
}
```

Now the contributor can edit src/ and docs/, but not secrets.

### Scenario 2: You want to allow a tool to modify test files only

**Risk:** Tool might accidentally modify source code

**Solution:**

```json
{
  "permissions": {
    "file_patterns": ["tests/**", "Cargo.toml"],
    "deny_patterns": ["src/**"]
  }
}
```

### Scenario 3: You're using MCP but worried about security

**Risk:** MCP server might leak data

**Solution:**

```json
{
  "mcp": {
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "enabled": true,
      "timeout_seconds": 5  // Kill if hangs
    }
  },
  "audit": {
    "enabled": true,
    "export_url": "https://audit.company.com/upload"
  }
}
```

Monitor audit logs: Every MCP call is recorded.

### Scenario 4: Production deployment (maximum safety)

**Risk:** Accidental modification of production files

**Solution:**

```json
{
  "permissions": {
    "file_patterns": ["src/**", "docs/**", "tests/**"],
    "deny_patterns": ["infra/**", "terraform/**", "*.prod.*"],
    "cli_commands": {
      "allowed": ["cargo build", "cargo test"],
      "deny": ["sudo", "rm", "systemctl", "aws", "docker"]
    },
    "network": {
      "enabled": true,
      "hosts": ["api.github.com"],
      "deny_hosts": ["prod-database.internal", "aws.amazon.com"]
    }
  },
  "audit": {
    "enabled": true,
    "retention_days": 365
  }
}
```

Now:
- No access to infra code
- No destructive CLI commands
- No connection to production systems
- Full audit trail for compliance

---

## Security Checklist

**Before sharing project with others:**

- [ ] Review `file_patterns` in config.json (what can be read/written)
- [ ] Review `deny_patterns` (secrets, private code)
- [ ] Review `cli_commands` (what can be executed)
- [ ] Review network `hosts` (what can be accessed)
- [ ] Check `~/.config/open_crust/config.json` permissions: `chmod 600`
- [ ] Rotate any API keys if shared
- [ ] Enable audit logging
- [ ] Test: Try accessing a denied file/host, verify error message

**Before production deployment:**

- [ ] All secrets in config, not .env or workspace
- [ ] File permissions locked down (minimal access)
- [ ] Network access restricted to required hosts only
- [ ] Audit logging enabled
- [ ] MCP servers whitelisted
- [ ] CLI commands restricted
- [ ] Offline mode enabled (if no network needed)

---

## Threat Model

### Threats We Defend Against

✅ **LLM generates malicious code** → Code reviewed before execution, tools require permission  
✅ **Tool script reads .env** → Denied by default unless explicitly allowed  
✅ **MCP server steals data** → Runs in sandbox, communication via JSON-RPC only  
✅ **Attacker modifies config** → File permissions (600) prevent unauthorized access  
✅ **Tool crashes, leaves temp files** → Subprocess isolation (separate PID)  
✅ **Network exfiltration** → Denied by default, explicit allowlist required  

### Threats We DON'T Defend Against

❌ **User runs malicious config voluntarily** → User is in control  
❌ **User's machine compromised** → OS-level threat (outside scope)  
❌ **SSH key stolen from ~/.ssh** → OS-level threat, use ssh-agent  
❌ **Keylogger on user's system** → OS-level threat, use secure keyboard  

---

## Debugging Permissions Issues

### Tool execution denied

```bash
# Check which tool failed:
grep '"result":"permission_denied"' ~/.config/open_crust/audit.json | tail -1

# Read the error message:
{
  "event_type": "tool_executed",
  "tool_name": "my_linter",
  "result": "permission_denied",
  "reason": "Tool 'my_linter' not found in .opencrust/tools/"
}

# Solution: Move tool to correct location
mv ~/my_linter .opencrust/tools/my_linter
chmod +x .opencrust/tools/my_linter
```

### File access denied

```bash
# Check denied path:
grep '"event_type":"file_accessed".*"result":"permission_denied"' \
  ~/.config/open_crust/audit.json | tail -1

# Result:
{
  "event_type": "file_accessed",
  "path": "secrets/.env",
  "result": "permission_denied",
  "reason": "Path matches deny_patterns: secrets/*"
}

# Solution: Add to allowed patterns in config.json
"file_patterns": ["src/**", "secrets/config.json"]
```

---

## References

- **ARCHITECTURE.md** — How security boundaries are enforced
- **DEVELOPMENT.md** — Writing secure custom tools
- **CONFIGURATION.md** — All permission options
- **CONTRIBUTING.md** — Security review checklist for PRs
