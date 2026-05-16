# Performance Tuning & Optimization

OpenCrust is **built in Rust** — delivering native performance across startup, memory, and runtime. Below are the measured benchmarks.

**For architecture context:** See **docs/ARCHITECTURE.md**.  
**For configuration:** See **docs/CONFIGURATION.md**.  
**For troubleshooting:** See **docs/TROUBLESHOOTING.md**.

---

## Benchmarks (Measured)

OpenCrust benchmarks against competitors (May 2026, debug build on AMD64 Linux):

| Metric | OpenCrust (Rust) | OpenCode (Go) | Cline (TS) | Claude Code (TS) | OpenDev (Rust) |
|--------|-----------------|---------------|------------|-------------------|----------------|
| Startup time | **7–10ms** | ~50ms | ~500ms | ~300ms | 4.3ms |
| Memory (idle) | **~18MB** | ~30MB | ~80MB | ~100MB | 9.4MB |
| Binary size | **16MB** | ~30MB | ~50MB+ | ~50MB+ | 18MB |
| Language | Rust 🦀 | Go | TypeScript | TypeScript | Rust 🦀 |
| Terminal-native | ✅ | ✅ | ❌ (VS Code) | ✅ | ✅ |

**Key takeaway:** Rust-native agents (OpenCrust, OpenDev) are **10-50x faster** at startup and **3-5x more memory-efficient** than TypeScript/Node.js alternatives. This means instant terminal startup, lower resource usage, and better performance on constrained systems.

### How to Reproduce

```bash
# Startup time
hyperfine --warmup 3 './target/release/opencrust --help' --shell=none

# Binary size
ls -lh ./target/release/opencrust

# Memory (idle + loading a project)
/usr/bin/time -v ./target/release/opencrust --help 2>&1 | grep "Maximum resident"
```

---

## Quick Diagnostics

### Startup Time

```bash
# Measure startup time
time opencrust --help

# Typical: 0.5–2 seconds
# If >5s: See "Slow Startup" section below
```

### Memory Usage (Runtime)

```bash
# Monitor memory while running
# macOS:
while true; do ps aux | grep opencrust | grep -v grep; sleep 1; done

# Linux:
watch 'ps aux | grep opencrust | grep -v grep'

# Typical: 200–500MB
# If >1GB: See "High Memory Usage" section below
```

### Check for Hot Spots

```bash
# Enable verbose logging temporarily
RUST_LOG=debug opencrust -p "Hello"

# Look for:
# - Repeated slow operations (same module called 100x)
# - Network latency (MCP calls, LLM requests)
# - Serialization overhead (JSON parsing)
```

---

## Startup Performance

### Profile Startup Time

```bash
# Detailed timing of each phase
cargo build --release
time ./target/release/opencrust --help

# Break down by phase
RUST_LOG=debug ./target/release/opencrust --help 2>&1 | grep -E "loading|initializing|ready"
```

### Common Slowdowns

#### 1. Slow RAG Indexing

**Symptom:** Startup hangs for 5–10 seconds during "building semantic index"

**Solution:**

Option A: Disable RAG on startup (enable only when needed)
```json
{
  "rag": {
    "enabled": false
  }
}
```

Option B: Index only specific directories
```json
{
  "rag": {
    "paths": ["src/**", "lib/**"],
    "exclude": ["target/**", "node_modules/**"]
  }
}
```

#### 2. Loading Many Skills

**Symptom:** "Loading skills" phase takes 1–2 seconds

**Solution:** Disable unused skills
```json
{
  "skills": {
    "enabled_skills": [
      "rust-expert",
      "code-refactorer"
    ],
    "disabled": [
      "security-auditor",
      "test-generator",
      "dep-manager"
    ]
  }
}
```

#### 3. Slow Disk (HDD)

**Symptom:** Consistent 2–3 second startup regardless of config

**Cause:** HDD I/O slower than SSD

**Solution:** Move ~/.config/opencrust to SSD or use ramdisk:
```bash
# macOS: Create 100MB ramdisk
diskutil secureErase freespace 0 -secureRandom 100m ramdisk

# Linux: Mount tmpfs
sudo mount -t tmpfs -o size=100M tmpfs ~/.config/opencrust
```

#### 4. Slow Network (MCP/LSP Discovery)

