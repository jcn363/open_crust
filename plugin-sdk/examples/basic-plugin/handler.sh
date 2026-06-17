#!/bin/bash
# Basic Example Plugin Handler
# Receives JSON on stdin, outputs JSON on stdout

INPUT=$(cat)
HOOK=$(echo "$INPUT" | jq -r '.hook')
TOOL=$(echo "$INPUT" | jq -r '.context.tool // empty')
CONFIG_GREETING=$(echo "$INPUT" | jq -r '.config.greeting // "Hello from plugin!"')

case "$HOOK" in
    "on_startup")
        echo "{\"message\": \"$CONFIG_GREETING\"}"
        ;;
    "on_shutdown")
        echo '{"message": "Basic example plugin unloaded."}'
        ;;
    "on_message")
        MESSAGE=$(echo "$INPUT" | jq -r '.context.message // empty')
        if [ -n "$MESSAGE" ]; then
            echo "{\"message\": \"Plugin received: $MESSAGE\"}"
        else
            echo '{}'
        fi
        ;;
    "on_tool_execute")
        if [ "$TOOL" = "example_tool" ]; then
            INPUT_TEXT=$(echo "$INPUT" | jq -r '.context.input.input // empty')
            echo "{\"message\": \"Processing: $INPUT_TEXT\"}"
        else
            echo '{}'
        fi
        ;;
    *)
        echo '{}'
        ;;
esac
