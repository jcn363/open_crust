# Extending OpenCrust: Development Guide

This guide shows you how to add features to OpenCrust: custom tools, skills, CLI commands, TUI components, configuration options, and language server integrations.

See **docs/MODULES.md** for understanding where each part lives.  
See **AGENTS.md** for coding standards.  
See **docs/EXAMPLES.md** for practical walkthroughs.

---

## 1. Adding a Custom Tool

**When to use:** You want OpenCrust to run a script or command that's not built-in.

### How It Works

Custom tools are discovered from `.opencrust/tools/` at startup. Each tool is an executable script with a comment header.

### Step-by-Step

**1. Create the script file**

```bash
cat > .opencrust/tools/my_linter << 'EOF'
#!/bin/bash
# name: my_linter
# description: Check code with my custom linter

set -e

# Accept file path as argument
FILE="$1"
if [[ ! -f "$FILE" ]]; then
  echo "Error: File not found: $FILE" >&2
  exit 1
fi

# Run linter (example)
pylint "$FILE" || exit 1
echo "Linting passed for $FILE"
EOF

chmod +x .opencrust/tools/my_linter
```

**2. Verify OpenCrust discovers it**

```bash
# Restart OpenCrust
opencrust

# In chat, ask: "Use my_linter on src/file.py"
# OpenCrust will find and execute it
```

### What OpenCrust Does

1. **Discovers:** Scans `.opencrust/tools/` for executables
2. **Parses:** Reads comment header for `name:` and `description:`
3. **Registers:** Adds tool to available commands
4. **Executes:** Runs with provided arguments
5. **Logs:** Records in audit log

### Error Handling

If your script fails:
- Exit with non-zero code: `exit 1`
- Write error to stderr: `echo "error" >&2`
- OpenCrust will capture and report failure

### Permissions

If your script needs file access, OpenCrust will check permissions from `config.json`:

```json
{
  "permissions": {
    "file_patterns": [".opencrust/tools/**"]
  }
}
```

---

## 2. Adding a Skill

**When to use:** You want to inject specialized behavior or instructions for specific tasks.

### How It Works

Skills are YAML + markdown files in `.opencrust/skills/<skill-name>/SKILL.md`. OpenCrust loads them and passes them as context to the LLM.

### Step-by-Step

**1. Create the skill file**

```bash
mkdir -p .opencrust/skills/my_skill
cat > .opencrust/skills/my_skill/SKILL.md << 'EOF'
---
name: my_skill
description: Specialized behavior for my project
priority: high
---

## Instructions

When the user asks you to [task], follow these steps:

1. First, understand the requirement
2. Check for existing patterns in the codebase (use `semantic_search`)
3. Follow the project's style (see AGENTS.md)
4. Write tests first (TDD approach)
5. Verify with `cargo test` before responding

## Key Practices

- Always use Rust idioms from the project
- Reference AGENTS.md for standards
- Check existing code patterns
- Never unwrap in production code

## Example

When asked to add a feature:
- [ ] Find similar features in codebase
- [ ] Follow existing patterns
- [ ] Write tests
- [ ] Verify clippy passes
EOF
```

**2. Activate the skill**

```bash
# In OpenCrust, use Skill Browser (Ctrl+Shift+K)
# Or from CLI:
opencrust skills activate my_skill
```

**3. Verify it's loaded**

When the skill is active, it will be included in the LLM context for relevant tasks.

### Key Practices

- **Name:** Use snake_case, short and descriptive
- **Description:** 1-2 sentences what it does
- **Priority:** `high` for critical, `normal` for standard
- **Instructions:** Clear, actionable steps
- **Length:** 200-500 words (concise but complete)

### Where Skills Help

- Coding standards enforcement (e.g., "rust-expert")
- Security guidelines (e.g., "security-auditor")
- Project-specific practices (e.g., "follow this pattern")
- Team workflows (e.g., "git-workflow")

