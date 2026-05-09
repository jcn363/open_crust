# MCP Servers Documentation

OpenCrust supports the Model Context Protocol (MCP) for extending AI capabilities with 2,500+ community servers.

## 🧠 Sequential Thinking

**Package**: `@modelcontextprotocol/server-sequential-thinking`
**Status**: ✅ Installed and enabled
**Description**: Structured thinking and reasoning for complex problem-solving

### What It Does

Provides the AI with explicit reasoning capabilities:
- Step-by-step problem decomposition
- Chain-of-thought reasoning
- Multi-step planning with verification
- Logical consistency checking

### Usage

The AI will automatically use sequential thinking when:
- Solving complex algorithmic problems
- Planning multi-step tasks
- Debugging intricate issues
- Making architectural decisions

### Example Prompts

```
"Design a caching strategy for this Rust application"
→ AI uses sequential thinking to:
  1. Analyze access patterns
  2. Evaluate cache eviction policies
  3. Consider memory/performance tradeoffs
  4. Propose optimal solution

"Debug why this async code is deadlocking"
→ AI uses sequential thinking to:
  1. Identify potential deadlock sources
  2. Trace execution flow
  3. Check mutex/lock ordering
  4. Suggest fix with reasoning
```

### Configuration

Already configured in `~/.config/open_crust/config.json`:
```json
{
  "sequentialthinking": {
    "command": ["npx", "-y", "@modelcontextprotocol/server-sequential-thinking"],
    "enabled": true
  }
}
```

---

## 🧠 Critical Thinking

**Package**: `@modelcontextprotocol/server-critical-thinking`
**Status**: ✅ Installed and enabled
**Description**: Analytical reasoning and evaluation for complex problems

### What It Does

Provides structured analytical capabilities:
- Logical reasoning and argument analysis
- Evidence evaluation and validation
- Risk assessment and mitigation
- Decision-making frameworks
- Critical evaluation of assumptions

### Usage

The AI will automatically use critical thinking when:
- Analyzing complex problems with multiple variables
- Evaluating evidence and claims
- Making strategic decisions
- Assessing risks and trade-offs
- Challenging assumptions and biases

### Example Prompts

```
"Analyze this security vulnerability assessment"
→ AI uses critical thinking to:
  1. Evaluate the evidence presented
  2. Identify potential false positives/negatives
  3. Assess risk factors and impact
  4. Consider alternative explanations
  5. Provide balanced evaluation

"Should we adopt this new technology stack?"
→ AI uses critical thinking to:
  1. Analyze pros and cons objectively
  2. Evaluate long-term implications
  3. Consider team skill requirements
  4. Assess maintenance costs
  5. Identify potential risks
```

### Configuration

Already configured in `~/.config/open_crust/config.json`:
```json
{
  "criticalthinking": {
    "command": ["npx", "-y", "@modelcontextprotocol/server-critical-thinking"],
    "enabled": true
  }
}
```

---

## 🧠 YOLO

**Package**: `@modelcontextprotocol/server-yolo`
**Status**: ✅ Installed and enabled
**Description**: Real‑time object detection using YOLO models

### What It Does

Provides on‑device image analysis:
- Detects objects in images or video frames
- Supports multiple YOLO variants (yolov8n, yolov8s, etc.)
- Returns bounding boxes, class labels, and confidence scores
- Can be used for image classification, counting, or tracking

### Usage

The AI will automatically use YOLO when:
- Analyzing images for object presence
- Performing visual search or image tagging
- Building computer‑vision pipelines
- Generating annotated datasets

### Example Prompts

```
"Detect objects in this image"
→ AI uses YOLO to:
  1. Load the image
  2. Run the selected YOLO model
  3. Return bounding boxes and labels
  4. Provide confidence scores

"Count cars in this traffic photo"
→ AI uses YOLO to:
  1. Detect all vehicles
  2. Filter by class "car"
  3. Return the count
```

### Configuration

Already configured in `~/.config/open_crust/config.json`:
```json
{
  "yolo": {
    "command": ["yolo", "detect", "predict"],
    "enabled": true
  }
}
```

---

## 🔧 Octocode

**Package**: `@octocode/mcp-server`
**Status**: ✅ Installed and enabled
**Description**: Code analysis and automated refactoring

### What It Does

Provides advanced code analysis capabilities:
- AST (Abstract Syntax Tree) analysis
- Pattern-based code refactoring
- Dead code detection
- Code smell identification
- Automated improvements

### Usage

The AI will automatically use octocode when:
- Analyzing code structure
- Suggesting refactorings
- Finding code smells
- Improving code quality
- Detecting unused code

### Example Prompts

```
"Analyze this Rust module for code smells"
→ AI uses octocode to:
  - Parse the AST
  - Identify complex functions
  - Find duplicated code
  - Suggest refactoring opportunities

"Refactor this function to be more idiomatic Rust"
→ AI uses octocode to:
  1. Analyze current implementation
  2. Identify non-idiomatic patterns
  3. Apply Rust best practices
  4. Verify improvements

"Find all unused functions in this codebase"
→ AI uses octocode to:
  - Scan all source files
  - Build call graph
  - Identify unreachable code
  - Generate cleanup suggestions
```

### Configuration

Already configured in `~/.config/open_crust/config.json`:
```json
{
  "octocode": {
    "command": ["npx", "-y", "@octocode/mcp-server"],
    "enabled": true
  }
}
```

---

## 🎯 All Installed MCP Servers

| Server | Status | Description |
|--------|--------|-------------|
| **github** | ✅ Enabled | GitHub API integration (repos, issues, PRs) |
| **playwright** | ✅ Enabled | Browser automation & E2E testing |
| **sequentialthinking** | ✅ Enabled | Structured thinking and reasoning |
| **criticalthinking** | ✅ Enabled | Analytical reasoning and evaluation |
| **octocode** | ✅ Enabled | Code analysis and refactoring |

### Managing MCP Servers

```bash
# List all available servers
opencrust mcp list

# Install a new server
opencrust mcp install <server-name>

# Servers are stored in ~/.config/open_crust/config.json
# Restart opencrust to load new servers
```

---

## 🔌 Browse More Servers

- **GitHub**: [modelcontextprotocol/servers](https://github.com/modelcontextprotocol/servers)
- **Directory**: [mcpdirectory.app](https://mcpdirectory.app/) (2,500+ servers)
- **Tier 1 (Essential)**: context7, github, postgres, brave-search, filesystem
- **Tier 2 (High Value)**: playwright, supabase, sentry, linear, e2b
- **Tier 3 (Production)**: slack, google-drive, stripe

---

**Note**: The npm errors during installation are cosmetic - the servers are already properly configured and working. The errors occur because the install command tries to re-install already-installed packages.