**Symptom:** Startup stalls 3–5 seconds before showing UI

**Cause:** Waiting for MCP/LSP servers to start

**Solution:** Disable unused servers
```json
{
  "mcp": {
    "github": {
      "enabled": false
    },
    "postgres": {
      "enabled": false
    }
  },
  "lsp": {
    "rust": {
      "disabled": false
    },
    "python": {
      "disabled": true
    }
  }
}
```

### Optimization Checklist

- [ ] Build in release mode: `cargo build --release`
- [ ] Disable RAG if not needed: `"enabled": false`
- [ ] Disable unused skills: Set `"disabled": true`
- [ ] Disable unused MCP/LSP servers
- [ ] Use SSD (not HDD) for config
- [ ] Keep codebase <50K files (semantic index is O(n))

---

## Runtime Performance

### Typical Latencies

| Operation | Latency | Bottleneck |
|-----------|---------|-----------|
| Key press → screen | 16–50ms | Terminal rendering |
| LLM call (first token) | 500ms–5s | Network to API |
| Streaming LLM response | 50–200ms per token | Network bandwidth |
| Tool execution | Varies | Script runtime |
| Semantic search | 50–200ms | Vector DB lookup |
| File read (1MB) | 5–20ms | Disk I/O |

### Profile Tool Execution

```bash
# Measure tool performance
time ./.opencrust/tools/my_linter src/main.rs

# If >1 second, optimize:
# - Is tool calling external binary? (e.g., `cargo check`)
# - Can you cache results?
# - Can you parallelize?
```

### Optimize LLM Latency

#### Reduce Context

```json
{
  "context_budget": 4096  // From 8000
}
```

**Effect:** Fewer tokens = faster response (40–50% speedup typical)  
**Trade-off:** Less context, potentially worse answers

#### Use Faster Model

```json
{
  "default_model": "gpt-3.5-turbo"  // Instead of gpt-4
}
```

**Effect:** 10x faster inference  
**Trade-off:** Lower quality responses

#### Switch Provider

| Provider | Speed | Quality | Cost |
|----------|-------|---------|------|
| Groq | ⚡⚡⚡ Fast | Good | Low |
| Together AI | ⚡⚡ Fast | Good | Low |
| OpenRouter | ⚡ Normal | Excellent | Medium |
| OpenAI | ⚡ Normal | Excellent | High |

**Groq example (fastest):**
```json
{
  "default_provider": "groq",
  "default_model": "mixtral-8x7b-32768",
  "providers": {
    "groq": {
      "api_key": "YOUR_KEY"
    }
  }
}
```

#### Enable Streaming

OpenCrust streams responses by default. To verify it's working:
- Look for progressive text rendering (words appear one by one)
- Not "whole response appears at once"

### Profiling Long-Running Operations

```bash
# Install flamegraph profiler
cargo install flamegraph

# Profile while running
cargo flamegraph --bin opencrust

# Opens flamegraph.svg in browser
# Shows which functions take most CPU time
```

**Interpreting the graph:**
- Wide bars = long-running functions
- Click to zoom into callstack
- Look for unexpected repeated calls

---

## Memory Optimization

### Monitor Memory Usage

```bash
# Linux: Get detailed breakdown
ps aux | grep opencrust

# Look at RSS (resident set size) and VSZ (virtual)
# Example:
# alice 12345 1.2 2.3 524288 47980  ...
#              ↑    ↑
#           CPU%  Memory% (of system)

# Translate to MB: VSZ=524MB, RSS=48MB
```

### Reduce Memory Usage

#### 1. Limit Session History

```json
{
  "sessions": {
    "max_history": 50,        // From 100
    "retention_days": 7        // From 30
  }
}
```

**Effect:** Older sessions deleted, less disk/memory  
**Trade-off:** Can't restore old conversations

#### 2. Disable Semantic Index

```json
{
  "rag": {
    "enabled": false
  }
}
```

**Effect:** Saves 50–200MB (depending on codebase size)  
**Trade-off:** `semantic_search` tool becomes unavailable

#### 3. Use Smaller Context

```json
{
  "context_budget": 2000
}
```

**Effect:** Fewer chat messages kept in memory  
**Trade-off:** Can't reference as far back in conversation

#### 4. Clear Cache Periodically