---

## 3. Adding an MCP Server Integration

**When to use:** You want to connect an external service (GitHub, Slack, database, etc.).

### How It Works

MCP servers expose capabilities via JSON-RPC. OpenCrust discovers and loads them from `config.json` or install via CLI.

### Step-by-Step

**1. Configure in config.json**

```json
{
  "mcp": {
    "my_service": {
      "command": ["npx", "-y", "@org/my-mcp-server"],
      "args": ["--arg1", "value1"],
      "env": {
        "MY_API_KEY": "secret_key"
      },
      "enabled": true
    }
  }
}
```

**2. Restart OpenCrust**

```bash
opencrust
```

**3. Use it in chat**

```
User: "Query my_service for records matching X"
OpenCrust: Uses MCP server tools to query
```

### Common MCP Servers

- **GitHub:** `@modelcontextprotocol/server-github`
- **Postgres:** `@modelcontextprotocol/server-postgres`
- **Filesystem:** `@modelcontextprotocol/server-filesystem`
- **Slack:** Community MCP server for Slack

### Where to Find Servers

Browse 2,500+ servers at [mcpdirectory.app](https://mcpdirectory.app/)

### Testing Integration

```bash
# Check if server starts
opencrust mcp list

# Try calling it
# In chat: "Use [server_name] to [do something]"
```

---

## 4. Adding a CLI Command

**When to use:** You want a new terminal-native command (e.g., `opencrust mycommand [args]`).

### How It Works

CLI commands are parsed in `main.rs` and routed to handlers in various modules.

### Step-by-Step

**1. Modify `main.rs` to add your subcommand**

```rust
// In main.rs
#[derive(Parser)]
#[command(name = "opencrust")]
enum Commands {
    #[command(about = "My new command")]
    MyCommand {
        #[arg(help = "Input file")]
        input: String,
        
        #[arg(short, long, help = "Output format")]
        format: Option<String>,
    },
}

// In main() or run() function:
Commands::MyCommand { input, format } => {
    handle_my_command(&input, format)?;
}
```

**2. Implement the handler**

Create a new module or add to existing:

```rust
// In a new module, e.g., commands/my_command.rs
pub fn handle_my_command(input: &str, format: Option<String>) -> Result<()> {
    // Your logic here
    println!("Processing: {}", input);
    Ok(())
}
```

**3. Test it**

```bash
cargo build
./target/debug/opencrust mycommand --help
./target/debug/opencrust mycommand test.txt --format json
```

### Accepted Pattern

```bash
opencrust <COMMAND> [OPTIONS] [ARGS]
```

Common commands:
- `opencrust session list`
- `opencrust desktop detect`
- `opencrust mcp install github`
- `opencrust skills activate rust-expert`

---

## 5. Adding a TUI Component

**When to use:** You want a new UI view or interactive widget.

### How It Works

TUI components are built with Ratatui. They're rendered in `ui.rs` and receive input events through the event loop.

### Step-by-Step

**1. Create a component module**

```rust
// In ui.rs or a new file
pub struct MyWidget {
    title: String,
    items: Vec<String>,
    selected: usize,
}

impl MyWidget {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: vec![],
            selected: 0,
        }
    }
    
    pub fn render(&self, area: Rect) -> Widget {
        // Ratatui rendering logic
        // Use Block::default() for borders
        // Use List, Table, Paragraph, etc. for content
    }
    
    pub fn handle_input(&mut self, key: KeyEvent) {
        // Respond to user input
        match key.code {
            KeyCode::Up => if self.selected > 0 { self.selected -= 1; },
            KeyCode::Down => if self.selected < self.items.len() - 1 { self.selected += 1; },
            _ => {}
        }
    }
}
```

**2. Register in `ui.rs`**

Add rendering and input handling:

```rust
// In render function
match app.current_tab {
    Tab::MyTab => {
        my_widget.render(main_area)?;
    }
}

// In input handling
if let Some(key) = get_user_input() {
    my_widget.handle_input(key);
}
```

**3. Connect to app state**

Store widget in `app.rs`:

```rust
pub struct App {
    pub my_widget: MyWidget,
    // ... other fields
}
```

### Ratatui Resources

- [Ratatui examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [Common widgets](https://docs.rs/ratatui/latest/ratatui/widgets/)

---

## 6. Adding a Configuration Option

**When to use:** You want users to customize a new behavior via `config.json`.

### How It Works

Config is loaded in `config.rs` and parsed into `Config` struct. All options should have sensible defaults.

### Step-by-Step

**1. Add to `config.rs` struct**

```rust
#[derive(Deserialize)]
pub struct Config {
    // ... existing fields
    
    #[serde(default)]
    pub my_new_option: MyNewOption,
}

#[derive(Deserialize)]
pub struct MyNewOption {
    pub enabled: bool,
    pub timeout_ms: u64,
}

impl Default for MyNewOption {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 5000,
        }
    }
}
```

**2. Use in code**

```rust
// Anywhere you need the config
if config.my_new_option.enabled {
    // Custom behavior
}
```

**3. Update config schema documentation**

In `docs/CONFIGURATION.md`, add:

```markdown
### my_new_option
- `enabled` (bool, default: true): Enable the feature
- `timeout_ms` (number, default: 5000): Timeout in milliseconds
```

**4. Test with example config**

```json
{
  "my_new_option": {
    "enabled": false,
    "timeout_ms": 10000
  }
}
```

### Defaults Principle

Always provide sensible defaults. Users should have a working config without customization.

---

## 7. Adding LSP (Language Server) Support

**When to use:** You want to support a new programming language with completion, diagnostics, formatting.

### How It Works

LSP servers provide language features. OpenCrust communicates via JSON-RPC. Configuration maps language→server.

### Step-by-Step

**1. Configure in config.json**

```json
{
  "lsp": {
    "my_language": {
      "command": ["my-language-lsp"],
      "extensions": ["ml", "mli"],
      "disabled": false
    }
  }
}
```

**2. Restart and test**

```bash
opencrust
# Create a test file: test.ml
# Edit it → should see completions (Ctrl+Space)
```

**3. Update docs**

In `docs/CONFIGURATION.md`, add:

```markdown
### my_language
- `command`: [array] Command to start the LSP server
- `extensions`: [array] File extensions for this language
- `disabled`: [bool] Whether to disable this LSP
```

### Common LSP Servers

- **Rust:** `rust-analyzer`
- **Python:** `pyright` or `pylsp`
- **JavaScript:** `typescript-language-server`
- **Go:** `gopls`

### Testing LSP

1. Edit a file with the extension
2. Type code → completions should appear
3. Introduce syntax error → should show diagnostic
4. Press Ctrl+F (format) → should format document

---

## Checklist: Before Submitting Your Extension

For **any** extension (tool, skill, command, component, config, LSP):

- [ ] Code follows **AGENTS.md** conventions
- [ ] Tests written for non-trivial logic
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] Documentation added/updated in **docs/**
- [ ] Example or walkthrough in **docs/EXAMPLES.md**
- [ ] Commit message clear and follows phase convention
- [ ] No unintended changes included
- [ ] Branch up to date with main

---

## Next Steps

- **For practical tutorials:** See **docs/EXAMPLES.md**
- **For architecture context:** See **docs/ARCHITECTURE.md**
- **For coding standards:** See **AGENTS.md**
- **For module structure:** See **docs/MODULES.md**
- **For security considerations:** See **docs/SECURITY.md**

---

## Getting Help

- **Questions?** Open a GitHub Discussion
- **Stuck?** Ask in PR comments
- **Bug or unexpected behavior?** File an Issue
- **Want to chat?** Join community Discord

Happy extending! 🦀
