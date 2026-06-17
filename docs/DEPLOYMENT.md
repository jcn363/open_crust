# OpenCrust Deployment Guide

> **Version:** v0.1.3 | **Last updated:** 2026-06-17

This guide covers deploying OpenCrust in various environments, from local development to air-gapped enterprise installations.

---

## Table of Contents

1. [Ollama Local Setup](#1-ollama-local-setup)
2. [Air-Gapped Deployment](#2-air-gapped-deployment)
3. [Custom Model Endpoints](#3-custom-model-endpoints)
4. [Compliance Checklist](#4-compliance-checklist)
5. [MCP Server Isolation](#5-mcp-server-isolation)

---

## 1. Ollama Local Setup

Ollama provides fully local LLM inference with no data leaving your machine.

### Installing Ollama

```bash
# Linux/macOS
curl -fsSL https://ollama.ai/install.sh | sh

# Verify installation
ollama --version
```

### Pulling Models

```bash
# Code-focused models
ollama pull codellama        # 7B parameters, good for code
ollama pull deepseek-coder   # 6.7B parameters, excellent for code
ollama pull starcoder2       # 3B-15B parameters

# General-purpose models
ollama pull llama3           # 8B parameters
ollama pull mistral          # 7B parameters
ollama pull phi3             # 3.8B parameters

# List installed models
ollama list
```

### Configuring OpenCrust

Create or edit `~/.config/opencrust/config.json`:

```json
{
    "provider": "ollama",
    "model": "codellama",
    "ollama_url": "http://localhost:11434"
}
```

### Verifying Connection

```bash
# Test Ollama is running
curl http://localhost:11434/api/tags

# Start OpenCrust - it should auto-detect Ollama
opencrust
```

### Performance Tuning

```bash
# Set number of GPU layers (if using GPU)
ollama run codellama --num-gpu 35

# Set context window
OLLAMA_NUM_CTX=4096 ollama serve
```

---

## 2. Air-Gapped Deployment

For environments where no internet access is available.

### Pre-Download Models (on connected machine)

```bash
# Pull all needed models
ollama pull codellama
ollama pull deepseek-coder

# Export model files
# Models are stored in ~/.ollama/models/
tar czf ollama-models.tar.gz ~/.ollama/models/
```

### Transfer to Air-Gapped Machine

```bash
# Copy via USB, secure transfer, etc.
scp ollama-models.tar.gz target-host:~/

# On target machine
cd ~/
tar xzf ollama-models.tar.gz
```

### Network Isolation

```bash
# Block all outbound traffic (except local Ollama)
sudo iptables -A OUTPUT -o lo -j ACCEPT
sudo iptables -A OUTPUT -d 127.0.0.0/8 -j ACCEPT
sudo iptables -A OUTPUT -o eth0 -j DROP

# Verify isolation
curl -s https://api.openai.com  # Should timeout
curl http://localhost:11434/api/tags  # Should work
```

### Verification Checklist

- [ ] Ollama models present: `ollama list`
- [ ] Network blocked: outbound connections fail
- [ ] OpenCrust config points to localhost Ollama
- [ ] Audit logging enabled in config
- [ ] Token budget configured

---

## 3. Custom Model Endpoints

### OpenAI-Compatible APIs

Many self-hosted solutions expose OpenAI-compatible APIs:

```json
{
    "provider": "openai",
    "model": "your-model-name",
    "openai_key": "not-needed"
}
```

Note: OpenCrust sends requests to the provider's API endpoint. For custom endpoints, you may need to use a proxy or modify the base URL.

### LocalAI

```bash
# Install LocalAI
curl -sSL https://raw.githubusercontent.com/mudler/LocalAI/master/docs/README.md | bash

# Pull a model
local-ai model download thebloke/starcoder2-3b-GGUF

# Start LocalAI
local-ai --models-path /path/to/models

# Configure OpenCrust
```

```json
{
    "provider": "openai",
    "model": "starcoder2-3b",
    "openai_key": "not-needed"
}
```

### vLLM

```bash
# Install vLLM
pip install vllm

# Start server
vllm serve meta-llama/Llama-3-8B-Instruct

# Configure OpenCrust to use it
```

### Text Generation WebUI

```bash
# Install
git clone https://github.com/oobabooga/text-generation-webui
cd text-generation-webui
./start_linux.sh

# Start with API
python server.py --api --listen
```

---

## 4. Compliance Checklist

### Data Residency

- [ ] Use local Ollama (no cloud providers)
- [ ] Verify no external API calls in audit logs
- [ ] Configure `allowed_domains: []` to block all external access

### Audit Trail

Enable in `config.json`:

```json
{
    "compliance_mode": true,
    "compliance_log_path": "/var/log/opencrust/audit.jsonl",
    "audit_retention_days": 90,
    "audit_max_size_bytes": 104857600
}
```

### Permission Model

```json
{
    "role": "developer",
    "permission": {
        "bash": {
            "rm -rf /": "deny",
            "*": "ask"
        },
        "write": {
            "/etc/*": "deny",
            "*": "allow"
        }
    }
}
```

### Network Gating

```json
{
    "allowed_domains": [
        "github.com",
        "api.github.com"
    ]
}
```

### Token Budgets

```json
{
    "token_budget_enabled": true,
    "token_budget_max_tokens": 1000000
}
```

---

## 5. MCP Server Isolation

### Running MCP Servers Locally

```json
{
    "mcp": {
        "local-server": {
            "command": "/usr/local/bin/mcp-server",
            "args": ["--config", "/etc/mcp/config.json"],
            "env": {}
        }
    }
}
```

### Firewall Rules for MCP

```bash
# Block MCP server from making external connections
sudo iptables -A OUTPUT -m owner --uid-owner mcp-server -d 127.0.0.0/8 -j ACCEPT
sudo iptables -A OUTPUT -m owner --uid-owner mcp-server -j DROP
```

### Verifying MCP Server Integrity

```bash
# Check MCP server binary
sha256sum /usr/local/bin/mcp-server

# Verify against known good hash
echo "expected-hash  /usr/local/bin/mcp-server" | sha256sum -c -
```

---

## 6. Running as a Service

### systemd (Linux)

Create `/etc/systemd/system/opencrust.service`:

```ini
[Unit]
Description=OpenCrust AI Coding Agent
After=network.target ollama.service

[Service]
Type=simple
User=developer
WorkingDirectory=/home/developer
ExecStart=/usr/local/bin/opencrust
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable opencrust
sudo systemctl start opencrust
sudo journalctl -u opencrust -f
```

### Launchd (macOS)

Create `~/Library/LaunchAgents/com.opencrust.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.opencrust</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/opencrust</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

```bash
launchctl load ~/Library/LaunchAgents/com.opencrust.plist
```

---

## 7. Troubleshooting

### Ollama Connection Failed

```bash
# Check Ollama is running
systemctl status ollama

# Check port
netstat -tlnp | grep 11434

# Test manually
curl http://localhost:11434/api/tags
```

### Permission Denied

```bash
# Check file permissions
ls -la ~/.config/opencrust/

# Fix permissions
chmod 600 ~/.config/opencrust/config.json
chmod 700 ~/.config/opencrust/
```

### Token Budget Exhausted

```bash
# Check current usage
opencrust  # Then type /cost

# Reset budget
opencrust  # Then type /budget 2000000
```

---

*For more information, see [CONFIGURATION.md](CONFIGURATION.md), [SECURITY.md](SECURITY.md), and [COMPLIANCE.md](COMPLIANCE.md).*