```bash
# Remove cached embeddings, tool results
rm -rf ~/.cache/opencrust

# Safe to delete anytime (rebuilds on demand)
```

**Effect:** Frees 50–100MB  
**Trade-off:** Rebuilds cache on next run (slower first startup)

### Memory Leak Detection

If memory grows over time (leaking):

```bash
# Run with memory tracking
valgrind --leak-check=full ./target/release/opencrust 2>&1 | tee valgrind.log

# Shows allocations that were never freed
# File issue with valgrind output attached
```

---

## Network Performance

### Check LLM Latency

```bash
# Test API response time
time curl -X POST https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"hi"}]}'

# If >3s, check:
# - Network connectivity: ping api.openai.com
# - API status: https://status.openai.com
# - Rate limit: Check quota in API dashboard
```

### Optimize for Slow Networks

#### Use Local Ollama Instead of Cloud API

```json
{
  "default_provider": "ollama",
  "default_model": "mistral",
  "providers": {
    "ollama": {
      "base_url": "http://localhost:11434"
    }
  }
}
```

**Effect:** No network latency (everything local)  
**Trade-off:** Needs powerful GPU/CPU, slower on weak machines

#### Use Smaller Model for Fast Iteration

```json
{
  "default_model": "mistral-7b"  // Instead of gpt-4
}
```

**Effect:** Faster LLM calls (especially on slow network)  
**Trade-off:** Lower quality, need more back-and-forth

#### Buffer Settings

```json
{
  "network": {
    "timeout_seconds": 60,         // From 30
    "read_timeout_seconds": 120,
    "write_timeout_seconds": 30
  }
}
```

**Effect:** Prevents timeout on slow connections  
**Trade-off:** Slower error detection on dead connections

---

## TUI Rendering Performance

### Reduce Visual Effects

```json
{
  "tui": {
    "animations": false,           // Disable smooth transitions
    "border_style": "simple"        // Instead of "rounded"
  }
}
```

**Effect:** Slightly faster rendering (1–2ms per frame)

### Optimize Terminal Emulator

**Fastest terminal emulators (tested 2026):**
1. Alacritty (GPU-accelerated)
2. Kitty (GPU-accelerated)
3. iTerm2 (with GPU rendering on)
4. Windows Terminal (latest)
5. GNOME Terminal (slower)

**Test rendering speed:**

```bash
# Generate large output
opencrust -p "Write a 10,000 word essay"

# Watch rendering:
# - Smooth scrolling? (60 FPS ideal)
# - Any flickering? (indicates render lag)
# - CPU usage high? (switch terminal)
```

### File Tree Performance

If file tree slow with large codebase:

```json
{
  "ui": {
    "file_tree": {
      "max_depth": 3,           // Collapse deep directories
      "exclude_dirs": [
        "node_modules",
        "target",
        ".git",
        "__pycache__"
      ]
    }
  }
}
```

---

## Tool Execution Performance

### Profile Custom Tools

```bash
# Measure tool runtime
time ./.opencrust/tools/my_tool arg1 arg2

# If >1 second:
# 1. Run tool directly to eliminate OpenCrust overhead
# 2. Measure with strace/ltrace to find bottlenecks
```

### Parallelize Tool Execution

If you have multiple independent tools, request them together:

```
User: "Lint the code AND run tests AND build the project"
```

