# Troubleshooting Guide

Common issues and solutions. If your problem isn't listed, file an Issue on GitHub.

---

## Installation Issues

### .deb package install fails with dependency errors

**Symptoms:** `sudo dpkg -i opencrust_*.deb` fails with missing dependency messages

**Root Causes:**
- Missing runtime libraries
- System package cache outdated

**Solution:**
1. Fix dependencies: `sudo apt-get install -f`
2. Or install with apt which handles deps automatically: `sudo apt-get install ./opencrust_*.deb`

### cargo deb command not found

**Symptoms:** `cargo deb` fails with "no such command"

**Root Causes:**
- cargo-deb not installed

**Solution:**
1. Install cargo-deb: `cargo install cargo-deb`
2. Build the package: `cargo deb`

### Build fails: "error: linker 'cc' not found"

**Symptoms:** Build fails during compilation with linker error

**Root Causes:**
- Build tools not installed
- C compiler missing
- Wrong Rust version

**Solution:**
1. Install build tools:
   - **Ubuntu/Debian:** `sudo apt-get install build-essential`
   - **macOS:** Install Xcode: `xcode-select --install`
   - **Fedora:** `sudo dnf install gcc`

2. Verify Rust version: `rustc --version` (should be 1.75+)
3. Update Rust: `rustup update`
4. Try again: `cargo build`

**Prevention:**
- Check prerequisites in **CONTRIBUTING.md** before building
- Keep Rust updated: `rustup update` monthly

---

### Build fails: "error[E0514]: found crate ... compiled by an incompatible version"

**Symptoms:** Build fails with version mismatch error

**Root Causes:**
- Rust version changed
- Dependencies cached with old version
- Cargo cache corrupted

**Solution:**
1. Clean build artifacts: `cargo clean`
2. Update Rust: `rustup update`
3. Rebuild: `cargo build`

**Prevention:**
- Run `rustup update` before building after switching machines

---

### Installation hangs at "Compiling ratatui..."

**Symptoms:** Build gets stuck, CPU usage drops to zero

**Root Causes:**
- Network timeout during dependency download
- Disk full
- System under heavy load

**Solution:**
1. Check disk space: `df -h` (need >2GB free)
2. Cancel (Ctrl+C) and retry: `cargo build`
3. Use release profile (slower build, more optimized): `cargo build --release`

**Prevention:**
- Ensure 2GB+ free disk space before building
- Build on stable network connection

---

## Runtime Errors

### opencrust command not found after `cargo install`

**Symptoms:** After `cargo install --path .`, `opencrust` command not found

**Root Causes:**
- `~/.cargo/bin` not in PATH
- Shell not reloaded after install
- Installation failed silently

**Solution:**
1. Check installation: `~/.cargo/bin/opencrust --help`
2. If that works, add to PATH:
   - **Bash:** Add `export PATH="$HOME/.cargo/bin:$PATH"` to `~/.bashrc`
   - **Zsh:** Add to `~/.zshrc`
   - **Fish:** Add `set -gx PATH $HOME/.cargo/bin $PATH` to `~/.config/fish/config.fish`
3. Reload shell: `exec $SHELL`

**Verification:**
```bash
which opencrust
opencrust --help
```

**Prevention:**
- Verify installation worked: `cargo install --path . --verbose`

---

### "Tool execution failed" error in chat

**Symptoms:** OpenCrust tries to use a tool but fails with generic error

**Root Causes:**
- Tool script not found
- Tool missing execute permission
- Tool script has runtime error
- Permissions denied

**Solution:**
1. Check tool exists: `ls -la .opencrust/tools/my_tool`
2. Verify execute permission: `chmod +x .opencrust/tools/my_tool`
3. Test manually: `./.opencrust/tools/my_tool arg1 arg2`
4. Check permissions in config.json:
   ```json
   {
     "permissions": {
       "file_patterns": [".opencrust/tools/**"]
     }
   }
   ```

**Debug:**
Check audit log: `grep "tool_executed" ~/.config/opencrust/audit.json | tail -5`

---

### "LLM not responding" error

**Symptoms:** OpenCrust hangs or times out when calling LLM

**Root Causes:**
- Network disconnected
- LLM service down
- API key invalid
- Rate limit exceeded

