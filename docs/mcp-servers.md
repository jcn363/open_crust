# OpenCrust MCP Server Integration Guide

This document catalogs the recommended MCP (Model Context Protocol) servers for OpenCrust, organized by category and priority.

## Why MCP Matters

- **97M+** monthly MCP SDK downloads
- **12,000+** servers available across npm, PyPI, and registries
- Universal protocol supported by Claude, Cursor, ChatGPT, and more
- OpenCrust's Rust foundation makes MCP integration exceptionally fast

---

## Recommended MCP Servers (Priority Order)

### Tier 1: Essential (Install First)

| Server | Installs | Purpose | Setup Complexity |
|--------|----------|---------|------------------|
| **Context7** | 120K+ | Version-accurate library docs | Low |
| **GitHub** | 398K+ | Repository, issues, PRs, CI/CD | Low |
| **PostgreSQL** | 312K+ | Natural language DB queries | Medium |
| **Brave Search** | 287K+ | Web research | Low |
| **Filesystem** | 485K+ | Enhanced file operations | Low |

### Tier 2: High Value

| Server | Installs | Purpose | Setup Complexity |
|--------|----------|---------|------------------|
| **Playwright** | 180K+ | Browser automation, E2E testing | Medium |
| **Supabase** | 95K+ | RLS-aware database access | Medium |
| **Sentry** | — | Error monitoring integration | Low |
| **Linear** | — | Issue tracking | Medium |
| **E2B** | — | Secure cloud sandbox for code execution | Medium |

### Tier 3: Production (Advanced)

| Server | Auth | Purpose |
|--------|------|---------|
| **Stripe** | OAuth | Payment integration |
| **HubSpot** | OAuth | CRM workflows |
| **AWS Suite** | OAuth | CloudWatch, Cost Explorer |
| **Cloudflare** | OAuth | Deploy, DNS, Workers |
| **Kubernetes** | Token | Container management |
| **Terraform** | Token | Infrastructure-as-code |

---

## Quick Start: Essential Stack

Add to your `~/.config/open_crust/config.json`:

```json
{
  "mcp": {
    "context7": {
      "command": ["npx", "-y", "@context7/mcp-server"],
      "enabled": true
    },
    "github": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "enabled": true,
      "environment": {
        "GITHUB_TOKEN": "ghp_xxxxxxxxxxxx"
      }
    },
    "brave-search": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-brave-search"],
      "enabled": true,
      "environment": {
        "BRAVE_API_KEY": "your-brave-api-key"
      }
    },
    "postgres": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-postgres"],
      "enabled": true,
      "environment": {
        "DATABASE_URL": "postgres://user:pass@localhost:5432/db"
      }
    },
    "filesystem": {
      "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem"],
      "enabled": true,
      "environment": {
        "ALLOWED_DIRS": "/home/user/projects"
      }
    }
  }
}
```

---

## Server Details

### Context7 MCP Server

**Purpose:** Real-time, version-accurate library documentation retrieval.

**Why it matters:** Eliminates hallucinated APIs — your AI always gets current documentation.

**Install:**
```bash
npx -y @context7/mcp-server
```

**Use cases:**
- "How do I use async/await in Rust 2024 edition?"
- "Show me the latest Ratatui API for Table widget"
- "What's the correct tokio::spawn signature?"

---

### GitHub MCP Server

**Purpose:** Full GitHub integration — repositories, issues, PRs, code search.

**Install:**
```bash
npx -y @modelcontextprotocol/server-github
```

**Environment:**
```json
{
  "GITHUB_TOKEN": "ghp_xxxxxxxxxxxx"
}
```

**Use cases:**
- "List open PRs in this repo"
- "Show me recent issues tagged 'bug'"
- "Create a new issue for the login timeout"

---

### PostgreSQL MCP Server

**Purpose:** Natural language database queries with schema introspection.

**Install:**
```bash
npx -y @modelcontextprotocol/server-postgres
```

**Environment:**
```json
{
  "DATABASE_URL": "postgres://user:pass@localhost:5432/db"
}
```

**Use cases:**
- "How many users signed up this week?"
- "Show me the schema for the orders table"
- "Find all inactive accounts older than 90 days"

**Security:** Read-only by default. Configure carefully for production.

---

### Brave Search MCP Server

**Purpose:** Web search without leaving the TUI.

**Install:**
```bash
npx -y @modelcontextprotocol/server-brave-search
```

**Environment:**
```json
{
  "BRAVE_API_KEY": "your-api-key"
}
```

**Get API key:** https://brave.com/search/api/

**Use cases:**
- "Search for Rust async runtime benchmarks 2026"
- "Find latest release of tokio"
- "Research MCP protocol specifications"

---

### Filesystem MCP Server

**Purpose:** Enhanced file operations beyond OpenCrust's built-in capabilities.

**Install:**
```bash
npx -y @modelcontextprotocol/server-filesystem
```

**Environment:**
```json
{
  "ALLOWED_DIRS": "/home/user/projects,/home/user/docs"
}
```

**Use cases:**
- Bulk file operations
- Recursive search within allowed directories
- File metadata operations

---

### Playwright MCP Server

**Purpose:** Browser automation and E2E testing.

**Install:**
```bash
npx -y @modelcontextprotocol/server-playwright
```

**Use cases:**
- "Take a screenshot of localhost:3000"
- "Fill out the login form and verify redirect"
- "Extract all links from this page"

**Note:** Requires Playwright browsers installed (`npx playwright install`)

---

### E2B MCP Server

**Purpose:** Secure cloud sandbox for code execution.

**Install:**
```bash
npx -y @e2b/mcp-server
```

**Environment:**
```json
{
  "E2B_API_KEY": "your-e2b-api-key"
}
```

**Use cases:**
- Execute Python/JS code safely
- Run data processing scripts
- Test code in isolated environment

**Why it matters:** Differentiator from competitors — execute code without risking local machine.

---

## MCP Registry Resources

- **Official Repository:** https://github.com/modelcontextprotocol/servers
- **Directory:** https://mcpdirectory.app/ (2,500+ servers)
- **Awesome List:** https://mcp-awesome.com/ (1,200+ verified)

---

## Troubleshooting

### Server Won't Start

1. Check Node.js is installed (`node --version`)
2. Verify npm is available (`npm --version`)
3. Try running the command manually in terminal

### Authentication Errors

1. Verify environment variables are set correctly
2. Check token permissions (e.g., GitHub token needs `repo` scope)
3. Ensure API keys are valid and not expired

### Tool Not Found

1. Restart OpenCrust after adding new MCP servers
2. Check server logs for initialization errors
3. Verify the tool name prefix (servers are prefixed with `servername_toolname`)

---

## Performance Notes

OpenCrust's Rust foundation provides:
- **Sub-millisecond** tool execution
- **Minimal memory footprint** (single static binary)
- **Efficient concurrency** for multiple MCP servers

Compared to Node.js-based agents (Cline, OpenCode), OpenCrust is typically 5-10x faster for tool-heavy workflows.

---

*Last updated: 2026-05-06*