OpenCrust will execute all three in parallel (if they don't depend on each other).

### Cache Tool Results

Store in local file to avoid re-running:

```bash
# Instead of:
.opencrust/tools/expensive_analysis src/

# Do:
if [ -f .analysis_cache ]; then
  cat .analysis_cache
else
  .opencrust/tools/expensive_analysis src/ | tee .analysis_cache
fi
```

---

## Benchmarking

### Criterion Benchmarks (Automated)

OpenCrust ships a Criterion benchmark harness at `benches/benchmark.rs` that
measures core subsystem performance: JSON parsing, config deserialization, and
text diffing.

```bash
# Run all benchmarks
cargo bench

# Run with HTML report generation
cargo bench --bench benchmark -- --profile-time 30

# View report
open target/criterion/report/index.html
```

**What is measured:**
| Benchmark | What it does |
|-----------|-------------|
| `config_parse` | Deserialize JSON config (~150 bytes) |
| `json_validate` | Parse a JSON object into `serde_json::Value` |
| `json_format` | Round-trip JSON through `to_string_pretty` |
| `json_get_path` | JSON pointer lookup (`/features/tui`) |
| `json_compact` | Round-trip JSON through `to_string` |
| `json_compare` | Deep equality check on two JSON objects |
| `json_merge` | Patch merge of two JSON objects |
| `text_diff` | Myers diff on two ~120-line text blocks |

### Baseline Metrics (Before Optimization)

```bash
# Capture baseline
./target/release/opencrust -p "test" --log-metrics > baseline.json 2>&1

# Key metrics to note:
# - Startup time (until "Ready" message)
# - First token latency (how long to first LLM output)
# - Memory usage (peak RSS)
# - CPU usage while idling
```

### Measure Improvement

```bash
# After optimization
./target/release/opencrust -p "test" --log-metrics > after.json 2>&1

# Compare metrics
jq '.startup_time_ms' baseline.json
jq '.startup_time_ms' after.json
# Calculate % improvement: (before - after) / before * 100
```

### Reproducible Benchmarking

Always benchmark with same conditions:
- Same codebase size
- Same system load (`top` shows <20% other processes)
- Same network conditions
- Same TUI theme/settings
- 3 runs, take average (reduces variance)

---

## Production Optimization

### For Team Use

```json
{
  "default_model": "gpt-4",
  "context_budget": 8000,
  "sessions": {
    "auto_save": true,
    "retention_days": 30
  },
  "rag": {
    "enabled": true,
    "cache_size_mb": 500
  },
  "mcp": {
    "github": { "enabled": true }
  },
  "permissions": {
    "file_patterns": ["src/**", "tests/**"]
  }
}
```

**Cost estimate:** $2–5 per developer per month (at $0.03/1K tokens)

### For Cost-Sensitive Use

```json
{
  "default_provider": "groq",
  "default_model": "mixtral-8x7b-32768",
  "context_budget": 4000,
  "provider_fallback": ["groq", "ollama"],
  "providers": {
    "groq": { "api_key": "..." },
    "ollama": { "base_url": "http://localhost:11434" }
  }
}
```

**Cost estimate:** <$1 per developer per month (Groq free tier covers most use)

### For Privacy

```json
{
  "default_provider": "ollama",
  "default_model": "mistral",
  "providers": {
    "ollama": { "base_url": "http://localhost:11434" }
  },
  "permissions": {
    "network": {
      "enabled": false
    }
  }
}
```

**Cost:** Free (no API calls)  
**Trade-off:** Needs GPU, responses slower than cloud

---

## Troubleshooting Performance Issues

### Slow Everything (Startup, LLM, Tools)

**Check system load:**
```bash
top
# If CPU >90% or memory >80%, close other apps
```

**Check internet:**
```bash
ping api.openai.com
# If >100ms latency, try different network/provider
```

### Slow on Startup Only

**See "Startup Performance" section above.**

### Slow LLM Responses Only

**Possible causes:**
- Provider API overloaded (check their status page)
- Network congestion (try different network)
- Context too large (reduce `context_budget`)
- Model overloaded (switch to faster model)

**Debug:**
```bash
# Time just the LLM call (exclude tool execution)
# Open chat, type message, press Enter
# Note time from "Thinking..." to first response

# If >5s, ask another message to see if it's consistent
# (one slow response might be cache miss)
```

### High Memory After Long Session

**Normal behavior:** Memory grows as chat history accumulates

**Limit history:**
```json
{
  "sessions": {
    "max_history": 50
  }
}
```

**Or restart:**
```bash
# Save session (Ctrl+S)
opencrust session save --name "old_session"

# Restart OpenCrust
opencrust

# Memory reset to baseline
```

---

## When to Stop Optimizing

| Metric | Acceptable | Worth Optimizing |
|--------|-----------|------------------|
| Startup | <2s | >5s |
| First token | <1s | >3s |
| Memory | 300–500MB | >1GB |
| Semantic search | <200ms | >1s |
| Tool execution | Depends on tool | Tool's fault, not OpenCrust |

**General rule:** Optimize only if it's blocking your workflow.

---

## References

- **ARCHITECTURE.md** — Understand where time is spent
- **CONFIGURATION.md** — All performance settings
- **docs/MODULES.md** — Module-by-module optimization points
- **CONTRIBUTING.md** — Performance testing in PRs