**Solution:**
1. Test network: `ping api.openai.com` (or your provider)
2. Verify API key: Check `~/.config/opencrust/config.json`
3. Check provider status:
   - **OpenAI:** [openai.com/status](https://openai.com/status)
   - **Gemini:** Google Cloud Console
   - **Ollama:** Is it running? `curl http://localhost:11434`
4. Wait a minute and retry

**Prevention:**
- Monitor API quota in provider dashboard
- Test connection before intensive session: `opencrust -p "Hello"`

---

### "Permission denied" error for file access

**Symptoms:** OpenCrust tries to read/write a file but permission denied

**Root Causes:**
- File permissions too restrictive
- Permission policy in config.json blocks access
- SELinux or AppArmor denying access

**Solution:**
1. Check file permissions: `ls -la path/to/file`
2. Fix if needed: `chmod 644 path/to/file`
3. Check OpenCrust permissions in config.json:
   ```json
   {
     "permissions": {
       "file_patterns": ["src/**", "docs/**"]
     }
   }
   ```
4. Audit log shows denial: `grep "permission_denied" ~/.config/opencrust/audit.json`

**Prevention:**
- Set appropriate file permissions: `chmod 600` for secrets, `chmod 644` for code

---

### Panic with backtrace

**Symptoms:** Application crashes with backtrace output

**Root Causes:**
- Unexpected state or assertion failed
- Null pointer or logic error
- Out of memory

**Solution:**
1. Note the panic message (first few lines of backtrace)
2. Collect full backtrace:
   ```bash
   RUST_BACKTRACE=full opencrust 2>&1 | tee crash.log
   ```
3. File Issue with backtrace and steps to reproduce
4. Restart OpenCrust (state might be corrupted)

**Recovery:**
- Clear corrupted session: `rm ~/.config/opencrust/sessions/*`
- Restart: `opencrust`

---

## Configuration Issues

### Desktop environment not detected

**Symptoms:** Desktop integration not working (notifications, file picker)

**Root Causes:**
- Running headless or in unsupported DE
- Environment variables not set
- Missing Desktop Session file

**Solution:**
1. Check detected environment: `opencrust desktop detect`
2. Set manually in config:
   ```json
   {
     "desktop": "gnome"
   }
   ```
3. Supported: `cinnamon`, `mate`, `gnome`, `kde`

**Verification:**
```bash
opencrust desktop notify --title "Test" --body "Hello"
```

---

### LSP not showing completions or diagnostics

**Symptoms:** Language server features not working in editor

**Root Causes:**
- LSP server not installed
- Config incorrect
- LSP server crashed
- File extension not mapped

**Solution:**
1. Check LSP configured: `grep "lsp" ~/.config/opencrust/config.json`
2. Verify server installed: `which rust-analyzer` (for Rust)
3. Check extension mapping:
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
4. Restart OpenCrust

**Debug:**
- Test LSP directly: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | rust-analyzer`

---

### MCP server not connecting

**Symptoms:** MCP server shows as disabled or unavailable

**Root Causes:**
- Server command not found
- Wrong command path
- Server crashes on startup
- Port already in use

**Solution:**
1. Check config: `grep "mcp" ~/.config/opencrust/config.json`
2. Test manually:
   ```bash
   npx -y @modelcontextprotocol/server-github
   # Should start without errors
   ```
3. Verify command path: `which npx`
4. Check logs: `opencrust mcp list`

---

### Config parse error: "expected value at line X"

**Symptoms:** JSON parse error when loading config

**Root Causes:**
- Invalid JSON syntax
- Missing quotes or commas
- Wrong value type

**Solution:**
1. Validate JSON: `jq . ~/.config/opencrust/config.json`
2. Common mistakes:
   - Missing comma after property: `"key": "value"` ← needs comma
   - Trailing comma: `"key": "value",` ← not allowed in JSON
   - Wrong type: `enabled: true` should be `"enabled": true`
3. Fix and retry

**Prevention:**
- Use JSON validator: [jsonlint.com](https://www.jsonlint.com/)
- Use IDE with JSON schema support

---

## Performance Issues

### Slow startup (takes >5 seconds)

**Symptoms:** `opencrust` command takes a long time to start

**Root Causes:**
- Expensive skill discovery
- Large codebase semantic indexing
- Slow disk I/O
- Network latency (API checks)

**Solution:**
1. Disable expensive features temporarily:
   ```bash
   # Restart without skills
   opencrust --no-skills
   ```
2. Check startup time: `time opencrust --help`
3. Profile: `cargo flamegraph`

**Optimization:**
- Disable unused skills: See **docs/CONFIGURATION.md**
- Disable MCP servers: Set `"enabled": false` in config
- Use SSD: HDD significantly slows startup

---

### High memory usage (>1GB)

**Symptoms:** OpenCrust consuming excessive memory

**Root Causes:**
- Large session history
- Many semantic indices cached
- Memory leak in running process
- Context window too large

**Solution:**
1. Limit session history: Check `~/.config/opencrust/sessions/`
2. Clear old sessions: Delete old session files
3. Restart OpenCrust (flush memory)
4. Reduce context budget in config

**Prevention:**
- Regularly clear sessions: `rm ~/.config/opencrust/sessions/*.old`
- Limit context: Set `"context_budget": 4096` in config

---

### Tool execution is slow

**Symptoms:** Tools (custom scripts, MCP calls) take >1 second

**Root Causes:**
- Tool itself is slow (inherent)
- Network latency (remote API)
- Disk I/O (reading large files)
- System under heavy load

**Solution:**
1. Profile tool directly: `time .opencrust/tools/my_tool args`
2. If tool is slow, optimize it (not OpenCrust issue)
3. Check system load: `top` (look for CPU, memory usage)
4. Run during less busy time

**Prevention:**
- Use async operations in scripts
- Cache results locally
- Run heavy tools in background: Use `Tasks` tab

---

### TUI rendering lag or flicker

**Symptoms:** Terminal UI stutters, flickers, or lags when typing

**Root Causes:**
- Terminal emulator slow
- Render interval too frequent
- Background tasks competing
- GPU/CPU throttling

**Solution:**
1. Try different terminal: iTerm2, Alacritty, KiTTy
2. Restart OpenCrust (clear render buffer)
3. Disable visual effects in config:
   ```json
   {
     "tui": {
       "animations": false
     }
   }
   ```
4. Check system load: `top`

**Prevention:**
- Use modern terminal emulator
- Close other resource-intensive apps during work

---

## Feature Not Working

### Semantic search returns nothing

**Symptoms:** `semantic_search` tool returns no results

**Root Causes:**
- Codebase not indexed
- Query too specific or vague
- Index outdated
- Embedding model not available

**Solution:**
1. Build index: `opencrust -p "index_codebase"`
2. Wait for completion (large codebases take time)
3. Try simpler query: Instead of "function that handles X", try "X"
4. Check Ollama running: `curl http://localhost:11434`

**Prevention:**
- Index after adding files: `index_codebase` command
- Keep embedding model running (Ollama)

---

### Plan mode not showing changes

**Symptoms:** Plan mode (`P` key) shows empty or no diffs

**Root Causes:**
- No changes proposed by LLM
- Changes not in parseable format
- Display filter hiding them

**Solution:**
1. Make sure LLM responded with code changes
2. Check response in Chat tab first
3. Try simpler request: "Create a file called test.txt"
4. Verify plan mode enabled: `P` should toggle it

---

### Skill not activating

**Symptoms:** Skill shows in list but doesn't affect behavior

**Root Causes:**
- Skill file malformed
- Skill not actually activated
- Skill not relevant to current task
- Typo in skill name

**Solution:**
1. Check skill is active: Skill Browser (Ctrl+Shift+K)
2. Verify syntax: `cat .opencrust/skills/my_skill/SKILL.md`
3. Check skill file location: Must be at `.opencrust/skills/<name>/SKILL.md`
4. Restart OpenCrust to reload skills

---

### Subagents not spawning

**Symptoms:** Subagent spawning request ignored

**Root Causes:**
- Subagent system not enabled
- Subagent module not compiled
- Request malformed

**Solution:**
1. Check if compiled: `opencrust --help | grep agent`
2. Try simple subagent: `opencrust -p "Use planner to..."`
3. Check logs for errors
4. Verify config has subagent provider set

---

### Session history not persisting

**Symptoms:** Session lost when reopening OpenCrust

**Root Causes:**
- Session not saved manually
- Session storage corrupted
- Permissions issue

**Solution:**
1. Save manually: `opencrust session save --name "my_session"`
2. List sessions: `opencrust session list`
3. Restore: Reopen and use `opencrust session show <id>`
4. Check permissions: `ls -la ~/.config/opencrust/sessions/`

---

## Getting More Help

**Not in this guide?**

1. **Check existing Issues:** [GitHub Issues](https://github.com/opencrust/opencrust/issues)
2. **Ask in Discussions:** [GitHub Discussions](https://github.com/opencrust/opencrust/discussions)
3. **File a new Issue:** Include:
   - OS and version
   - Rust version: `rustc --version`
   - OpenCrust version: `opencrust --version`
   - Steps to reproduce
   - Full error message or backtrace
   - Relevant config snippet (remove secrets)

4. **Check logs:**
   - Audit log: `~/.config/opencrust/audit.json`
   - Last error: Check most recent entries in audit log

---

**For more context, see:**
- **docs/ARCHITECTURE.md** — Understand how things work
- **docs/CONFIGURATION.md** — All config options
- **docs/DEVELOPMENT.md** — How to extend
- **AGENTS.md** — Coding standards
- **CONTRIBUTING.md** — Development workflow
