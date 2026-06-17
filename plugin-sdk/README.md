# OpenCrust Plugin SDK

> **Version:** v0.1.3 | **Last updated:** 2026-06-17

This guide covers developing plugins for OpenCrust.

## Overview

OpenCrust plugins are directories containing a `plugin.json` manifest and executable scripts. Plugins can hook into the agent lifecycle, register custom tools, and extend OpenCrust's functionality.

## Plugin Manifest (plugin.json)

```json
{
    "name": "my-plugin",
    "version": "0.1.0",
    "description": "Description of what this plugin does",
    "author": "Your Name",
    "hooks": ["on_startup", "on_shutdown"],
    "tools": [
        {
            "name": "my_tool",
            "description": "What this tool does",
            "parameters": {
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "Input text"
                    }
                },
                "required": ["input"]
            }
        }
    ],
    "permissions": ["read_files"],
    "config": {}
}
```

### Manifest Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique plugin identifier |
| `version` | string | Yes | Semantic version (e.g., "0.1.0") |
| `description` | string | Yes | What the plugin does |
| `author` | string | Yes | Plugin author name |
| `hooks` | array | No | Lifecycle hooks to subscribe to |
| `tools` | array | No | Tools to register |
| `entry` | string | No | Entry script path (default: "handler.sh") |
| `permissions` | array | No | Required permissions |
| `config` | object | No | Plugin-specific configuration |
| `dependencies` | array | No | Required plugins or tools |

## Available Hooks

| Hook | When | Context |
|------|------|---------|
| `on_startup` | Plugin loaded | `{}` |
| `on_shutdown` | Plugin unloaded | `{}` |
| `on_message` | User sends message | `{message: string}` |
| `on_tool_execute` | Before tool execution | `{tool: string, input: object}` |
| `on_tool_result` | After tool execution | `{tool: string, result: object}` |
| `on_response` | LLM responds | `{response: string}` |

## Handler Script

The default entry point is `handler.sh`. It receives JSON on stdin and should output JSON on stdout.

### Input Format

```json
{
    "hook": "on_tool_execute",
    "context": {
        "tool": "bash",
        "input": {"command": "ls"}
    },
    "config": {}
}
```

### Output Format

```json
{
    "modified_input": null,
    "block": false,
    "message": "Optional message to show user"
}
```

### Handler Template

```bash
#!/bin/bash
# My OpenCrust Plugin Handler
# Receives JSON on stdin, outputs JSON on stdout

INPUT=$(cat)
HOOK=$(echo "$INPUT" | jq -r '.hook')
TOOL=$(echo "$INPUT" | jq -r '.context.tool // empty')

case "$HOOK" in
    "on_startup")
        echo '{"message": "Plugin loaded!"}'
        ;;
    "on_tool_execute")
        # Example: block dangerous commands
        if [ "$TOOL" = "bash" ]; then
            CMD=$(echo "$INPUT" | jq -r '.context.input.command // empty')
            if echo "$CMD" | grep -q "rm -rf /"; then
                echo '{"block": true, "message": "Blocked dangerous command!"}'
                exit 0
            fi
        fi
        echo '{}'
        ;;
    "on_shutdown")
        echo '{"message": "Plugin unloaded."}'
        ;;
    *)
        echo '{}'
        ;;
esac
```

## Tool Registration

Tools defined in `plugin.json` are available to the agent. The handler receives tool calls via the `on_tool_execute` hook.

### Tool Input/Output

When the agent calls your tool:

1. The `on_tool_execute` hook fires with the tool name and input
2. Your handler can modify the input or block the call
3. The tool executes (unless blocked)
4. The `on_tool_result` hook fires with the result

## Permissions

Plugins declare required permissions in the manifest:

| Permission | Description |
|------------|-------------|
| `read_files` | Read file contents |
| `write_files` | Create/modify files |
| `execute_commands` | Run shell commands |
| `network_access` | Make HTTP requests |
| `manage_mcp` | Configure MCP servers |

## Installation

### From Git Repository

```bash
opencrust plugin install https://github.com/user/plugin-repo
```

### From Local Directory

```bash
opencrust plugin install /path/to/plugin-directory
```

### From URL

```bash
opencrust plugin install https://example.com/plugin.zip
```

## Development Workflow

1. Create plugin directory:
   ```bash
   mkdir -p ~/.config/opencrust/plugins/my-plugin
   cd ~/.config/opencrust/plugins/my-plugin
   ```

2. Create `plugin.json` manifest

3. Create `handler.sh` (make executable):
   ```bash
   chmod +x handler.sh
   ```

4. Test the plugin:
   ```bash
   opencrust plugin list
   opencrust plugin show my-plugin
   opencrust plugin enable my-plugin
   ```

5. Test hooks:
   ```bash
   echo '{"hook": "on_startup"}' | ./handler.sh
   ```

## Example Plugins

See `examples/` directory for working examples:

- `basic-plugin/` - Minimal plugin with startup hook
- `command-blocker/` - Blocks dangerous bash commands
- `code-formatter/` - Auto-formats code on write

## Debugging

### Check Plugin Status

```bash
opencrust plugin stats my-plugin
```

### View Plugin Logs

Check OpenCrust's audit log for plugin execution:

```bash
tail -f ~/.config/opencrust/logs/audit.log | grep plugin
```

### Common Issues

1. **Plugin not found**: Ensure `plugin.json` exists and is valid JSON
2. **Handler not executing**: Check file is executable (`chmod +x handler.sh`)
3. **Permission denied**: Verify plugin has required permissions
4. **Hook not firing**: Check hook name matches exactly (case-sensitive)

## Best Practices

1. **Keep plugins focused**: One plugin = one feature
2. **Handle errors gracefully**: Always output valid JSON
3. **Use minimal permissions**: Only request what you need
4. **Version your plugins**: Use semantic versioning
5. **Document your plugin**: Clear description in `plugin.json`

## Publishing Plugins

To share your plugin:

1. Create a GitHub repository with your plugin
2. Ensure `plugin.json` is complete and valid
3. Users can install via:
   ```bash
   opencrust plugin install https://github.com/yourname/your-plugin
   ```

## Support

- Open an issue on the plugin's GitHub repository
- Check the OpenCrust documentation
- Review existing plugin examples

---

*Built with OpenCrust v0.1.3*
